use tonic::Request;
use tonic::Response;
use tonic::Status;

use crate::proto::Empty;
use crate::proto::Inty;
use crate::proto::Mixed;
use crate::proto::Stringy;
use crate::proto::battlebots_service_server::BattlebotsService as Svc;

pub struct BattlebotsService;

impl From<crate::workloads::stringy::Payload> for Stringy {
    fn from(value: crate::workloads::stringy::Payload) -> Self {
        Self {
            configuration: value.configuration,
            body: Some(value.body),
            messages: value.messages,
        }
    }
}

impl From<crate::workloads::inty::Payload> for Inty {
    fn from(value: crate::workloads::inty::Payload) -> Self {
        Self {
            configuration: value.configuration,
            header: Some(value.header),
            ids: value.ids,
        }
    }
}

impl From<crate::workloads::mixed::Payload> for Mixed {
    fn from(value: crate::workloads::mixed::Payload) -> Self {
        Self {
            stringy: Some(value.stringy.into()),
            inty: Some(value.inty.into()),
        }
    }
}

#[tonic::async_trait]
impl Svc for BattlebotsService {
    async fn get_stringy(&self, _request: Request<Empty>) -> Result<Response<Stringy>, Status> {
        let stringy = crate::workloads::stringy::Payload::rand(&mut rand::rng());
        Ok(Response::new(stringy.into()))
    }

    async fn get_inty(&self, _request: Request<Empty>) -> Result<Response<Inty>, Status> {
        let inty = crate::workloads::inty::Payload::rand(&mut rand::rng());
        Ok(Response::new(inty.into()))
    }

    async fn get_mixed(&self, _request: Request<Empty>) -> Result<Response<Mixed>, Status> {
        let mixed = crate::workloads::mixed::Payload::rand(&mut rand::rng());
        Ok(Response::new(mixed.into()))
    }
}
