use crate::models::app::{App, AppMode};
use crate::repositories::file_manager_repository;
use crate::services::calculator_service::CalculatorService;
use crate::views::app_view;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseButton, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, prelude::*};
use std::io;
use std::time::Instant;

const RESULT_PANEL_WIDTH_PERCENT: f32 = 0.25;

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
        if !handle_event(
            &mut app,
            &mut calculator,
            &mut clipboard,
            &mut terminal,
            event,
        )? {
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

fn update_scroll(
    app: &mut App,
    terminal: &Terminal<CrosstermBackend<io::Stdout>>,
) -> io::Result<()> {
    let size = terminal.size()?;
    let main_height = size.height.saturating_sub(3) as usize;

    // Vertical scroll
    if app.cursor_line >= app.scroll_offset as usize + main_height {
        app.scroll_offset = app.cursor_line.saturating_sub(main_height - 1) as u16;
    } else if app.cursor_line < app.scroll_offset as usize {
        app.scroll_offset = app.cursor_line as u16;
    }

    // Horizontal scroll
    let result_panel_width = (size.width as f32 * RESULT_PANEL_WIDTH_PERCENT) as u16;
    let code_panel_width = size
        .width
        .saturating_sub(6)
        .saturating_sub(result_panel_width)
        .saturating_sub(2);

    if app.cursor_col >= app.scroll_x as usize + code_panel_width as usize {
        app.scroll_x = app.cursor_col.saturating_sub(code_panel_width as usize - 1) as u16;
    } else if app.cursor_col < app.scroll_x as usize {
        app.scroll_x = app.cursor_col as u16;
    }
    Ok(())
}

fn handle_event(
    app: &mut App,
    calculator: &mut CalculatorService,
    clipboard: &mut Option<arboard::Clipboard>,
    terminal: &Terminal<CrosstermBackend<io::Stdout>>,
    event: Event,
) -> io::Result<bool> {
    match event {
        Event::Mouse(mouse_event) => handle_mouse_event(app, clipboard, terminal, mouse_event)?,
        Event::Key(key_event) => return handle_key_event(app, calculator, terminal, key_event),
        _ => {}
    }
    Ok(true)
}

fn handle_mouse_event(
    app: &mut App,
    clipboard: &mut Option<arboard::Clipboard>,
    terminal: &Terminal<CrosstermBackend<io::Stdout>>,
    mouse_event: event::MouseEvent,
) -> io::Result<()> {
    let size = terminal.size()?;
    let main_height = size.height.saturating_sub(3) as u16;

    match mouse_event.kind {
        MouseEventKind::ScrollDown => {
            if (app.scroll_offset as usize + main_height as usize) < app.lines.len() {
                app.scroll_offset += 1;
            }
        }
        MouseEventKind::ScrollUp => {
            app.scroll_offset = app.scroll_offset.saturating_sub(1);
        }
        MouseEventKind::Down(MouseButton::Left) => {
            let y = mouse_event.row;
            let x = mouse_event.column;

            if y > 0 && y <= main_height {
                let target_line = (y.saturating_sub(1) + app.scroll_offset) as usize;
                if target_line < app.lines.len() {
                    let result_panel_width =
                        (size.width as f32 * RESULT_PANEL_WIDTH_PERCENT) as u16;
                    let result_start_x = size.width.saturating_sub(result_panel_width);

                    if x >= result_start_x {
                        copy_result_to_clipboard(app, clipboard, target_line);
                    } else {
                        app.cursor_line = target_line;
                        let target_col = (x.saturating_sub(7) + app.scroll_x) as usize;
                        app.cursor_col = target_col.min(app.get_current_line().chars().count());
                        app.clear_autocomplete();
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn copy_result_to_clipboard(
    app: &mut App,
    clipboard: &mut Option<arboard::Clipboard>,
    line_idx: usize,
) {
    let result_text = &app.results[line_idx];
    if !result_text.is_empty() && !result_text.starts_with("error") {
        let clean_result = result_text
            .strip_prefix("= ")
            .unwrap_or(result_text)
            .replace(".", "")
            .replace(",", ".");
        if let Some(cb) = clipboard {
            if cb.set_text(clean_result).is_ok() {
                app.status_message = Some("Result copied to clipboard!".to_string());
                app.status_timer = Some(Instant::now());
            }
        }
    }
}

fn handle_key_event(
    app: &mut App,
    calculator: &mut CalculatorService,
    terminal: &Terminal<CrosstermBackend<io::Stdout>>,
    key: event::KeyEvent,
) -> io::Result<bool> {
    // Status message cleanup
    if let Some(timer) = app.status_timer {
        if timer.elapsed().as_secs() >= 3 {
            app.status_message = None;
            app.status_timer = None;
        }
    } else if app.status_message.is_some() {
        app.status_message = None;
    }

    if key.code == KeyCode::Char('c') && key.modifiers.contains(event::KeyModifiers::CONTROL) {
        return Ok(false);
    }

    match app.mode {
        AppMode::SavePrompt => handle_save_prompt_keys(app, key),
        AppMode::Editing => handle_editing_keys(app, calculator, terminal, key),
    }
    Ok(true)
}

fn handle_save_prompt_keys(app: &mut App, key: event::KeyEvent) {
    match key.code {
        KeyCode::Char(c) => app.save_input.push(c),
        KeyCode::Backspace => {
            app.save_input.pop();
        }
        KeyCode::Enter => {
            if !app.save_input.is_empty() {
                let name = app.save_input.trim().to_string();
                let path = if name.ends_with(".cpad") {
                    name
                } else {
                    format!("{}.cpad", name)
                };
                if file_manager_repository::save(&app.lines, &path).is_ok() {
                    app.file_path = Some(path);
                    app.status_message = Some("Saved successfully!".to_string());
                    app.status_timer = Some(Instant::now());
                }
            }
            app.mode = AppMode::Editing;
            app.save_input = String::new();
        }
        KeyCode::Esc => {
            app.mode = AppMode::Editing;
            app.save_input = String::new();
        }
        _ => {}
    }
}

fn handle_editing_keys(
    app: &mut App,
    calculator: &mut CalculatorService,
    terminal: &Terminal<CrosstermBackend<io::Stdout>>,
    key: event::KeyEvent,
) {
    if !app.autocomplete_options.is_empty() && handle_autocomplete_keys(app, calculator, key) {
        return;
    }

    match key.code {
        KeyCode::Char('s') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
            if let Some(path) = app.file_path.clone() {
                if file_manager_repository::save(&app.lines, &path).is_ok() {
                    app.status_message = Some("Saved successfully!".to_string());
                    app.status_timer = Some(Instant::now());
                }
            } else {
                app.mode = AppMode::SavePrompt;
            }
        }
        KeyCode::Char(c) => {
            app.insert_char(c);
            app.results = calculator.evaluate_document(&app.lines);
            update_autocomplete(app, calculator);
        }
        KeyCode::Backspace => {
            if app.backspace() {
                app.results = calculator.evaluate_document(&app.lines);
                update_autocomplete(app, calculator);
            }
        }
        KeyCode::Delete => {
            if app.delete() {
                app.results = calculator.evaluate_document(&app.lines);
                update_autocomplete(app, calculator);
            }
        }
        KeyCode::Enter => {
            app.cursor_line += 1;
            app.lines.insert(app.cursor_line, String::new());
            app.results.insert(app.cursor_line, String::new());
            app.cursor_col = 0;
            app.clear_autocomplete();
        }
        KeyCode::Up => move_cursor_up(app),
        KeyCode::Down => move_cursor_down(app),
        KeyCode::Left => move_cursor_left(app),
        KeyCode::Right => move_cursor_right(app),
        KeyCode::Home => {
            app.cursor_col = 0;
            app.clear_autocomplete();
        }
        KeyCode::End => {
            app.cursor_col = app.get_current_line().chars().count();
            app.clear_autocomplete();
        }
        KeyCode::PageUp => {
            let jump = terminal
                .size()
                .map(|s| s.height.saturating_sub(3))
                .unwrap_or(10) as usize;
            move_page(app, -(jump as i32));
        }
        KeyCode::PageDown => {
            let jump = terminal
                .size()
                .map(|s| s.height.saturating_sub(3))
                .unwrap_or(10) as usize;
            move_page(app, jump as i32);
        }
        KeyCode::Esc => app.mode = AppMode::Editing,
        _ => {}
    }
}

fn handle_autocomplete_keys(
    app: &mut App,
    calculator: &mut CalculatorService,
    key: event::KeyEvent,
) -> bool {
    match key.code {
        KeyCode::Down => {
            let idx = app.autocomplete_index.unwrap_or(0);
            app.autocomplete_index = Some((idx + 1) % app.autocomplete_options.len());
            true
        }
        KeyCode::Up => {
            let idx = app.autocomplete_index.unwrap_or(0);
            app.autocomplete_index = Some(if idx == 0 {
                app.autocomplete_options.len() - 1
            } else {
                idx - 1
            });
            true
        }
        KeyCode::Tab | KeyCode::Enter => {
            if app.confirm_autocomplete() {
                app.results = calculator.evaluate_document(&app.lines);
            }
            true
        }
        KeyCode::Esc => {
            app.clear_autocomplete();
            true
        }
        _ => {
            app.clear_autocomplete();
            false
        }
    }
}

fn move_cursor_up(app: &mut App) {
    if app.cursor_line > 0 {
        app.cursor_line -= 1;
        app.cursor_col = app.cursor_col.min(app.get_current_line().chars().count());
    }
    app.clear_autocomplete();
}

fn move_cursor_down(app: &mut App) {
    if app.cursor_line < app.lines.len() - 1 {
        app.cursor_line += 1;
        app.cursor_col = app.cursor_col.min(app.get_current_line().chars().count());
    }
    app.clear_autocomplete();
}

fn move_cursor_left(app: &mut App) {
    if app.cursor_col > 0 {
        app.cursor_col -= 1;
    } else if app.cursor_line > 0 {
        app.cursor_line -= 1;
        app.cursor_col = app.get_current_line().chars().count();
    }
    app.clear_autocomplete();
}

fn move_cursor_right(app: &mut App) {
    if app.cursor_col < app.get_current_line().chars().count() {
        app.cursor_col += 1;
    } else if app.cursor_line < app.lines.len() - 1 {
        app.cursor_line += 1;
        app.cursor_col = 0;
    }
    app.clear_autocomplete();
}

fn move_page(app: &mut App, jump: i32) {
    if jump < 0 {
        app.cursor_line = app.cursor_line.saturating_sub((-jump) as usize);
    } else {
        app.cursor_line = (app.cursor_line + jump as usize).min(app.lines.len() - 1);
    }
    app.cursor_col = app.cursor_col.min(app.get_current_line().chars().count());
    app.clear_autocomplete();
}

fn update_autocomplete(app: &mut App, calculator: &CalculatorService) {
    app.clear_autocomplete();
    let (start, _) = app.get_current_word_bounds();

    if start < app.cursor_col {
        let prefix: String = app
            .get_current_line()
            .chars()
            .skip(start)
            .take(app.cursor_col - start)
            .collect();
        if !prefix.is_empty() && prefix.chars().next().unwrap().is_alphabetic() {
            let mut matches: Vec<String> = calculator
                .state
                .get_variable_names()
                .into_iter()
                .filter(|name| name.starts_with(&prefix) && name != &prefix)
                .collect();
            matches.sort();
            if !matches.is_empty() {
                app.autocomplete_options = matches;
                app.autocomplete_index = Some(0);
            }
        }
    }
}
