use axum::async_trait;

pub mod grpc;
pub mod rest;

#[async_trait]
pub trait Client {
    type Stringy;
    type Inty: Send;
    type Mixed;
    type Error: std::error::Error + Send;
    async fn stringy(&mut self) -> Result<Self::Stringy, Self::Error>;
    async fn inty(&mut self) -> Result<Self::Inty, Self::Error>;
    async fn mixed(&mut self) -> Result<Self::Mixed, Self::Error>;
}
