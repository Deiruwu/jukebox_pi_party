use rodio::{Decoder, Player};
use rodio::stream::{DeviceSinkBuilder, MixerDeviceSink};
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use rodio::Source;
use crate::model::PlayableTrack;
use rodio::cpal::traits::{DeviceTrait, HostTrait};
use serde::{Serialize, Deserialize};

// --- ERROR -------------------------------------------------------------------

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
            EngineError::OutputDeviceUnavailable => write!(f, "No audio output device available"),
            EngineError::FileNotFound(p)         => write!(f, "File not found: {}", p),
            EngineError::DecodeFailed(e)         => write!(f, "Decode failed: {}", e),
            EngineError::NoTrackLoaded           => write!(f, "No track currently loaded"),
        }
    }
}

impl std::error::Error for EngineError {}

// --- ESTADO ------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
}

// --- ENGINE ------------------------------------------------------------------

/// AudioEngine es el unico responsable de hablarle a PipeWire/ALSA.
/// No sabe nada de colas ni comandos — solo recibe rutas y controla playback.
pub struct AudioEngine {
    _sink:       MixerDeviceSink,
    player:      Arc<Mutex<Option<Player>>>,
    state:       Arc<Mutex<PlaybackState>>,
    volume:      Arc<Mutex<f32>>,
    current_pos: Arc<Mutex<Duration>>,
    duration:    Arc<Mutex<Option<Duration>>>,
}

impl AudioEngine {
    /// Inicializa el dispositivo de salida. Falla rapido si no hay dispositivo.
    pub fn new() -> Result<Self, EngineError> {
        let host = rodio::cpal::default_host();

        for device in host.output_devices().unwrap() {
            if let Ok(desc) = device.description() {
                println!("[AUDIO] Dispositivo encontrado: {}", desc.name());
            }
        }

        let device = host.output_devices()
            .map_err(|_| EngineError::OutputDeviceUnavailable)?
            .find(|d| {
                d.description()
                    .map(|desc| {
                        let name = desc.name().to_lowercase();
                        name.contains("bluetooth audio")
                    })
                    .unwrap_or(false)
            })
            .or_else(|| host.default_output_device())
            .ok_or(EngineError::OutputDeviceUnavailable)?;

        println!(
            "[AUDIO] Usando dispositivo: {}",
            device.description().map(|d| d.name().to_string()).unwrap_or_default()
        );

        let _sink = DeviceSinkBuilder::from_device(device)
            .map_err(|_| EngineError::OutputDeviceUnavailable)?
            .open_stream()
            .map_err(|_| EngineError::OutputDeviceUnavailable)?;

        Ok(Self {
            _sink,
            player:      Arc::new(Mutex::new(None)),
            state:       Arc::new(Mutex::new(PlaybackState::Stopped)),
            volume:      Arc::new(Mutex::new(1.0)),
            current_pos: Arc::new(Mutex::new(Duration::ZERO)),
            duration:    Arc::new(Mutex::new(None)),
        })
    }

    // --- PLAYBACK -------------------------------------------------------------

    /// Carga y reproduce un PlayableTrack. Detiene lo que este sonando antes.
    pub fn play(&self, playable: &PlayableTrack) -> Result<(), EngineError> {
        self.stop_player();

        *self.current_pos.lock().unwrap() = Duration::ZERO;

        let player = Player::connect_new(&self._sink.mixer());
        player.set_volume(*self.volume.lock().unwrap());

        let pos_shared = Arc::clone(&self.current_pos);

        let file = File::open(&playable.path)
            .map_err(|_| EngineError::FileNotFound(playable.path.clone()))?;

        let reader = BufReader::with_capacity(256 * 1024, file);

        let source = Decoder::try_from(reader)
            .map_err(|e| EngineError::DecodeFailed(e.to_string()))?
            .track_position()
            .periodic_access(Duration::from_millis(200), move |s| {
                if let Ok(mut pos) = pos_shared.try_lock() {
                    *pos = s.get_pos();
                }
            });

        player.append(source);

        *self.player.lock().unwrap()   = Some(player);
        *self.state.lock().unwrap()    = PlaybackState::Playing;
        *self.duration.lock().unwrap() = playable.audio.duration_secs
            .map(Duration::from_secs);

        Ok(())
    }
    pub fn pause(&self) -> Result<(), EngineError> {
        let player = self.player.lock().unwrap();
        match player.as_ref() {
            Some(p) => {
                p.pause();
                *self.state.lock().unwrap() = PlaybackState::Paused;
                Ok(())
            }
            None => Err(EngineError::NoTrackLoaded),
        }
    }

    pub fn resume(&self) -> Result<(), EngineError> {
        let player = self.player.lock().unwrap();
        match player.as_ref() {
            Some(p) => {
                p.play();
                *self.state.lock().unwrap() = PlaybackState::Playing;
                Ok(())
            }
            None => Err(EngineError::NoTrackLoaded),
        }
    }

    pub fn stop(&self) {
        self.stop_player();
        *self.state.lock().unwrap()       = PlaybackState::Stopped;
        *self.current_pos.lock().unwrap() = Duration::ZERO;
    }

    // --- VOLUMEN --------------------------------------------------------------

    pub fn set_volume(&self, level: f32) {
        let level = level.clamp(0.0, 1.0);
        *self.volume.lock().unwrap() = level;

        if let Some(player) = self.player.lock().unwrap().as_ref() {
            player.set_volume(level);
        }
    }

    pub fn get_volume(&self) -> f32 {
        *self.volume.lock().unwrap()
    }

    // --- ESTADO ---------------------------------------------------------------

    pub fn state(&self) -> PlaybackState {
        self.state.lock().unwrap().clone()
    }

    /// Devuelve true si el player termino de reproducir (cancion acabo sola)
    pub fn is_finished(&self) -> bool {
        self.player
            .lock()
            .unwrap()
            .as_ref()
            .map(|p| p.empty())
            .unwrap_or(true)
    }

    /// Posición actual dentro del track, reportada por el decoder.
    pub fn position(&self) -> Duration {
        *self.current_pos.lock().unwrap()
    }

    /// Metodo para cambiar la posición de la canción
    pub fn seek(&self, position: Duration) -> Result<(), EngineError> {
        let player_guard = self.player.lock().unwrap();
        let player = player_guard.as_ref().ok_or(EngineError::NoTrackLoaded)?;

        player.try_seek(position)
            .map_err(|e| {EngineError::DecodeFailed(format!("Fallo al hacer seek: {}", e))})?;

        *self.current_pos.lock().unwrap() = position;

        Ok(())
    }

    /// Duracion total del track actual, extraida por symphonia al momento de encolar.
    pub fn duration(&self) -> Option<Duration> {
        *self.duration.lock().unwrap()
    }

    // --- INTERNO --------------------------------------------------------------

    fn stop_player(&self) {
        if let Some(player) = self.player.lock().unwrap().take() {
            player.stop();
        }
    }
}