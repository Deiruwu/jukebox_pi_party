// src/web.rs
//
// API REST + WebSocket. Sin frontend — el cliente lo pone tu amigo.
//
// POST /api/search          { "query": "..." }
// POST /api/pause
// POST /api/resume
// POST /api/stop
// POST /api/skip
// POST /api/prev
// POST /api/play_at         { "index": n }
// POST /api/remove          { "index": n }
// POST /api/move            { "from": n, "to": n }
// GET  /ws                  WebSocket push — recibe PlayerState JSON en cada cambio

use axum::{
    Router,
    extract::{State, WebSocketUpgrade, ws::{WebSocket, Message}},
    response::IntoResponse,
    routing::{get, post},
    Json,
};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc};

use crate::player_cmd::PlayerCmd;
use crate::tui::PlayerState;

// ─── Estado compartido del servidor ──────────────────────────────────────────

#[derive(Clone)]
pub struct AppState {
    pub cmd_tx:   mpsc::UnboundedSender<PlayerCmd>,
    pub state_tx: broadcast::Sender<PlayerState>,
}

// ─── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Deserialize)] pub struct SearchBody { pub query: String }
#[derive(Deserialize)] pub struct IndexBody  { pub index: usize }
#[derive(Deserialize)] pub struct MoveBody   { pub from: usize, pub to: usize }
#[derive(Serialize)]   pub struct OkResponse { pub ok: bool }

fn ok() -> Json<OkResponse> { Json(OkResponse { ok: true }) }

// ─── Router ───────────────────────────────────────────────────────────────────

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/search",  post(search_handler))
        .route("/api/pause",   post(|s: State<AppState>| text_cmd(s, "pause")))
        .route("/api/resume",  post(|s: State<AppState>| text_cmd(s, "resume")))
        .route("/api/stop",    post(|s: State<AppState>| text_cmd(s, "stop")))
        .route("/api/skip",    post(|s: State<AppState>| text_cmd(s, "skip")))
        .route("/api/prev",    post(|s: State<AppState>| text_cmd(s, "prev")))
        .route("/api/play_at", post(play_at_handler))
        .route("/api/remove",  post(remove_handler))
        .route("/api/move",    post(move_handler))
        .route("/ws",          get(ws_handler))
        .with_state(state)
}

pub async fn serve(state: AppState) {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🎵  API en http://0.0.0.0:3000");
    axum::serve(listener, build_router(state)).await.unwrap();
}

// ─── Handlers REST ────────────────────────────────────────────────────────────

async fn search_handler(
    State(s): State<AppState>,
    Json(b): Json<SearchBody>,
) -> impl IntoResponse {
    let _ = s.cmd_tx.send(PlayerCmd::Search(b.query));
    ok()
}

async fn text_cmd(State(s): State<AppState>, cmd: &'static str) -> impl IntoResponse {
    let _ = s.cmd_tx.send(PlayerCmd::TextCmd(cmd.to_string()));
    ok()
}

async fn play_at_handler(State(s): State<AppState>, Json(b): Json<IndexBody>) -> impl IntoResponse {
    let _ = s.cmd_tx.send(PlayerCmd::PlayAt(b.index));
    ok()
}

async fn remove_handler(State(s): State<AppState>, Json(b): Json<IndexBody>) -> impl IntoResponse {
    let _ = s.cmd_tx.send(PlayerCmd::RemoveAt(b.index));
    ok()
}

async fn move_handler(State(s): State<AppState>, Json(b): Json<MoveBody>) -> impl IntoResponse {
    let _ = s.cmd_tx.send(PlayerCmd::MoveTrack { from: b.from, to: b.to });
    ok()
}

// ─── WebSocket ────────────────────────────────────────────────────────────────

async fn ws_handler(ws: WebSocketUpgrade, State(s): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| push_loop(socket, s.state_tx.subscribe()))
}

async fn push_loop(mut socket: WebSocket, mut rx: broadcast::Receiver<PlayerState>) {
    while let Ok(state) = rx.recv().await {
        let json = serde_json::to_string(&state).unwrap_or_default();
        if socket.send(Message::Text(json)).await.is_err() { break; }
    }
}