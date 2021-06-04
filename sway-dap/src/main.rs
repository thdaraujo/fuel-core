use structopt::StructOpt;
use sway_dap::Service;
use tracing::{info, trace};

use std::io;

mod args;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = args::Opt::from_args().exec()?;

    info!("Sway-DAP online");
    trace!("Initiating in TRACE mode");

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut service = Service::from(addr);

    while service.handle(stdin.lock(), stdout.lock()).await? {}

    info!("Sway-DAP shutting down!");

    Ok(())
}
