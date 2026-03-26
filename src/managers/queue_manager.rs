// src/managers/queue_manager.rs

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use crate::audio::{AudioEngine, probe_file};
use crate::model::Track;

#[derive(Debug)]
pub enum QueueError {
    EmptyQueue,
    InvalidIndex(usize),
    NoPath(String),
    DecodeError(String),
    EngineError(String),
}

impl std::fmt::Display for QueueError {
    /**
     *  Implementamos Display en QueueError para poder visualizar
     *  el tipo de error y su descripción si es que la posee
     */
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            QueueError::EmptyQueue                => write!(f, "La queue está vacía"),
            QueueError::InvalidIndex(i)    => write!(f, "Índice inválido: {}", i),
            QueueError::NoPath(id)         => write!(f, "Track sin path descargado: {}", id),
            QueueError::DecodeError(e)     => write!(f, "Error de decodificación: {}", e),
            QueueError::EngineError(e)     => write!(f, "Error de engine: {}", e),
        }
    }
}

impl std::error::Error for QueueError {}
/**
 *  Implementación del struct QueState.
 *  Contiene Queue: un VecDeque de Tracks qué sirve para hacer inserciones eficientes O(1),
 *  history: un Vec de Tracks para almacenar canciones que ya fueron reproducidas y salieron de cola,
 *  current: un Option qué contiene un Track, representa la canción Actual, no estar vacío el Option, inferimos que no hay nada sonando.
 */

#[derive(Debug, Clone)]
struct QueueState {
    queue:   VecDeque<Track>,
    history: Vec<Track>,
    current: Option<Track>,
    on_repeat_track: bool,
}

/**
 *  Implementamos un constructor vacío para inicializar el sistema.
 */
impl QueueState {
    fn new() -> Self {
        Self {
            queue:   VecDeque::new(),
            history: Vec::new(),
            current: None,
            on_repeat_track: false,
        }
    }
}

/**
 *  ======================
 *      QUEUE MANAGER
 *  ======================
 */

/**
 * Struct de QueueManager, donde contiene la struct AudioEngine
 * el cual se encarga de gestionar la comunicación con ALSA/PipeWire
 * y un Arc, Mutex de QueueState para garantizar una clonación entre hilos
 * y que un solo un hilo sea capaz de modificarlo a la vez.
 */
pub struct QueueManager {
    state:  Arc<Mutex<QueueState>>,
    engine: AudioEngine,
}

impl QueueManager {
    pub fn new(engine: AudioEngine) -> Self {
        Self {
            state:  Arc::new(Mutex::new(QueueState::new())),
            engine,
        }
    }

    /**
     *  Enqueue, función qué añade una canción al final de la queue,
     *  si no hay nada sonando, se reproduce automáticamente.
     *  Sistema FIFO
    */

    pub fn enqueue(&self, track: Track) -> Result<(), QueueError> {
        let should_play = {
            let mut s = self.state.lock().unwrap();
            s.queue.push_back(track);
            s.current.is_none() && self.engine.is_finished() == true
        };

        if should_play {
            self.next()?;
        }

        Ok(())
    }

    /**
     *  Play next, función que permite insertar una canción en el índice 0 del Vec queue
     *  permitiendo implementar un "Reproducir siguiente"
     */
    pub fn play_next_track(&self, track: Track) {
        self.state.lock().unwrap().queue.push_front(track);
    }

    /**
     *  Función utilizada para hacer un pop en el Vec queue y reproducir la canción al instante.
     *  Permite gestionar también el proceso natural las canciones, ejecutándose al terminar una canción
     *  también se encarga de mover la canción que estaba sonando al historial.
     */

    pub fn next(&self) -> Result<(), QueueError> {
        let repeat = self.state.lock().unwrap().on_repeat_track;

        if repeat {
            return self.engine.seek(Duration::ZERO)
                .map_err(|e| QueueError::EngineError(e.to_string()));
        }

        let track = {
            let mut s = self.state.lock().unwrap();
            if let Some(prev) = s.current.take() {
                s.history.push(prev);
            }
            s.queue.pop_front().ok_or(QueueError::EmptyQueue)?
        };

        self.play_track(track)
    }

    /**
     *  Función para reencolar la última canción añadida en el historial.
     */
    pub fn prev(&self) -> Result<(), QueueError> {
        let (prev, current) = {
            let mut s = self.state.lock().unwrap();

            let prev = s.history.pop().ok_or(QueueError::EmptyQueue)?;

            // Reencolar el actual al frente
            if let Some(current) = s.current.take() {
                s.queue.push_front(current);
            }

            let prev_clone = prev.clone();
            s.current = Some(prev);
            (prev_clone, ())
        };

        let _ = current;
        self.play_track(prev)
    }

    pub fn pause(&self) {
        let _ = self.engine.pause();
    }

    pub fn resume(&self) {
        let _ = self.engine.resume();
    }

    pub fn skip(&self) -> Result<(), QueueError> {
        self.state.lock().unwrap().on_repeat_track = false;
        self.engine.stop();
        self.next()
    }

    pub fn stop(&self) {
        self.engine.stop();
        let mut s = self.state.lock().unwrap();
        s.current = None;
    }

    pub fn set_on_repeat(&self, enabled: bool) {
        self.state.lock().unwrap().on_repeat_track = enabled;
    }

    pub fn toggle_repeat(&self) {
        let mut s = self.state.lock().unwrap();
        s.on_repeat_track = !s.on_repeat_track;
    }

   /**
    * ===================================
    * FUNCIONES DE MANIPULACIÓN DE QUEUE
    * ===================================
   */

    /**
     *  Función capaz de eliminar un elemento de la posición n
     *  (0-indexed sobre la queue pendiente).
     */
    pub fn remove(&self, index: usize) -> Result<Track, QueueError> {
        let mut s = self.state.lock().unwrap();
        if index >= s.queue.len() {
            return Err(QueueError::InvalidIndex(index));
        }
        Ok(s.queue.remove(index).unwrap())
    }

    /**
     *  Función utilizada para mover un elemento from hasta to
     */
    pub fn move_track(&self, from: usize, to: usize) -> Result<(), QueueError> {
        let mut s = self.state.lock().unwrap();
        let len = s.queue.len();

        if from >= len || to >= len {
            return Err(QueueError::InvalidIndex(from.max(to)));
        }

        let track = s.queue.remove(from).unwrap();
        s.queue.insert(to, track);

        Ok(())
    }

    /**
     *  ===============================
     *  GESTION DE ESTADOS DE LA QUEUE
     *  ===============================
     */

    pub fn current(&self) -> Option<Track> {
        self.state.lock().unwrap().current.clone()
    }

    pub fn list(&self) -> Vec<Track> {
        self.state.lock().unwrap().queue.iter().cloned().collect()
    }

    pub fn is_finished(&self) -> bool {
        self.engine.is_finished()
    }

    pub fn history(&self) -> Vec<Track> {
        self.state.lock().unwrap().history.clone()
    }

    pub fn current_track(&self) -> Option<Track> {
        self.state.lock().unwrap().current.clone()
    }

    pub fn position(&self) -> Duration {
        self.engine.position()
    }

    pub fn duration(&self) -> Option<Duration> {
        self.engine.duration()
    }

    pub fn set_volume(&self, level: f32) {
        self.engine.set_volume(level);
    }

    pub fn get_volume(&self) -> f32 {
        self.engine.get_volume()
    }

    pub fn seek(&self, position: Duration) -> Result<(), QueueError> {
        self.engine.seek(position)
            .map_err(|e| QueueError::EngineError(e.to_string()))
    }

    /**
     *  Función interna del sistema qué se encarga de gestionar la reproducción
     *  de las canciones utilizando las funciones del servicio engine y encoder
    */

    fn play_track(&self, track: Track) -> Result<(), QueueError> {
        let path = track.path.as_deref()
            .ok_or_else(|| QueueError::NoPath(track.id.clone()))?;

        let playable = probe_file(path, track.clone())
            .map_err(|e| QueueError::DecodeError(e.to_string()))?;

        self.engine.play(&playable)
            .map_err(|e| QueueError::EngineError(e.to_string()))?;

        self.state.lock().unwrap().current = Some(track);

        Ok(())
    }
}