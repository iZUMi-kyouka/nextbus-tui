mod api;
mod app;
mod config;
mod i18n;
mod layout;
mod message;
mod models;
mod theme;
mod ui;

use std::io;
use std::sync::mpsc;
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use app::App;
use app::input::key_to_message;
use app::mouse::mouse_to_message;
use message::Message;
use models::AppEvent;

fn main() -> io::Result<()> {
    let mut terminal = setup_terminal()?;

    let (tx, rx) = mpsc::channel::<AppEvent>();
    let mut app = App::new(tx.clone());

    // Background tick thread — drives auto-refresh countdown and status expiry.
    let tick_tx = tx.clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_millis(500));
            if tick_tx.send(AppEvent::Tick).is_err() {
                break;
            }
        }
    });

    // Prime the detail panel immediately for the first visible stop.
    app.ensure_data();

    let result = run_loop(&mut terminal, &mut app, rx);

    restore_terminal(&mut terminal)?;
    result
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()
}

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
