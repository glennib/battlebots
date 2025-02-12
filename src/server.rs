use std::net::SocketAddr;

use tonic::service::Routes;
use tracing::{info, instrument};
use crate::proto::battlebots_service_server::BattlebotsServiceServer;

pub mod grpc;
pub mod rest;

#[instrument]
pub async fn run(addr: &SocketAddr) -> anyhow::Result<()> {
    let service = grpc::BattlebotsService;
    let service = BattlebotsServiceServer::new(service);
    let grpc_router = Routes::new(service).prepare().into_axum_router();
    let http_router = rest::router();
    let router = grpc_router.merge(http_router);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("listening");
    axum::serve(listener, router).await?;
    Ok(())
}
