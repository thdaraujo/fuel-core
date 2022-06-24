use crate::{Config, Relayer};
use anyhow::Error;
use ethers_providers::{Http, Middleware, Provider, ProviderError, Ws};
use fuel_core_interfaces::{
    block_importer::NewBlockEvent,
    relayer::{RelayerDb, RelayerEvent},
};
use std::sync::Arc;
use tokio::{
    sync::{broadcast, mpsc},
    task::JoinHandle,
};
use url::Url;

const PROVIDER_INTERVAL: u64 = 1000;

pub struct Service {
    stop_join: Option<JoinHandle<()>>,
    sender: mpsc::Sender<RelayerEvent>,
}

impl Service {
    pub async fn new<P>(
        config: &Config,
        private_key: &[u8],
        db: Box<dyn RelayerDb>,
        new_block_event: broadcast::Receiver<NewBlockEvent>,
        provider: Arc<P>,
        interface: Arc<Interface>,
        sender: mpsc::Sender<TxPoolMpsc>,
        broadcast: broadcast::Sender<TxStatusBroadcast>,
        join: Mutex<Option<JoinHandle<mpsc::Receiver<TxPoolMpsc>>>>,
        receiver: Arc<Mutex<Option<mpsc::Receiver<TxPoolMpsc>>>>,
    ) -> Result<Self, anyhow::Error>
    where
        P: Middleware<Error = ProviderError> + 'static,
    {
        let (sender, receiver) = mpsc::channel(100);
        let relayer =
            Relayer::new(config.clone(), private_key, db, receiver, new_block_event).await;

        let stop_join = Some(tokio::spawn(Relayer::run(relayer, provider)));
        Ok(Self { sender, stop_join })
    }

    pub async fn start(&self) -> bool {
        let mut join = self.join.lock().await;
        if join.is_none() {
            if let Some(receiver) = self.receiver.lock().await.take() {
                let interface = self.interface.clone();
                *join = Some(tokio::spawn(async {
                    interface.run(new_block, receiver).await
                }));
                return true;
            } else {
                warn!("Starting FuelRelayer service that is stopping");
            }
        } else {
            warn!("Service FuelRelayer is already started");
        }
        false
    }

    pub async fn stop(&mut self) {
        let mut join = self.join.lock().await;
        let join_handle = join.take();
        if let Some(join_handle) = join_handle {
            let _ = self.sender.send(TxPoolMpsc::Stop).await;
            let receiver = self.receiver.clone();
            Some(tokio::spawn(async move {
                let ret = join_handle.await;
                *receiver.lock().await = ret.ok();
            }))
        } else {
            None
        }
    }

    /// create provider that we use for communication with ethereum.
    pub async fn provider(uri: &str) -> Result<Provider<Ws>, Error> {
        let ws = Ws::connect(uri).await?;
        let provider =
            Provider::new(ws).interval(std::time::Duration::from_millis(PROVIDER_INTERVAL));
        Ok(provider)
    }

    pub fn provider_http(uri: &str) -> Result<Provider<Http>, Error> {
        let url = Url::parse(uri).unwrap();
        let ws = Http::new(url);
        let provider =
            Provider::new(ws).interval(std::time::Duration::from_millis(PROVIDER_INTERVAL));
        Ok(provider)
    }

    pub fn sender(&self) -> &mpsc::Sender<RelayerEvent> {
        &self.sender
    }
}


pub struct Service {
    interface: Arc<Interface>,
    sender: mpsc::Sender<TxPoolMpsc>,
    broadcast: broadcast::Sender<TxStatusBroadcast>,
    join: Mutex<Option<JoinHandle<mpsc::Receiver<TxPoolMpsc>>>>,
    receiver: Arc<Mutex<Option<mpsc::Receiver<TxPoolMpsc>>>>,
}

impl Service {
    pub fn new(db: Box<dyn TxPoolDb>, config: Config) -> Result<Self, anyhow::Error> {
        let (sender, receiver) = mpsc::channel(100);
        let (broadcast, _receiver) = broadcast::channel(100);
        Ok(Self {
            interface: Arc::new(Interface::new(db, broadcast.clone(), config)),
            sender,
            broadcast,
            join: Mutex::new(None),
            receiver: Arc::new(Mutex::new(Some(receiver))),
        })
    }

    pub async fn start(&self, new_block: broadcast::Receiver<ImportBlockBroadcast>) -> bool {
        let mut join = self.join.lock().await;
        if join.is_none() {
            if let Some(receiver) = self.receiver.lock().await.take() {
                let interface = self.interface.clone();
                *join = Some(tokio::spawn(async {
                    interface.run(new_block, receiver).await
                }));
                return true;
            } else {
                warn!("Starting TxPool service that is stopping");
            }
        } else {
            warn!("Service TxPool is already started");
        }
        false
    }

    pub async fn stop(&self) -> Option<JoinHandle<()>> {
        let mut join = self.join.lock().await;
        let join_handle = join.take();
        if let Some(join_handle) = join_handle {
            let _ = self.sender.send(TxPoolMpsc::Stop).await;
            let receiver = self.receiver.clone();
            Some(tokio::spawn(async move {
                let ret = join_handle.await;
                *receiver.lock().await = ret.ok();
            }))
        } else {
            None
        }
    }

    pub fn subscribe_ch(&self) -> broadcast::Receiver<TxStatusBroadcast> {
        self.broadcast.subscribe()
    }

    pub fn sender(&self) -> &mpsc::Sender<TxPoolMpsc> {
        &self.sender
    }
}