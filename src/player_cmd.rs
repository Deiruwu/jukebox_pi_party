use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};

use crate::managers::{QueueManager, TrackManager};
use crate::tui::PlayerState;

// --- Enum --------------------------------------------------------------------

pub enum PlayerCmd {
    Enqueue(crate::model::Track),
    Search(String),
    PlayAt(usize),
    PlayNext,
    TextCmd(String),
    StatusMsg(String),
    RemoveAt(usize),
    MoveTrack { from: usize, to: usize },
    Tick,
}

// --- Contexto que necesitan los handlers -------------------------------------

pub struct PlayerContext {
    pub queue:         QueueManager,
    pub track_manager: Arc<TrackManager>,
    pub cmd_tx:        mpsc::UnboundedSender<PlayerCmd>,
    pub tui_tx:        std::sync::mpsc::Sender<PlayerState>,
    pub ws_tx:         broadcast::Sender<PlayerState>,
    pub is_playing:    bool,
}

impl PlayerContext {
    /// Construye y envia un snapshot del estado actual a TUI y WS.
    pub fn push_state(&self, msg: Option<String>) {
        let elapsed  = self.queue.position().as_secs();
        let duration = self.queue.duration().map(|d| d.as_secs()).unwrap_or(0);
        let progress = if duration > 0 {
            elapsed as f64 / duration as f64
        } else {
            0.0
        };

        let state = PlayerState {
            current:       self.queue.current(),
            queue:         self.queue.list(),
            is_playing:    self.is_playing,
            progress,
            elapsed_secs:  elapsed,
            duration_secs: duration,
            status_msg:    msg,
        };
        let _ = self.tui_tx.send(state.clone());
        let _ = self.ws_tx.send(state);
    }
}

// --- Dispatcher --------------------------------------------------------------

pub async fn handle(cmd: PlayerCmd, ctx: &mut PlayerContext) {
    match cmd {
        PlayerCmd::Search(query)          => handle_search(query, ctx),
        PlayerCmd::Enqueue(track)         => handle_enqueue(track, ctx),
        PlayerCmd::PlayAt(index)          => handle_play_at(index, ctx),
        PlayerCmd::PlayNext               => handle_play_next(ctx),
        PlayerCmd::RemoveAt(index)        => handle_remove(index, ctx),
        PlayerCmd::MoveTrack { from, to } => handle_move(from, to, ctx),
        PlayerCmd::StatusMsg(msg)         => ctx.push_state(Some(msg)),
        PlayerCmd::TextCmd(cmd)           => handle_text_cmd(&cmd, ctx),
        PlayerCmd::Tick                   => { if ctx.is_playing { ctx.push_state(None); } }
    }
}

// --- Handlers individuales ---------------------------------------------------

fn handle_search(query: String, ctx: &PlayerContext) {
    let tm  = Arc::clone(&ctx.track_manager);
    let tx  = ctx.cmd_tx.clone();
    tokio::spawn(async move {
        let _ = tx.send(PlayerCmd::StatusMsg(format!("Buscando: {}", query)));
        let tx2 = tx.clone();
        match tm.resolve_with_status(&query, move |msg| {
            let _ = tx2.send(PlayerCmd::StatusMsg(msg));
        }).await {
            Ok(track) => { let _ = tx.send(PlayerCmd::Enqueue(track)); }
            Err(e)    => { let _ = tx.send(PlayerCmd::StatusMsg(format!("Error: {}", e))); }
        }
    });
}

fn handle_enqueue(track: crate::model::Track, ctx: &mut PlayerContext) {
    if let Err(e) = ctx.queue.enqueue(track) {
        eprintln!("Error al encolar: {}", e);
        return;
    }
    if !ctx.is_playing { ctx.is_playing = true; }
    ctx.push_state(None);
}

fn handle_play_at(index: usize, ctx: &mut PlayerContext) {
    if ctx.queue.move_track(index, 0).is_err() { return; }
    ctx.queue.stop();
    if ctx.queue.next().is_ok() {
        ctx.is_playing = true;
        ctx.push_state(None);
    }
}

fn handle_play_next(ctx: &mut PlayerContext) {
    if !ctx.queue.is_finished() || ctx.queue.current().is_none() { return; }
    match ctx.queue.next() {
        Ok(_)  => { ctx.is_playing = true;  ctx.push_state(None); }
        Err(_) => { ctx.is_playing = false; ctx.push_state(None); }
    }
}

fn handle_remove(index: usize, ctx: &mut PlayerContext) {
    let _ = ctx.queue.remove(index);
    ctx.push_state(None);
}

fn handle_move(from: usize, to: usize, ctx: &mut PlayerContext) {
    let _ = ctx.queue.move_track(from, to);
    ctx.push_state(None);
}

fn handle_text_cmd(cmd: &str, ctx: &mut PlayerContext) {
    match cmd {
        "pause"         => { ctx.queue.pause();  ctx.is_playing = false; ctx.push_state(None); }
        "resume"        => { ctx.queue.resume(); ctx.is_playing = true;  ctx.push_state(None); }
        "stop"          => { ctx.queue.stop();   ctx.is_playing = false; ctx.push_state(None); }
        "skip"          => { let _ = ctx.queue.skip(); ctx.push_state(None); }
        "prev"          => { let _ = ctx.queue.prev(); ctx.push_state(None); }
        "quit" | "exit" => { ctx.queue.stop(); std::process::exit(0); }
        _ => {}
    }
}