// src/tui.rs
//
// Layout:
//   ┌─────────────────────────────┐
//   │  [ Search bar             ] │  <- Focus::Search  (Tab para cambiar)
//   ├─────────────────────────────┤
//   │  Now playing: titulo        │
//   │  Artist                     │
//   │  [████████░░░░░░] 1:23/3:45 │
//   │  ▶ Playing                  │
//   ├─────────────────────────────┤
//   │  Queue  (3 tracks)          │
//   │  > 0  titulo - artista      │  <- seleccionado
//   │    1  titulo - artista      │
//   └─────────────────────────────┘
//
// Teclas:
//   Tab          → alternar foco Search ↔ Queue
//   Search:  Enter → buscar y añadir | Backspace → borrar
//   Queue:   ↑/↓  → navegar | Enter → reproducir ahora
//   Global:  Space → pause/resume | s → skip | p → prev | q → quit

use std::io;
use std::time::Duration;
use std::sync::mpsc::Receiver;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use crate::model::Track;

// ─── PALETA ──────────────────────────────────────────────────────────────────

const C_BG:       Color = Color::Rgb(12,  12,  18);
const C_SURFACE:  Color = Color::Rgb(22,  22,  32);
const C_ACCENT:   Color = Color::Rgb(139, 92,  246);
const C_ACCENT2:  Color = Color::Rgb(236, 72,  153);
const C_TEXT:     Color = Color::Rgb(220, 220, 235);
const C_MUTED:    Color = Color::Rgb(100, 100, 130);
const C_SELECTED: Color = Color::Rgb(30,  30,  50);

// ─── ESTADO ──────────────────────────────────────────────────────────────────

#[derive(PartialEq)]
enum Focus { Search, Queue }
#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub current:       Option<Track>,
    pub queue:         Vec<Track>,
    pub is_playing:    bool,
    pub progress:      f64,
    pub elapsed_secs:  u64,
    pub duration_secs: u64,
    pub status_msg:    Option<String>,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            current:       None,
            queue:         Vec::new(),
            is_playing:    false,
            progress:      0.0,
            elapsed_secs:  0,
            duration_secs: 0,
            status_msg:    None,
        }
    }
}

// ─── TUI APP ─────────────────────────────────────────────────────────────────

pub struct TuiApp {
    focus:        Focus,
    search_input: String,
    list_state:   ListState,
    pub state:    PlayerState,
    tx:           UnboundedSender<crate::PlayerCmd>,
    state_rx:     Receiver<PlayerState>,
    status_msg:   Option<String>,
}

impl TuiApp {
    pub fn new(tx: UnboundedSender<crate::PlayerCmd>, state_rx: Receiver<PlayerState>) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            focus: Focus::Search,
            search_input: String::new(),
            list_state,
            state: PlayerState::default(),
            tx,
            state_rx,
            status_msg: None,
        }
    }

    // ─── RUN ─────────────────────────────────────────────────────────────────

    pub fn run(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend  = CrosstermBackend::new(stdout);
        let mut term = Terminal::new(backend)?;

        loop {
            // Aplicar todos los updates de estado pendientes antes de renderizar
            while let Ok(new_state) = self.state_rx.try_recv() {
                self.state = new_state;
            }

            term.draw(|f| self.render(f))?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind != KeyEventKind::Press { continue; }
                    if self.handle_key(key.code) { break; }
                }
            }
        }

        disable_raw_mode()?;
        execute!(term.backend_mut(), LeaveAlternateScreen)?;
        Ok(())
    }

    // ─── INPUT ───────────────────────────────────────────────────────────────

    /// Devuelve true si hay que salir.
    fn handle_key(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Tab => {
                self.focus = if self.focus == Focus::Search {
                    Focus::Queue
                } else {
                    Focus::Search
                };
                return false;
            }

            // Globales — solo cuando NO estamos escribiendo en el search
            KeyCode::Char('q') if self.focus == Focus::Queue => {
                let _ = self.tx.send(crate::PlayerCmd::TextCmd("quit".into()));
                return true;
            }
            KeyCode::Char(' ') if self.focus == Focus::Queue => {
                let cmd = if self.state.is_playing { "pause" } else { "resume" };
                let _ = self.tx.send(crate::PlayerCmd::TextCmd(cmd.into()));
                self.state.is_playing = !self.state.is_playing;
            }
            KeyCode::Char('s') if self.focus == Focus::Queue => {
                let _ = self.tx.send(crate::PlayerCmd::TextCmd("skip".into()));
                self.status_msg = Some("Skipped.".into());
            }
            KeyCode::Char('p') if self.focus == Focus::Queue => {
                let _ = self.tx.send(crate::PlayerCmd::TextCmd("prev".into()));
                self.status_msg = Some("Previous.".into());
            }

            _ => match self.focus {
                Focus::Search  => self.handle_search_key(key),
                Focus::Queue   => self.handle_queue_key(key),
            }
        }
        false
    }

    fn handle_search_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char(c) => self.search_input.push(c),
            KeyCode::Backspace => { self.search_input.pop(); }
            KeyCode::Enter => {
                let q = self.search_input.trim().to_string();
                if !q.is_empty() {
                    let _ = self.tx.send(crate::PlayerCmd::Search(q.clone()));
                    self.status_msg = Some(format!("Buscando: {}", q));
                    self.search_input.clear();
                }
            }
            _ => {}
        }
    }

    fn handle_queue_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Up => {
                let i = self.list_state.selected().unwrap_or(0);
                if i > 0 { self.list_state.select(Some(i - 1)); }
            }
            KeyCode::Down => {
                let len = self.state.queue.len();
                let i   = self.list_state.selected().unwrap_or(0);
                if i + 1 < len { self.list_state.select(Some(i + 1)); }
            }
            KeyCode::Enter => {
                if let Some(i) = self.list_state.selected() {
                    let _ = self.tx.send(crate::PlayerCmd::PlayAt(i));
                }
            }
            _ => {}
        }
    }

    // ─── RENDER ──────────────────────────────────────────────────────────────

    fn render(&mut self, f: &mut Frame) {
        let area = f.area();

        // Fondo
        f.render_widget(
            Block::default().style(Style::default().bg(C_BG)),
            area,
        );

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),  // search
                Constraint::Length(7),  // player
                Constraint::Min(5),     // queue
                Constraint::Length(1),  // status bar
            ])
            .split(area);

        self.render_search(f, chunks[0]);
        self.render_player(f, chunks[1]);
        self.render_queue(f,  chunks[2]);
        self.render_status(f, chunks[3]);
    }

    fn render_search(&self, f: &mut Frame, area: Rect) {
        let focused  = self.focus == Focus::Search;
        let border_c = if focused { C_ACCENT } else { C_MUTED };

        let prefix = Span::styled(" / ", Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD));
        let input  = Span::styled(
            format!("{}", self.search_input),
            Style::default().fg(C_TEXT),
        );
        let cursor = if focused {
            Span::styled("█", Style::default().fg(C_ACCENT2))
        } else {
            Span::raw("")
        };

        let para = Paragraph::new(Line::from(vec![prefix, input, cursor]))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_c))
                    .title(Span::styled(
                        " Buscar ",
                        Style::default().fg(border_c).add_modifier(Modifier::BOLD),
                    ))
                    .style(Style::default().bg(C_SURFACE)),
            );

        f.render_widget(para, area);
    }

    fn render_player(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // titulo
                Constraint::Length(1), // artista
                Constraint::Length(2), // barra progreso
                Constraint::Length(1), // estado
            ])
            .split(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(C_MUTED))
                .title(Span::styled(
                    " Ahora ",
                    Style::default().fg(C_MUTED).add_modifier(Modifier::BOLD),
                ))
                .inner(area));

        // Renderizar el bloque exterior
        f.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(C_MUTED))
                .title(Span::styled(
                    " Ahora ",
                    Style::default().fg(C_MUTED).add_modifier(Modifier::BOLD),
                ))
                .style(Style::default().bg(C_SURFACE)),
            area,
        );

        if let Some(track) = &self.state.current {
            // Titulo
            let title = Paragraph::new(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(&track.title, Style::default().fg(C_TEXT).add_modifier(Modifier::BOLD)),
            ]));

            // Artista
            let artist = Paragraph::new(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(&track.artist, Style::default().fg(C_MUTED)),
            ]));

            // Barra de progreso
            let elapsed  = format_duration(self.state.elapsed_secs);
            let duration = format_duration(self.state.duration_secs);
            let label    = format!(" {} / {} ", elapsed, duration);

            let gauge = Gauge::default()
                .gauge_style(
                    Style::default()
                        .fg(C_ACCENT)
                        .bg(C_SURFACE)
                )
                .ratio(self.state.progress.clamp(0.0, 1.0))
                .label(label);

            f.render_widget(title,  chunks[0]);
            f.render_widget(artist, chunks[1]);
            f.render_widget(gauge,  chunks[2]);

            // Estado
            let (icon, color) = if self.state.is_playing {
                ("▶  Playing", C_ACCENT)
            } else {
                ("⏸  Paused",  C_MUTED)
            };
            let status = Paragraph::new(
                Span::styled(format!("  {}", icon), Style::default().fg(color))
            );
            f.render_widget(status, chunks[3]);

        } else {
            let idle = Paragraph::new(
                Span::styled("  Nada sonando", Style::default().fg(C_MUTED))
            );
            f.render_widget(idle, chunks[0]);
        }
    }

    fn render_queue(&mut self, f: &mut Frame, area: Rect) {
        let focused  = self.focus == Focus::Queue;
        let border_c = if focused { C_ACCENT } else { C_MUTED };
        let title    = format!(" Cola  {} tracks ", self.state.queue.len());

        let max_width = area.width.saturating_sub(12) as usize; // 6 num + 2 borders + 4 margen

        let items: Vec<ListItem> = self.state.queue
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let num = Span::styled(
                    format!("  {:>2}  ", i),
                    Style::default().fg(C_MUTED),
                );
                let full  = format!("{} — {}", t.title, t.artist);
                let label = truncate(&full, max_width);
                let title = Span::styled(label, Style::default().fg(C_TEXT));
                ListItem::new(Line::from(vec![num, title]))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_c))
                    .title(Span::styled(
                        title,
                        Style::default().fg(border_c).add_modifier(Modifier::BOLD),
                    ))
                    .style(Style::default().bg(C_SURFACE)),
            )
            .highlight_style(
                Style::default()
                    .bg(C_SELECTED)
                    .fg(C_ACCENT2)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        f.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn render_status(&self, f: &mut Frame, area: Rect) {
        let hints = if self.focus == Focus::Search {
            " Tab: cola   Enter: buscar   (escribe con espacios libremente) "
        } else {
            " Tab: buscar   ↑↓: navegar   Enter: reproducir   Space: play/pause   s: skip   p: prev   q: salir "
        };

        // state.status_msg tiene prioridad (mensajes de descarga), luego hints
        let msg = self.state.status_msg.as_deref().unwrap_or(hints);

        let color = if self.state.status_msg.is_some() { C_ACCENT2 } else { C_MUTED };

        let para = Paragraph::new(Span::styled(msg, Style::default().fg(color)))
            .alignment(Alignment::Center);

        f.render_widget(para, area);
    }
}

// ─── HELPERS ─────────────────────────────────────────────────────────────────

fn format_duration(secs: u64) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let cut: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{}…", cut)
    }
}