use async_std::{fs::File, io::ReadExt, path::PathBuf};
use fuel_core::service::{Config, FuelService};
use fuel_wasm_executor::{IndexerConfig, IndexerService, Manifest};
use serde::Deserialize;
use structopt::StructOpt;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Indexer Service",
    about = "Standalone binary for the fuel indexer service"
)]
pub struct Args {
    #[structopt(short, long, help = "Run a fuel node locally")]
    local: bool,
    #[structopt(parse(from_os_str), help = "Indexer service config file")]
    config: PathBuf,
    #[structopt(short, long, parse(from_os_str), help = "Indexer service config file")]
    manifest: PathBuf,
}

async fn load_yaml<'a, T: for<'de> Deserialize<'de>>(filename: &PathBuf) -> anyhow::Result<T> {
    let mut file = File::open(filename).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;

    Ok(serde_yaml::from_str(&contents)?)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = match std::env::var_os("RUST_LOG") {
        Some(_) => EnvFilter::try_from_default_env().expect("Invalid `RUST_LOG` provided"),
        None => EnvFilter::new("info"),
    };

    tracing_subscriber::fmt::Subscriber::builder()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .init();

    let opt = Args::from_args();

    let mut config: IndexerConfig = load_yaml(&opt.config).await?;

    #[cfg(feature = "db-sqlite")]
    {
        let canonicalized = PathBuf::from(config.database_url)
            .canonicalize()
            .await
            .expect("Could not canonicalize path");
        config.database_url = canonicalized
            .into_os_string()
            .into_string()
            .expect("Could not stringify path");
    }

    let _local_node = if opt.local {
        let s = FuelService::new_node(Config::local_node()).await.unwrap();
        config.fuel_node_addr = s.bound_address;
        Some(s)
    } else {
        None
    };

    let mut service = IndexerService::new(config)?;

    let mut path = opt.manifest;
    let manifest: Manifest = load_yaml(&path).await?;

    path.pop();
    path.push(&manifest.graphql_schema);
    let mut file = File::open(&path).await?;
    let mut schema = String::new();
    file.read_to_string(&mut schema).await?;

    path.pop();
    path.push(&manifest.wasm_module);
    let mut file = File::open(&path).await?;
    let mut bytes = Vec::<u8>::new();
    file.read_to_end(&mut bytes).await?;
    let schema = service.add_indexer(manifest, &schema, bytes, false)?;

    info!("Indexer schema: {schema:?}");

    tokio::spawn(service.run()).await?;
    Ok(())
}