use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Deserialize, Serialize, Default, FromRow)]
pub struct Track {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: String,
    pub thumbnail: String,
    #[sqlx(default)]
    #[serde(skip)]
    pub path: Option<String>,
}