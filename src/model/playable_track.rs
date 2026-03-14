use crate::model::Track;
#[derive(Debug, Clone)]
pub struct AudioProperties {
    pub sample_rate: u32,      // 48000
    pub channels: u8,           // 2 (stereo)
    pub bit_depth: Option<u8>,  // 24, 16, None si es lossy (mp3)
    pub codec: String,          // "flac", "mp3", "aac"
    pub duration_secs: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct PlayableTrack {
    pub track: Track,             // metadatos de negocio
    pub path: String,             // /cache/xyz.flac
    pub audio: AudioProperties,   // leído por symphonia al momento de encolar
}