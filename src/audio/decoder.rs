use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default::get_probe;
use std::fs::File;
use std::path::Path;
use crate::model::{AudioProperties, PlayableTrack, Track};

#[derive(Debug)]
pub enum DecodeError {
    FileNotFound(String),
    UnsupportedFormat(String),
    NoAudioStream,
    MissingCodecParams(String),
    IoError(std::io::Error),
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DecodeError::FileNotFound(p)       => write!(f, "File not found: {}", p),
            DecodeError::UnsupportedFormat(p)  => write!(f, "Unsupported format: {}", p),
            DecodeError::NoAudioStream                 => write!(f, "No audio stream found in file"),
            DecodeError::MissingCodecParams(s) => write!(f, "Missing codec params: {}", s),
            DecodeError::IoError(e)             => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for DecodeError {}

impl From<std::io::Error> for DecodeError {
    fn from(e: std::io::Error) -> Self {
        DecodeError::IoError(e)
    }
}

/// Abre el archivo, extrae propiedades técnicas y devuelve un PlayableTrack.
/// Falla rápido con error explícito si algo no está bien.
pub fn probe_file(path: &str, track: Track) -> Result<PlayableTrack, DecodeError> {
    let path_obj = Path::new(path);

    if !path_obj.exists() {
        return Err(DecodeError::FileNotFound(path.to_string()));
    }

    // Hint de extensión para que symphonia no tenga que adivinar
    let mut hint = Hint::new();
    if let Some(ext) = path_obj.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let file = File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let probed = get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| DecodeError::UnsupportedFormat(e.to_string()))?;

    let format = probed.format;

    let audio_track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or(DecodeError::NoAudioStream)?;

    let params = &audio_track.codec_params;

    let sample_rate = params.sample_rate.unwrap_or(0);

    let channels = params
        .channels
        .map(|c| c.count() as u8)
        .unwrap_or(2); // fallback razonable para AAC

    let bit_depth = params.bits_per_sample.map(|b| b as u8);

    let codec = symphonia::default::get_codecs()
        .get_codec(params.codec)
        .map(|d| d.short_name.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let duration_secs = params.n_frames
        .zip(params.sample_rate)
        .map(|(frames, rate)| frames / rate as u64);

    let audio_props = AudioProperties {
        sample_rate,
        channels,
        bit_depth,
        codec,
        duration_secs,
    };

    Ok(PlayableTrack {
        track,
        path: path.to_string(),
        audio: audio_props,
    })
}