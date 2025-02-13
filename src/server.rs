use std::net::SocketAddr;

use tonic::service::Routes;
use tracing::info;
use tracing::instrument;

use crate::proto::battlebots_service_server::BattlebotsServiceServer;

pub mod grpc;
pub mod rest;

#[instrument]
pub async fn run(addr: &SocketAddr, grpc_only: bool, grpc_logger: bool) -> anyhow::Result<()> {
    let service = if grpc_logger {
        grpc::BattlebotsService::with_logger()
    } else {
        grpc::BattlebotsService::without_logger()
    };
    let service = BattlebotsServiceServer::new(service);
    if grpc_only {
        info!("listening");
        tonic::transport::Server::builder()
            .add_service(service)
            .serve(*addr)
            .await?;
    } else {
        let grpc_router = Routes::new(service).prepare().into_axum_router();
        let http_router = rest::router();
        let router = grpc_router.merge(http_router);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        info!("listening");
        axum::serve(listener, router).await?;
    }
    Ok(())
}
