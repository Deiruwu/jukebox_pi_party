mod services;
mod model;
mod audio;
mod repository;
mod managers;
mod tui;
mod web;
mod player_cmd;

use std::error::Error;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};

use crate::audio::AudioEngine;
use crate::managers::{QueueManager, TrackManager};
use crate::player_cmd::{PlayerContext, PlayerCmd, handle};
use crate::repository::{Database, TrackRepository};
use crate::services::{DownloadService, MetadataClient, PythonMicroservice};
use crate::tui::{PlayerState, TuiApp};
use crate::web::{AppState, serve};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    // ── Servicios ─────────────────────────────────────────────────────────────
    let py = PythonMicroservice::new("MetadataServices/.venv/", "MetadataServices/hub.py");
    py.spawn_service().await?;

    let db            = Database::connect("sqlite://./music.db?mode=rwc").await?;
    let repo          = TrackRepository::new(db.pool.clone());
    let metadata      = MetadataClient::new("127.0.0.1", 5005);
    let downloader    = DownloadService::new(".cache");
    let track_manager = Arc::new(TrackManager::new(metadata, repo, downloader));

    // ── Audio ─────────────────────────────────────────────────────────────────
    let engine = AudioEngine::new()?;
    let queue  = QueueManager::new(engine);

    // ── Channels ──────────────────────────────────────────────────────────────
    let (cmd_tx, mut cmd_rx)   = mpsc::unbounded_channel::<PlayerCmd>();
    let (tui_tx, tui_rx)       = std::sync::mpsc::channel::<PlayerState>();
    let (ws_tx, _)             = broadcast::channel::<PlayerState>(32);

    // ── Watcher: avanza track automáticamente ─────────────────────────────────
    spawn_watcher(cmd_tx.clone());

    // ── TUI en su thread ──────────────────────────────────────────────────────
    spawn_tui(cmd_tx.clone(), tui_rx);

    // ── API web ───────────────────────────────────────────────────────────────
    tokio::spawn(serve(AppState {
        cmd_tx:   cmd_tx.clone(),
        state_tx: ws_tx.clone(),
    }));

    // ── Main loop (delega todo a player_cmd::handle) ──────────────────────────
    let mut ctx = PlayerContext {
        queue,
        track_manager,
        cmd_tx,
        tui_tx,
        ws_tx,
        is_playing: false,
    };

    while let Some(cmd) = cmd_rx.recv().await {
        handle(cmd, &mut ctx).await;
    }

    Ok(())
}

// ─── Helpers de arranque ──────────────────────────────────────────────────────

fn spawn_watcher(tx: mpsc::UnboundedSender<PlayerCmd>) {
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(500));
        let _ = tx.send(PlayerCmd::PlayNext);
    });
}

fn spawn_tui(tx: mpsc::UnboundedSender<PlayerCmd>, rx: std::sync::mpsc::Receiver<PlayerState>) {
    std::thread::spawn(move || {
        let mut app = TuiApp::new(tx, rx);
        if let Err(e) = app.run() {
            eprintln!("TUI error: {}", e);
        }
    });
}