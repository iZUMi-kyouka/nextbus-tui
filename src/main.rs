mod api;
mod app;
mod config;
mod i18n;
mod layout;
mod message;
mod models;
mod theme;
mod time;
mod ui;
#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(not(target_arch = "wasm32"))]
use std::io;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::mpsc;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use crossterm::{
    event::{
        self, DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture, Event,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
#[cfg(not(target_arch = "wasm32"))]
use ratatui::{backend::CrosstermBackend, Terminal};

#[cfg(not(target_arch = "wasm32"))]
use app::input::key_to_message;
#[cfg(not(target_arch = "wasm32"))]
use app::mouse::mouse_to_message;
#[cfg(not(target_arch = "wasm32"))]
use app::App;
#[cfg(not(target_arch = "wasm32"))]
use message::Message;
#[cfg(not(target_arch = "wasm32"))]
use models::AppEvent;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> io::Result<()> {
    // Load .env if present; silently ignore if it doesn't exist.
    let _ = dotenvy::dotenv();

    let mut terminal = setup_terminal()?;

    let (tx, rx) = mpsc::channel::<AppEvent>();
    let mut app = App::new(tx.clone());

    // Background tick thread — drives auto-refresh countdown and status expiry.
    let tick_tx = tx.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(500));
        if tick_tx.send(AppEvent::Tick).is_err() {
            break;
        }
    });

    // Prime the detail panel immediately for the first visible stop.
    app.ensure_data();

    let result = run_loop(&mut terminal, &mut app, rx);

    restore_terminal(&mut terminal)?;
    result
}

#[cfg(target_arch = "wasm32")]
fn main() {
    web::start();
}

#[cfg(not(target_arch = "wasm32"))]
fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableFocusChange
    )?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

#[cfg(not(target_arch = "wasm32"))]
fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        DisableFocusChange
    )?;
    terminal.show_cursor()
}

#[cfg(not(target_arch = "wasm32"))]
fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    rx: mpsc::Receiver<AppEvent>,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        // Poll for keyboard / mouse events (50 ms timeout keeps the UI responsive).
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    if let Some(msg) = key_to_message(key, app) {
                        app.update(msg);
                    }
                }
                Event::Mouse(mouse) => {
                    let size = terminal.size()?;
                    if let Some(msg) = mouse_to_message(mouse, app, size.width, size.height) {
                        app.update(msg);
                    }
                }
                Event::FocusGained => app.update(Message::FocusGained),
                Event::FocusLost => app.update(Message::FocusLost),
                _ => {}
            }
        }

        // Drain all pending background events.
        while let Ok(ev) = rx.try_recv() {
            app.update(Message::from(ev));
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
