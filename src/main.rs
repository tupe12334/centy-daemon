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

// Include the file descriptor set for gRPC reflection
pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("centy_descriptor");

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

    // Create reflection service
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1()?;

    info!("Starting Centy daemon on {}", addr);

    Server::builder()
        .add_service(reflection_service)
        .add_service(CentyDaemonServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
