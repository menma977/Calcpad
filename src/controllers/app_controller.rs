use crate::models::app::{App, AppMode};
use crate::repositories::file_manager_repository;
use crate::services::calculator_service::CalculatorService;
use crate::views::app_view;
use crossterm::{
    event::{self, Event, KeyCode, MouseEventKind, MouseButton, EnableMouseCapture, DisableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{backend::CrosstermBackend, prelude::*};
use std::io;

pub fn run(file_path: Option<String>) -> io::Result<()> {
    // Ensure terminal is restored even on panic
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
        let main_height = terminal.size()?.height.saturating_sub(3) as usize;
        if app.cursor_line >= app.scroll_offset as usize + main_height {
            app.scroll_offset = (app.cursor_line - main_height + 1) as u16;
        } else if app.cursor_line < app.scroll_offset as usize {
            app.scroll_offset = app.cursor_line as u16;
        }

        // Adjust horizontal scroll offset
        let width = terminal.size()?.width;
        let result_panel_width = (width as f32 * 0.25) as u16;
        let code_panel_width = width.saturating_sub(6).saturating_sub(result_panel_width).saturating_sub(2);
        
        if app.cursor_col >= app.scroll_x as usize + code_panel_width as usize {
            app.scroll_x = (app.cursor_col - code_panel_width as usize + 1) as u16;
        } else if app.cursor_col < app.scroll_x as usize {
            app.scroll_x = app.cursor_col as u16;
        }

        terminal.draw(|frame| app_view::render(frame, &app))?;

        let event = event::read()?;

        if let Event::Mouse(mouse_event) = &event {
            match mouse_event.kind {
                MouseEventKind::ScrollDown => {
                    if app.scroll_offset as usize + main_height < app.lines.len() {
                        app.scroll_offset += 1;
                    }
                }
                MouseEventKind::ScrollUp => {
                    if app.scroll_offset > 0 {
                        app.scroll_offset -= 1;
                    }
                }
                MouseEventKind::Down(MouseButton::Left) => {
                    let y = mouse_event.row;
                    let x = mouse_event.column;
                    
                    if y > 0 && y <= main_height as u16 {
                        let target_line = (y.saturating_sub(1) + app.scroll_offset) as usize;
                        if target_line < app.lines.len() {
                            let width = terminal.size()?.width;
                            let result_panel_width = (width as f32 * 0.25) as u16;
                            let result_start_x = width.saturating_sub(result_panel_width);

                            // Check if click is in the result panel
                            if x >= result_start_x {
                                let result_text = &app.results[target_line];
                                if !result_text.is_empty() && !result_text.starts_with("error") {
                                    // Strip formatting dots to get raw number, but keep comma as decimal point if we want to copy the raw number.
                                    // For best interoperability, we'll copy the raw number with standard English format (no thousands, dot for decimal)
                                    // Or we can just copy exactly what is shown without the "="
                                    let clean_result = result_text.trim_start_matches("= ").replace(".", "").replace(",", ".");
                                    if let Some(cb) = &mut clipboard {
                                        let _ = cb.set_text(clean_result);
                                        app.status_message = Some("Result copied to clipboard!".to_string());
                                        app.status_timer = Some(std::time::Instant::now());
                                    }
                                }
                            } else {
                                app.cursor_line = target_line;
                                if x >= 7 {
                                    let target_col = (x.saturating_sub(7) + app.scroll_x) as usize;
                                    app.cursor_col = target_col.min(app.lines[app.cursor_line].chars().count());
                                } else {
                                    app.cursor_col = 0;
                                }
                                app.autocomplete_options.clear();
                                app.autocomplete_index = None;
                            }
                        }
                    }
                }
                _ => {}
            }
            continue;
        }

        if let Event::Key(key) = event {
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
                    // Check autocomplete first
                    if !app.autocomplete_options.is_empty() {
                        match key.code {
                            KeyCode::Down => {
                                let idx = app.autocomplete_index.unwrap_or(0);
                                app.autocomplete_index = Some((idx + 1) % app.autocomplete_options.len());
                                continue;
                            }
                            KeyCode::Up => {
                                let idx = app.autocomplete_index.unwrap_or(0);
                                app.autocomplete_index = Some(if idx == 0 { app.autocomplete_options.len() - 1 } else { idx - 1 });
                                continue;
                            }
                            KeyCode::Tab | KeyCode::Enter => {
                                if let Some(idx) = app.autocomplete_index {
                                    let word = &app.autocomplete_options[idx];
                                    let current_line = &app.lines[app.cursor_line];
                                    let (start, _end) = get_current_word_bounds(current_line, app.cursor_col);
                                    
                                    // Convert character indices to byte indices safely
                                    let byte_start = current_line.char_indices().nth(start).map(|(i, _)| i).unwrap_or(current_line.len());
                                    let byte_cursor = current_line.char_indices().nth(app.cursor_col).map(|(i, _)| i).unwrap_or(current_line.len());

                                    let mut new_line = String::new();
                                    new_line.push_str(&current_line[..byte_start]);
                                    new_line.push_str(word);
                                    
                                    // Replace up to cursor_col so user keeps typing after inserted word
                                    let remaining = &current_line[byte_cursor..];
                                    new_line.push_str(remaining);
                                    
                                    app.lines[app.cursor_line] = new_line;
                                    app.cursor_col = start + word.len();
                                    recalculate(&mut app, &mut calculator);
                                }
                                app.autocomplete_options.clear();
                                app.autocomplete_index = None;
                                continue;
                            }
                            KeyCode::Esc => {
                                app.autocomplete_options.clear();
                                app.autocomplete_index = None;
                                continue;
                            }
                            _ => {
                                app.autocomplete_options.clear();
                                app.autocomplete_index = None;
                            }
                        }
                    }

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
                            update_autocomplete(&mut app, &calculator);
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
                                    update_autocomplete(&mut app, &calculator);
                                }
                            } else if app.cursor_line > 0 {
                                let current_line = app.lines.remove(app.cursor_line);
                                app.results.remove(app.cursor_line);
                                app.cursor_line -= 1;
                                app.cursor_col = app.lines[app.cursor_line].chars().count();
                                app.lines[app.cursor_line].push_str(&current_line);
                                recalculate(&mut app, &mut calculator);
                                app.autocomplete_options.clear();
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
                                update_autocomplete(&mut app, &calculator);
                            }
                        }
                        KeyCode::Enter => {
                            let next_line = app.cursor_line + 1;
                            app.lines.insert(next_line, String::new());
                            app.results.insert(next_line, String::new());
                            app.cursor_line = next_line;
                            app.cursor_col = 0;
                            app.autocomplete_options.clear();
                        }
                        KeyCode::Home => {
                            app.cursor_col = 0;
                            app.autocomplete_options.clear();
                        }
                        KeyCode::End => {
                            app.cursor_col = app.lines[app.cursor_line].chars().count();
                            app.autocomplete_options.clear();
                        }
                        KeyCode::PageUp => {
                            let main_height = terminal.size()?.height.saturating_sub(3) as usize;
                            app.cursor_line = app.cursor_line.saturating_sub(main_height);
                            app.cursor_col = app.cursor_col.min(app.lines[app.cursor_line].len());
                            app.autocomplete_options.clear();
                        }
                        KeyCode::PageDown => {
                            let main_height = terminal.size()?.height.saturating_sub(3) as usize;
                            app.cursor_line = (app.cursor_line + main_height).min(app.lines.len() - 1);
                            app.cursor_col = app.cursor_col.min(app.lines[app.cursor_line].len());
                            app.autocomplete_options.clear();
                        }
                        KeyCode::Left => {
                            if app.cursor_col > 0 {
                                app.cursor_col -= 1;
                            } else if app.cursor_line > 0 {
                                app.cursor_line -= 1;
                                app.cursor_col = app.lines[app.cursor_line].len();
                            }
                            app.autocomplete_options.clear();
                        }
                        KeyCode::Right => {
                            let line_length = app.lines[app.cursor_line].chars().count();
                            if app.cursor_col < line_length {
                                app.cursor_col += 1;
                            } else if app.cursor_line < app.lines.len() - 1 {
                                app.cursor_line += 1;
                                app.cursor_col = 0;
                            }
                            app.autocomplete_options.clear();
                        }
                        KeyCode::Up => {
                            if app.cursor_line > 0 {
                                app.cursor_line -= 1;
                                app.cursor_col = app.cursor_col.min(app.lines[app.cursor_line].len());
                            }
                            app.autocomplete_options.clear();
                        }
                        KeyCode::Down => {
                            if app.cursor_line < app.lines.len() - 1 {
                                app.cursor_line += 1;
                                app.cursor_col = app.cursor_col.min(app.lines[app.cursor_line].len());
                            }
                            app.autocomplete_options.clear();
                        }
                        KeyCode::Esc => break,
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

fn recalculate(app: &mut App, calculator: &mut CalculatorService) {
    app.results = calculator.evaluate_document(&app.lines);
}

fn get_current_word_bounds(line: &str, cursor_col: usize) -> (usize, usize) {
    let mut start = cursor_col;
    let mut end = cursor_col;
    let chars: Vec<char> = line.chars().collect();
    
    if cursor_col > 0 && cursor_col <= chars.len() {
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }
    }
    (start, end)
}

fn update_autocomplete(app: &mut App, calculator: &CalculatorService) {
    app.autocomplete_options.clear();
    app.autocomplete_index = None;
    
    let line = &app.lines[app.cursor_line];
    let (start, _end) = get_current_word_bounds(line, app.cursor_col);
    
    if start < app.cursor_col {
        let prefix: String = line.chars().skip(start).take(app.cursor_col - start).collect();
        if prefix.len() >= 1 && prefix.chars().next().unwrap().is_alphabetic() {
            let mut matches: Vec<String> = calculator.state.get_variable_names()
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