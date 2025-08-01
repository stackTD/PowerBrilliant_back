use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct SharePostPayload {
    pub post_id: Uuid,
    pub shared_by: Uuid,
    pub shared_with: Vec<Uuid>,
    pub message: Option<String>,
}

#[derive(Serialize)]
pub struct SharePostResponse {
    pub status: String,
    pub message: String,
}