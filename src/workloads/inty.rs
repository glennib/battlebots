use std::collections::HashMap;

use rand::Rng;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Payload {
    pub header: String,
    pub configuration: HashMap<String, i64>,
    pub ids: Vec<i64>,
}

impl Payload {
    pub fn rand(rng: &mut impl Rng) -> Self {
        let n_words = rng.random_range(super::COLLECTION_SIZE);
        let header = super::words(rng, n_words);
        let n_configs = rng.random_range(super::COLLECTION_SIZE);
        let mut configuration = HashMap::with_capacity(n_configs);
        for _ in 0..n_configs {
            configuration.insert(super::word(rng), rng.random());
        }
        let n_ids = rng.random_range(super::COLLECTION_SIZE);
        let mut ids = Vec::with_capacity(n_ids);
        for _ in 0..n_ids {
            ids.push(rng.random())
        }
        Self {
            header,
            configuration,
            ids,
        }
    }
}
