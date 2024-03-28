use axum::{
    extract::{Json, Query},
    http::StatusCode,
};
use warp::multipass::identity::Identity;

pub async fn create_identity(
    Query(_query): Query<CreateIdentityQuery>,
) -> Result<Json<Identity>, StatusCode> {
    let dummy = Identity::default();

    Ok(Json(dummy))
}

#[derive(serde::Deserialize)]
pub struct CreateIdentityQuery {
    pub passphrase: String,
    pub seed_words: String,
}
