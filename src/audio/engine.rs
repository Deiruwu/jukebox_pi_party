// src/audio/engine.rs

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use rodio::Source;
use crate::model::PlayableTrack;
use rodio::cpal::traits::{DeviceTrait, HostTrait};

// ─── ERROR ────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum EngineError {
    OutputDeviceUnavailable,
    FileNotFound(String),
    DecodeFailed(String),
    NoTrackLoaded,
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EngineError::OutputDeviceUnavailable          => write!(f, "No audio output device available"),
            EngineError::FileNotFound(p)          => write!(f, "File not found: {}", p),
            EngineError::DecodeFailed(e)          => write!(f, "Decode failed: {}", e),
            EngineError::NoTrackLoaded                    => write!(f, "No track currently loaded"),
        }
    }
}

impl std::error::Error for EngineError {}

// ─── ESTADO ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
}

// ─── ENGINE ───────────────────────────────────────────────────────────────────

/// AudioEngine es el único responsable de hablarle a PipeWire/ALSA.
/// No sabe nada de colas ni comandos — solo recibe rutas y controla playback.
pub struct AudioEngine {
    // _stream debe vivir mientras el engine exista, si se dropea se corta el audio
    _stream: OutputStream,
    handle: OutputStreamHandle,
    sink: Arc<Mutex<Option<Sink>>>,
    state: Arc<Mutex<PlaybackState>>,
    volume: Arc<Mutex<f32>>,
}

impl AudioEngine {
    /// Inicializa el dispositivo de salida. Falla rápido si no hay dispositivo.
pub fn new() -> Result<Self, EngineError> {
        // 1. Obtenemos el host de ALSA (el estándar en la Pi)
        let host = rodio::cpal::default_host();
        
        // 2. Enumeramos dispositivos buscando "bluealsa" o el nombre de tu Echo
        // Tip: BlueALSA suele exponerse con ese nombre en el driver de ALSA
        let device = host.output_devices()
            .map_err(|_| EngineError::OutputDeviceUnavailable)?
            .find(|d| {
                d.name()
                    .map(|n| n.to_lowercase().contains("bluealsa") || n.contains("Echo Pop"))
                    .unwrap_or(false)
            })
            // Fallback al default si el Bluetooth está apagado para que la app no truene
            .or_else(|| host.default_output_device())
            .ok_or(EngineError::OutputDeviceUnavailable)?;

        println!("[AUDIO] Usando dispositivo: {:?}", device.name().unwrap_or_default());

        // 3. Inicializamos el stream con el dispositivo específico
        let (_stream, handle) = OutputStream::try_from_device(&device)
            .map_err(|_| EngineError::OutputDeviceUnavailable)?;

        Ok(Self {
            _stream,
            handle,
            sink: Arc::new(Mutex::new(None)),
            state: Arc::new(Mutex::new(PlaybackState::Stopped)),
            volume: Arc::new(Mutex::new(1.0)),
        })
    }
    // ─── PLAYBACK ─────────────────────────────────────────────────────────────

    /// Carga y reproduce un PlayableTrack. Detiene lo que esté sonando antes.
    pub fn play(&self, playable: &PlayableTrack) -> Result<(), EngineError> {
        self.stop_sink();

        let extension = std::path::Path::new(&playable.path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let source: Box<dyn Source<Item = i16> + Send> = match extension {
            "m4a" => {
                let output = std::process::Command::new("ffmpeg")
                    .args([
                        "-i", &playable.path,
                        "-f", "wav",
                        "-ar", "48000",
                        "-ac", "2",
                        "pipe:1",
                    ])
                    .output()
                    .map_err(|e| EngineError::DecodeFailed(e.to_string()))?;

                let cursor = std::io::Cursor::new(output.stdout);
                Box::new(Decoder::new(cursor)
                    .map_err(|e| EngineError::DecodeFailed(e.to_string()))?)
            }
            _ => {
                let file = File::open(&playable.path)
                    .map_err(|_| EngineError::FileNotFound(playable.path.clone()))?;
                Box::new(Decoder::new(BufReader::new(file))
                    .map_err(|e| EngineError::DecodeFailed(e.to_string()))?)
            }
        };

        let sink = Sink::try_new(&self.handle)
            .map_err(|_| EngineError::OutputDeviceUnavailable)?;

        let vol = *self.volume.lock().unwrap();
        sink.set_volume(vol);
        sink.append(source);

        *self.sink.lock().unwrap() = Some(sink);
        *self.state.lock().unwrap() = PlaybackState::Playing;

        Ok(())
    }
    pub fn pause(&self) -> Result<(), EngineError> {
        let sink = self.sink.lock().unwrap();
        match sink.as_ref() {
            Some(s) => {
                s.pause();
                *self.state.lock().unwrap() = PlaybackState::Paused;
                Ok(())
            }
            None => Err(EngineError::NoTrackLoaded),
        }
    }

    pub fn resume(&self) -> Result<(), EngineError> {
        let sink = self.sink.lock().unwrap();
        match sink.as_ref() {
            Some(s) => {
                s.play();
                *self.state.lock().unwrap() = PlaybackState::Playing;
                Ok(())
            }
            None => Err(EngineError::NoTrackLoaded),
        }
    }

    pub fn stop(&self) {
        self.stop_sink();
        *self.state.lock().unwrap() = PlaybackState::Stopped;
    }

    // ─── VOLUMEN ──────────────────────────────────────────────────────────────

    /// Valor entre 0.0 y 1.0. Se persiste para aplicarlo al siguiente track también.
    pub fn set_volume(&self, level: f32) {
        let level = level.clamp(0.0, 1.0);
        *self.volume.lock().unwrap() = level;

        if let Some(sink) = self.sink.lock().unwrap().as_ref() {
            sink.set_volume(level);
        }
    }

    pub fn get_volume(&self) -> f32 {
        *self.volume.lock().unwrap()
    }

    // ─── ESTADO ───────────────────────────────────────────────────────────────

    pub fn state(&self) -> PlaybackState {
        self.state.lock().unwrap().clone()
    }

    /// Devuelve true si el sink terminó de reproducir (canción acabó sola)
    pub fn is_finished(&self) -> bool {
        self.sink
            .lock()
            .unwrap()
            .as_ref()
            .map(|s| s.empty())
            .unwrap_or(true)
    }

    // ─── INTERNO ──────────────────────────────────────────────────────────────

    fn stop_sink(&self) {
        if let Some(sink) = self.sink.lock().unwrap().take() {
            sink.stop();
        }
    }
}
