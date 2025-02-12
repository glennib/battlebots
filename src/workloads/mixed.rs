use rand::Rng;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Payload {
    pub stringy: super::stringy::Payload,
    pub inty: super::inty::Payload,
}

impl Payload {
    pub fn rand(rng: &mut impl Rng) -> Self {
        Self {
            stringy: super::stringy::Payload::rand(rng),
            inty: super::inty::Payload::rand(rng),
        }
    }
}
