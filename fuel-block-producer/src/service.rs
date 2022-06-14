use crate::task::Task;
use crate::Config;
use fuel_core_interfaces::block_producer::BlockProducerMpsc;
use parking_lot::Mutex;
use tokio::{sync::mpsc, task::JoinHandle};
use tracing::info;

/// Primary entrypoint for the block producer.
/// Manages tasks related to block production.
pub struct Service {
    join: Mutex<Option<JoinHandle<()>>>,
    sender: mpsc::Sender<BlockProducerMpsc>,
    config: Config,
}

impl Service {
    pub async fn new(
        config: &Config,
        _db: (),
        sender: mpsc::Sender<BlockProducerMpsc>,
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            sender,
            join: Mutex::new(None),
            config: config.clone(),
        })
    }

    pub async fn start(&self, receiver: mpsc::Receiver<BlockProducerMpsc>, _txpool: ()) {
        let join = self.join.lock();
        if join.is_none() {
            let task = Task {
                receiver,
                config: self.config.clone(),
                db: todo!(),
                relayer: todo!(),
                txpool: todo!(),
            }
            .spawn();
            tokio::pin!(task);
            *join = Some(tokio::spawn(task));
        }
    }

    pub async fn stop(&self) -> Option<JoinHandle<()>> {
        info!("stopping block producer service...");
        let join = self.join.lock().take();
        if join.is_some() {
            let _ = self.sender.send(BlockProducerMpsc::Stop);
        }
        join
    }

    pub fn sender(&self) -> &mpsc::Sender<BlockProducerMpsc> {
        &self.sender
    }
}
