use structopt::StructOpt;
use sway_dap::Service;
use tracing::{error, info, trace};

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

    while match service.handle(stdin.lock(), stdout.lock()).await {
        Ok(proceed) => proceed,
        Err(e) => {
            error!("Error handling request: {}", e);
            true
        }
    } {}

    info!("Sway-DAP shutting down!");

    Ok(())
}
