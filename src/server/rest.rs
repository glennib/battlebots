use axum::Json;
use axum::Router;
use axum::routing::get;

use crate::workloads::inty::Payload as Inty;
use crate::workloads::mixed::Payload as Mixed;
use crate::workloads::stringy::Payload as Stringy;

pub fn router() -> Router {
    Router::new()
        .route("/inty", get(inty))
        .route("/stringy", get(stringy))
        .route("/mixed", get(mixed))
}

async fn inty() -> Json<Inty> {
    let mut rng = rand::rng();
    let payload = Inty::rand(&mut rng);
    Json(payload)
}

async fn stringy() -> Json<Stringy> {
    let mut rng = rand::rng();
    let payload = Stringy::rand(&mut rng);
    Json(payload)
}

async fn mixed() -> Json<Mixed> {
    let mut rng = rand::rng();
    let payload = Mixed::rand(&mut rng);
    Json(payload)
}
