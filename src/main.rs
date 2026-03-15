mod api;
mod app;
mod config;
mod models;
mod ui;

use std::io;
use std::sync::mpsc;
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;
use models::AppEvent;

fn main() -> io::Result<()> {
    // ── Terminal setup ────────────────────────────────────────────────────────
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // ── App + channels ────────────────────────────────────────────────────────
    let (tx, rx) = mpsc::channel::<AppEvent>();
    let mut app = App::new(tx.clone());

    // Background tick thread — drives auto-refresh countdown and status expiry
    let tick_tx = tx.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(500));
        if tick_tx.send(AppEvent::Tick).is_err() {
            break;
        }
    });

    // Prime the detail panel immediately for the first visible stop
    app.ensure_data();

    // ── Event loop ────────────────────────────────────────────────────────────
    let result = run_loop(&mut terminal, &mut app, rx);

    // ── Restore terminal ──────────────────────────────────────────────────────
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    rx: mpsc::Receiver<AppEvent>,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        // Poll for keyboard / mouse events (50 ms timeout keeps the UI responsive)
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => handle_key(app, key),
                Event::Mouse(_) => {} // mouse events reserved for future use
                _ => {}
            }
        }

        // Drain all pending background events
        while let Ok(ev) = rx.try_recv() {
            match ev {
                AppEvent::Tick => app.handle_tick(),
                AppEvent::DataReceived { stop_name, data } => app.handle_data(stop_name, data),
                AppEvent::FetchError { stop_name, error } => app.handle_error(stop_name, error),
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn handle_key(app: &mut App, key: event::KeyEvent) {
    if app.searching {
        // ── Search / filter mode ──────────────────────────────────────────────
        match key.code {
            KeyCode::Esc => {
                app.searching = false;
                app.search_query.clear();
                app.rebuild_list();
                app.ensure_data();
            }
            KeyCode::Enter => {
                app.searching = false;
                app.ensure_data();
            }
            KeyCode::Backspace => {
                app.search_query.pop();
                app.rebuild_list();
                app.ensure_data();
            }
            KeyCode::Char(c) => {
                app.search_query.push(c);
                app.rebuild_list();
                app.ensure_data();
            }
            // Allow navigation while typing
            KeyCode::Up => app.move_up(),
            KeyCode::Down => app.move_down(),
            _ => {}
        }
    } else {
        // ── Normal mode ───────────────────────────────────────────────────────
        match (key.code, key.modifiers) {
            (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                app.should_quit = true;
            }
            (KeyCode::Up, _) | (KeyCode::Char('k'), _) => app.move_up(),
            (KeyCode::Down, _) | (KeyCode::Char('j'), _) => app.move_down(),
            (KeyCode::Char('g'), KeyModifiers::NONE) => app.go_first(),
            (KeyCode::Char('G'), _) => app.go_last(),
            (KeyCode::Home, _) => app.go_first(),
            (KeyCode::End, _) => app.go_last(),
            (KeyCode::Char('f'), _) => app.toggle_favourite(),
            (KeyCode::Char('r'), _) => app.refresh_current(),
            (KeyCode::Char('/'), _) => {
                app.searching = true;
            }
            _ => {}
        }
    }
}
