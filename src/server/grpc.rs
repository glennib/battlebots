use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::time::Duration;

use tonic::Request;
use tonic::Response;
use tonic::Status;
use tracing::info;

use crate::proto::Empty;
use crate::proto::Inty;
use crate::proto::Mixed;
use crate::proto::Stringy;
use crate::proto::battlebots_service_server::BattlebotsService as Svc;

#[derive(Default)]
struct Parallelism {
    count: AtomicUsize,
}

impl Parallelism {
    fn lease(&self) -> Lease<'_> {
        self.count.fetch_add(1, Ordering::SeqCst);
        Lease { p: self }
    }
}

struct Lease<'p> {
    p: &'p Parallelism,
}

impl Drop for Lease<'_> {
    fn drop(&mut self) {
        self.p.count.fetch_sub(1, Ordering::SeqCst);
    }
}

pub struct BattlebotsService {
    parallelism: Arc<Parallelism>,
    logger: Option<(std::thread::JoinHandle<()>, mpsc::SyncSender<()>)>,
}

impl Drop for BattlebotsService {
    fn drop(&mut self) {
        if let Some((jh, cancel)) = self.logger.take() {
            let _ = cancel.send(());
            let _ = jh.join();
        }
    }
}

impl BattlebotsService {
    pub fn with_logger() -> Self {
        let parallelism = Arc::new(Parallelism::default());
        let parallelism_2 = Arc::clone(&parallelism);
        let (tx, rx) = mpsc::sync_channel(0);
        let jh = std::thread::spawn(move || {
            loop {
                if let Ok(()) = rx.recv_timeout(Duration::from_millis(1000)) {
                    break;
                }
                let parallelism = parallelism_2.count.load(Ordering::SeqCst);
                info!(parallelism);
            }
        });
        Self {
            parallelism,
            logger: Some((jh, tx)),
        }
    }

    pub fn without_logger() -> Self {
        Self {
            parallelism: Arc::new(Parallelism::default()),
            logger: None,
        }
    }
}

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
        let _l = self.parallelism.lease();
        let stringy = crate::workloads::stringy::Payload::rand(&mut rand::rng());
        Ok(Response::new(stringy.into()))
    }

    async fn get_inty(&self, _request: Request<Empty>) -> Result<Response<Inty>, Status> {
        let _l = self.parallelism.lease();
        let inty = crate::workloads::inty::Payload::rand(&mut rand::rng());
        Ok(Response::new(inty.into()))
    }

    async fn get_mixed(&self, _request: Request<Empty>) -> Result<Response<Mixed>, Status> {
        let _l = self.parallelism.lease();
        let mixed = crate::workloads::mixed::Payload::rand(&mut rand::rng());
        Ok(Response::new(mixed.into()))
    }
}
