mod config;
mod docs;
mod issue;
mod manifest;
mod migration;
mod reconciliation;
mod registry;
mod server;
mod template;
mod utils;
mod version;

use http::header::{ACCEPT, CONTENT_TYPE};
use http::Method;
use server::proto::centy_daemon_server::CentyDaemonServer;
use server::CentyDaemonService;
use tonic::transport::Server;
use tower_http::cors::{AllowOrigin, CorsLayer};
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

    // Configure CORS for gRPC-Web
    // In development, allow localhost origins. In production, configure appropriately.
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(|origin, _| {
            // Allow localhost origins for development
            if let Ok(origin_str) = origin.to_str() {
                origin_str.starts_with("http://localhost")
                    || origin_str.starts_with("http://127.0.0.1")
                    || origin_str.starts_with("https://localhost")
                    || origin_str.starts_with("https://127.0.0.1")
            } else {
                false
            }
        }))
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            ACCEPT,
            CONTENT_TYPE,
            "x-grpc-web".parse().unwrap(),
            "x-user-agent".parse().unwrap(),
            "grpc-timeout".parse().unwrap(),
        ])
        .expose_headers([
            "grpc-status".parse().unwrap(),
            "grpc-message".parse().unwrap(),
            "grpc-status-details-bin".parse().unwrap(),
        ]);

    info!("Starting Centy daemon on {} (gRPC + gRPC-Web)", addr);

    Server::builder()
        .accept_http1(true) // Required for gRPC-Web
        .layer(cors)
        .layer(tonic_web::GrpcWebLayer::new())
        .add_service(reflection_service)
        .add_service(CentyDaemonServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
