use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
};
use tracing::error;
use warp::multipass::identity::Identity;

use crate::warp::Warp;

pub async fn create_identity(
    Query(query): Query<CreateIdentityQuery>,
    State(warp): State<Warp>,
) -> Result<Json<Identity>, StatusCode> {
    warp.create_identity(query.username, query.passphrase, query.seed_words)
        .await
        .map_err(|e| {
            // FIXME: Do better error handling here.
            error!("create_identity failed: {:?}", e);
            StatusCode::BAD_REQUEST
        })
        .map(Json)
}

#[derive(serde::Deserialize)]
pub struct CreateIdentityQuery {
    pub username: String,
    pub passphrase: String,
    pub seed_words: String,
}
