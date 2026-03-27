use serde::{Serialize, Deserialize};
use crate::model::Track;

#[derive(Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub track:   Track,
    pub percent: f32,
    pub speed:   String,
}