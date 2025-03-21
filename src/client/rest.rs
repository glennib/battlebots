use std::sync::Arc;

use async_trait::async_trait;

use crate::workloads::inty;
use crate::workloads::mixed;
use crate::workloads::stringy;

#[derive(Clone)]
#[allow(clippy::struct_field_names)]
pub struct Client {
    client: reqwest::Client,
    url_stringy: Arc<str>,
    url_inty: Arc<str>,
    url_mixed: Arc<str>,
}

impl Client {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: reqwest::Client::builder()
                .http2_prior_knowledge()
                .pool_max_idle_per_host(1)
                .build()
                .unwrap(),
            url_stringy: format!("{base_url}/stringy").into(),
            url_inty: format!("{base_url}/inty").into(),
            url_mixed: format!("{base_url}/mixed").into(),
        }
    }
}

#[async_trait]
impl super::Client for Client {
    type Stringy = stringy::Payload;
    type Inty = inty::Payload;
    type Mixed = mixed::Payload;
    type Error = reqwest::Error;

    async fn stringy(&mut self) -> Result<Self::Stringy, Self::Error> {
        self.client
            .get(self.url_stringy.as_ref())
            .send()
            .await?
            .json()
            .await
    }

    async fn inty(&mut self) -> Result<Self::Inty, Self::Error> {
        self.client
            .get(self.url_inty.as_ref())
            .send()
            .await?
            .json()
            .await
    }

    async fn mixed(&mut self) -> Result<Self::Mixed, Self::Error> {
        self.client
            .get(self.url_mixed.as_ref())
            .send()
            .await?
            .json()
            .await
    }
}
