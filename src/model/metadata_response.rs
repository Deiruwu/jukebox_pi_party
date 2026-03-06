use serde::Deserialize;

use crate::model::track::Track;

#[derive(Deserialize, Debug)]
pub struct ApiResponse {
    pub status: String,
    pub data: Option<Vec<Track>>,
    pub message: Option<String>,
}