use crate::models::app::{App, AppMode};
use crate::repositories::file_manager_repository;
use crate::services::calculator_service::CalculatorService;
use crate::views::app_view;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{backend::CrosstermBackend, prelude::*};
use std::io;

pub fn run(file_path: Option<String>) -> io::Result<()> {
    // Ensure terminal is restored even on panic
    std::panic::set_hook(Box::new(|panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        eprintln!("{}", panic_info);
    }));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let mut calculator = CalculatorService::new();

    // Load the file if provided as an argument
    if let Some(path) = file_path {
        if let Ok(lines) = file_manager_repository::load(&path) {
            app.lines = if lines.is_empty() {
                vec![String::new()]
            } else {
                lines
            };
            app.results = vec![String::new(); app.lines.len()];
            app.file_path = Some(path);
            recalculate(&mut app, &mut calculator);
        } else {
            // File doesn't exist yet, will be created on save
            app.file_path = Some(path);
        }
    }

    loop {
        // Adjust scroll offset based on cursor line and terminal height
        // The main area has height: frame.area().height - 1 (for status bar) - 2 (for borders)
        let main_height = terminal.size()?.height.saturating_sub(3) as usize;
        if app.cursor_line >= app.scroll_offset as usize + main_height {
            app.scroll_offset = (app.cursor_line - main_height + 1) as u16;
        } else if app.cursor_line < app.scroll_offset as usize {
            app.scroll_offset = app.cursor_line as u16;
        }

        terminal.draw(|frame| app_view::render(frame, &app))?;

        if let Event::Key(key) = event::read()? {
            // Clear a status message on the next user input if the timer is expired or doesn't exist
            if let Some(timer) = app.status_timer {
                if timer.elapsed().as_secs() >= 3 {
                    app.status_message = None;
                    app.status_timer = None;
                }
            } else if app.status_message.is_some() {
                app.status_message = None;
            }

            if key.code == KeyCode::Char('c') && key.modifiers.contains(event::KeyModifiers::CONTROL) {
                break;
            }

            match app.mode {
                AppMode::SavePrompt => {
                    match key.code {
                        KeyCode::Char(character) => {
                            app.save_input.push(character);
                        }
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
                                match file_manager_repository::save(&app.lines, &path) {
                                    Ok(_) => {
                                        app.file_path = Some(path);
                                        app.status_message = Some("Saved successfully!".to_string());
                                        app.status_timer = Some(std::time::Instant::now());
                                    }
                                    Err(e) => {
                                        app.status_message = Some(format!("Error saving: {}", e));
                                        app.status_timer = Some(std::time::Instant::now());
                                    }
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
                AppMode::Editing => {
                    match key.code {
                        KeyCode::Char('s') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                            match &app.file_path {
                                Some(path) => {
                                    let path = path.clone();
                                    match file_manager_repository::save(&app.lines, &path) {
                                        Ok(_) => {
                                            app.status_message = Some("Saved successfully!".to_string());
                                            app.status_timer = Some(std::time::Instant::now());
                                        }
                                        Err(e) => {
                                            app.status_message = Some(format!("Error saving: {}", e));
                                            app.status_timer = Some(std::time::Instant::now());
                                        }
                                    }
                                }
                                None => {
                                    app.mode = AppMode::SavePrompt;
                                    app.save_input = String::new();
                                }
                            }
                        }
                        KeyCode::Char(character) => {
                            let byte_index = app.lines[app.cursor_line]
                                .char_indices()
                                .nth(app.cursor_col)
                                .map(|(index, _)| index)
                                .unwrap_or(app.lines[app.cursor_line].len());
                            app.lines[app.cursor_line].insert(byte_index, character);
                            app.cursor_col += 1;
                            recalculate(&mut app, &mut calculator);
                        }
                        KeyCode::Backspace => {
                            if app.cursor_col > 0 {
                                let line_length = app.lines[app.cursor_line].chars().count();
                                let safe_col = app.cursor_col.min(line_length);
                                if let Some((byte_index, _)) = app.lines[app.cursor_line]
                                    .char_indices()
                                    .nth(safe_col - 1)
                                {
                                    app.lines[app.cursor_line].remove(byte_index);
                                    app.cursor_col = safe_col - 1;
                                    recalculate(&mut app, &mut calculator);
                                }
                            } else if app.cursor_line > 0 {
                                let current_line = app.lines.remove(app.cursor_line);
                                app.results.remove(app.cursor_line);
                                app.cursor_line -= 1;
                                app.cursor_col = app.lines[app.cursor_line].chars().count();
                                app.lines[app.cursor_line].push_str(&current_line);
                                recalculate(&mut app, &mut calculator);
                            }
                        }
                        KeyCode::Delete => {
                            let line_length = app.lines[app.cursor_line].chars().count();
                            if let Some((byte_index, _)) = (app.cursor_col < line_length)
                                .then(|| app.lines[app.cursor_line].char_indices().nth(app.cursor_col))
                                .flatten()
                            {
                                app.lines[app.cursor_line].remove(byte_index);
                                recalculate(&mut app, &mut calculator);
                            }
                        }
                        KeyCode::Enter => {
                            let next_line = app.cursor_line + 1;
                            app.lines.insert(next_line, String::new());
                            app.results.insert(next_line, String::new());
                            app.cursor_line = next_line;
                            app.cursor_col = 0;
                        }
                        KeyCode::Home => {
                            app.cursor_col = 0;
                        }
                        KeyCode::End => {
                            app.cursor_col = app.lines[app.cursor_line].chars().count();
                        }
                        KeyCode::Left => {
                            if app.cursor_col > 0 {
                                app.cursor_col -= 1;
                            } else if app.cursor_line > 0 {
                                app.cursor_line -= 1;
                                app.cursor_col = app.lines[app.cursor_line].len();
                            }
                        }
                        KeyCode::Right => {
                            let line_length = app.lines[app.cursor_line].chars().count();
                            if app.cursor_col < line_length {
                                app.cursor_col += 1;
                            } else if app.cursor_line < app.lines.len() - 1 {
                                app.cursor_line += 1;
                                app.cursor_col = 0;
                            }
                        }
                        KeyCode::Up => {
                            if app.cursor_line > 0 {
                                app.cursor_line -= 1;
                                app.cursor_col = app.cursor_col.min(app.lines[app.cursor_line].len());
                            }
                        }
                        KeyCode::Down => {
                            if app.cursor_line < app.lines.len() - 1 {
                                app.cursor_line += 1;
                                app.cursor_col = app.cursor_col.min(app.lines[app.cursor_line].len());
                            }
                        }
                        KeyCode::Esc => break,
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

/// Recalculates all lines from scratch whenever the input changes.
/// Variables are re-evaluated top to bottom so dependencies stay consistent.
fn recalculate(app: &mut App, calculator: &mut CalculatorService) {
    app.results = calculator.evaluate_document(&app.lines);
}