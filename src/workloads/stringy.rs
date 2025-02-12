use std::collections::HashMap;

use rand::Rng;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Payload {
    pub body: String,
    pub messages: Vec<String>,
    pub configuration: HashMap<String, String>,
}

impl Payload {
    pub fn rand(rng: &mut impl Rng) -> Self {
        let n_words = rng.random_range(super::COLLECTION_SIZE);
        let body = super::words(rng, n_words);
        let n_configs = rng.random_range(super::COLLECTION_SIZE);
        let mut configuration = HashMap::with_capacity(n_configs);
        for _ in 0..n_configs {
            configuration.insert(super::word(rng), super::word(rng));
        }
        let n_messages = rng.random_range(super::COLLECTION_SIZE);
        let mut messages = Vec::with_capacity(n_messages);
        for _ in 0..n_messages {
            let n_words = rng.random_range(super::COLLECTION_SIZE);
            messages.push(super::words(rng, n_words));
        }
        Self {
            body,
            configuration,
            messages,
        }
    }
}
