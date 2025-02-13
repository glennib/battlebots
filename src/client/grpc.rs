use async_trait::async_trait;

use crate::proto::Inty;
use crate::proto::Mixed;
use crate::proto::Stringy;
pub use crate::proto::battlebots_service_client::BattlebotsServiceClient as Client;

#[async_trait]
impl super::Client for Client<tonic::transport::Channel> {
    type Stringy = Stringy;
    type Inty = Inty;
    type Mixed = Mixed;
    type Error = tonic::Status;

    async fn stringy(&mut self) -> Result<Self::Stringy, Self::Error> {
        let response = self
            .get_stringy(tonic::Request::new(crate::proto::Empty {}))
            .await?;
        Ok(response.into_inner())
    }

    async fn inty(&mut self) -> Result<Self::Inty, Self::Error> {
        let response = self
            .get_inty(tonic::Request::new(crate::proto::Empty {}))
            .await?;
        Ok(response.into_inner())
    }

    async fn mixed(&mut self) -> Result<Self::Mixed, Self::Error> {
        let response = self
            .get_mixed(tonic::Request::new(crate::proto::Empty {}))
            .await?;
        Ok(response.into_inner())
    }
}
