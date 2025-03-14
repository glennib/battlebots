use std::net::SocketAddr;

use tracing::info;
use tracing::instrument;

use crate::proto::battlebots_service_server::BattlebotsServiceServer;

pub mod grpc;
pub mod rest;

#[instrument]
pub async fn run_http(addr: &SocketAddr) -> anyhow::Result<()> {
    let router = rest::router();
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("listening");
    axum::serve(listener, router).await?;
    Ok(())
}

#[instrument]
pub async fn run_grpc(addr: &SocketAddr) -> anyhow::Result<()> {
    let service = grpc::BattlebotsService;
    let server = BattlebotsServiceServer::new(service);
    info!("listening");
    tonic::transport::Server::builder()
        .add_service(server)
        .serve(*addr)
        .await?;
    Ok(())
}
