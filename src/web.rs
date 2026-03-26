// POST /api/search          { "query": "..." }  → encola directo
// GET  /api/results?query=  → devuelve Vec<TrackDto> sin encolar
// GET  /api/current         → track actual + posición + estado
// GET  /api/queue           → cola pendiente
// POST /api/pause
// POST /api/resume
// POST /api/stop
// POST /api/skip
// POST /api/prev
// POST /api/play_at         { "index": n }
// POST /api/remove          { "index": n }
// POST /api/move            { "from": n, "to": n }
// POST /api/seek            { "secs": n }
// POST /api/volume          { "level": f32 }
// POST /api/repeat/toggle
// GET  /ws                  WebSocket push — recibe PlayerState JSON en cada cambio

use std::sync::Arc;

use axum::{
    Router,
    extract::{Query, State, WebSocketUpgrade, ws::{WebSocket, Message}},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json,
};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc, RwLock};
use crate::model::Track;
use crate::player_cmd::PlayerCmd;
use crate::TrackManager;
use crate::tui::PlayerState;

// --- Estado compartido del servidor ------------------------------------------

#[derive(Clone)]
pub struct AppState {
    pub cmd_tx:        mpsc::UnboundedSender<PlayerCmd>,
    pub state_tx:      broadcast::Sender<PlayerState>,
    pub track_manager: Arc<TrackManager>,
    pub last_state:    Arc<RwLock<PlayerState>>,
}

// --- DTOs --------------------------------------------------------------------

#[derive(Deserialize)] pub struct SearchBody  { pub query: String }
#[derive(Deserialize)] pub struct ResultQuery { pub query: String }
#[derive(Deserialize)] pub struct IndexBody   { pub index: usize }
#[derive(Deserialize)] pub struct MoveBody    { pub from: usize, pub to: usize }
#[derive(Deserialize)] pub struct SeekBody    { pub secs: u64 }
#[derive(Deserialize)] pub struct VolumeBody  { pub level: f32 }
#[derive(Serialize)]   pub struct OkResponse  { pub ok: bool }

#[derive(Serialize)]
pub struct TrackDto {
    pub id:        String,
    pub title:     String,
    pub artist:    String,
    pub thumbnail: Option<String>,
}

#[derive(Serialize)]
pub struct CurrentResponse {
    pub current:       Option<Track>,
    pub is_playing:    bool,
    pub elapsed_secs:  u64,
    pub duration_secs: u64,
    pub progress:      f64,
    pub on_repeat:     bool,
    pub volume:        f32,
}

fn ok() -> Json<OkResponse> { Json(OkResponse { ok: true }) }

// --- Router ------------------------------------------------------------------

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/search",        post(search_handler))
        .route("/api/results",       get(results_handler))
        .route("/api/current",       get(current_handler))
        .route("/api/queue",         get(queue_handler))
        .route("/api/pause",         post(|s: State<AppState>| text_cmd(s, "pause")))
        .route("/api/resume",        post(|s: State<AppState>| text_cmd(s, "resume")))
        .route("/api/stop",          post(|s: State<AppState>| text_cmd(s, "stop")))
        .route("/api/skip",          post(|s: State<AppState>| text_cmd(s, "skip")))
        .route("/api/prev",          post(|s: State<AppState>| text_cmd(s, "prev")))
        .route("/api/play_at",       post(play_at_handler))
        .route("/api/remove",        post(remove_handler))
        .route("/api/move",          post(move_handler))
        .route("/api/seek",          post(seek_handler))
        .route("/api/volume",        post(volume_handler))
        .route("/api/repeat/toggle", post(|s: State<AppState>| async move {
            let _ = s.cmd_tx.send(PlayerCmd::ToggleRepeat);
            ok()
        }))
        .route("/ws",                get(ws_handler))
        .with_state(state)
}

pub async fn serve(state: AppState) {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("API en http://0.0.0.0:3000");
    axum::serve(listener, build_router(state)).await.unwrap();
}

// --- Handlers REST -----------------------------------------------------------

async fn search_handler(
    State(s): State<AppState>,
    Json(b): Json<SearchBody>,
) -> impl IntoResponse {
    let _ = s.cmd_tx.send(PlayerCmd::Search(b.query));
    ok()
}

async fn results_handler(
    State(s): State<AppState>,
    Query(q): Query<ResultQuery>,
) -> impl IntoResponse {
    match s.track_manager.fetch_all_result(&q.query).await {
        Ok(tracks) => Json(tracks).into_response(),
        Err(e)     => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn current_handler(State(s): State<AppState>) -> impl IntoResponse {
    let state = s.last_state.read().await;
    Json(CurrentResponse {
        current:       state.current.clone(),
        is_playing:    state.is_playing,
        elapsed_secs:  state.elapsed_secs,
        duration_secs: state.duration_secs,
        progress:      state.progress,
        on_repeat:     state.on_repeat,
        volume:        state.volume,
    }).into_response()
}

async fn queue_handler(State(s): State<AppState>) -> impl IntoResponse {
    let state = s.last_state.read().await;
    Json(state.queue.clone()).into_response()
}

async fn text_cmd(State(s): State<AppState>, cmd: &'static str) -> impl IntoResponse {
    let _ = s.cmd_tx.send(PlayerCmd::TextCmd(cmd.to_string()));
    ok()
}

async fn play_at_handler(
    State(s): State<AppState>,
    Json(b): Json<IndexBody>,
) -> impl IntoResponse {
    let _ = s.cmd_tx.send(PlayerCmd::PlayAt(b.index));
    ok()
}

async fn remove_handler(
    State(s): State<AppState>,
    Json(b): Json<IndexBody>,
) -> impl IntoResponse {
    let _ = s.cmd_tx.send(PlayerCmd::RemoveAt(b.index));
    ok()
}

async fn move_handler(
    State(s): State<AppState>,
    Json(b): Json<MoveBody>,
) -> impl IntoResponse {
    let _ = s.cmd_tx.send(PlayerCmd::MoveTrack { from: b.from, to: b.to });
    ok()
}

async fn seek_handler(
    State(s): State<AppState>,
    Json(b): Json<SeekBody>,
) -> impl IntoResponse {
    let _ = s.cmd_tx.send(PlayerCmd::Seek(b.secs));
    ok()
}

async fn volume_handler(
    State(s): State<AppState>,
    Json(b): Json<VolumeBody>,
) -> impl IntoResponse {
    let _ = s.cmd_tx.send(PlayerCmd::SetVolume(b.level));
    ok()
}

// --- WebSocket ---------------------------------------------------------------

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(s): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| push_loop(socket, s.state_tx.subscribe()))
}

async fn push_loop(mut socket: WebSocket, mut rx: broadcast::Receiver<PlayerState>) {
    while let Ok(state) = rx.recv().await {
        let json = serde_json::to_string(&state).unwrap_or_default();
        if socket.send(Message::Text(json)).await.is_err() { break; }
    }
}