use crate::controllers::event_handler::handle_event;
use crate::controllers::scroll_controller::update_scroll;
use crate::models::app::App;
use crate::repositories::file_manager_repository;
use crate::services::calculator_service::CalculatorService;
use crate::views::app_view;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;

pub fn run(file_path: Option<String>) -> io::Result<()> {
    std::panic::set_hook(Box::new(|panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        eprintln!("{}", panic_info);
    }));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let mut calculator = CalculatorService::new();
    let mut clipboard = arboard::Clipboard::new().ok();

    if let Some(path) = file_path {
        load_file(&mut app, &mut calculator, path);
    }

    loop {
        update_scroll(&mut app, &terminal)?;
        terminal.draw(|frame| app_view::render(frame, &app))?;

        let event = event::read()?;
        if !handle_event(&mut app, &mut calculator, &mut clipboard, &terminal, event)? {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}

fn load_file(app: &mut App, calculator: &mut CalculatorService, path: String) {
    if let Ok(lines) = file_manager_repository::load(&path) {
        app.lines = if lines.is_empty() {
            vec![String::new()]
        } else {
            lines
        };
        app.results = vec![String::new(); app.lines.len()];
        app.file_path = Some(path);
        app.results = calculator.evaluate_document(&app.lines);
    } else {
        app.file_path = Some(path);
    }
}
