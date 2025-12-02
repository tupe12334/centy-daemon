mod config;
mod issue;
mod manifest;
mod reconciliation;
mod server;
mod utils;

use server::proto::centy_daemon_server::CentyDaemonServer;
use server::CentyDaemonService;
use tonic::transport::Server;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

const DEFAULT_ADDR: &str = "127.0.0.1:50051";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Parse address from environment or use default
    let addr = std::env::var("CENTY_DAEMON_ADDR")
        .unwrap_or_else(|_| DEFAULT_ADDR.to_string())
        .parse()?;

    let service = CentyDaemonService::new();

    info!("Starting Centy daemon on {}", addr);

    Server::builder()
        .add_service(CentyDaemonServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
