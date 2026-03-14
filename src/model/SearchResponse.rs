// ─── DTO nuevo ────────────────────────────────────────────────────────────────

use serde::Serialize;

#[derive(Serialize)]
#[serde(tag = "type", content = "data")]
pub enum SearchResponse {
    Results(Vec<TrackDto>),  // texto libre → front elige
    Queued,                  // ID directo  → ya se encoló
}

#[derive(Serialize)]
pub struct TrackDto {
    pub id:        String,
    pub title:     String,
    pub artist:    String,
    pub thumbnail: Option<String>,
}