use crate::controllers::keyboard::{autocomplete_keys::handle_autocomplete_keys, cursor_keys::*};
use crate::controllers::scroll_controller::AppTerminal;
use crate::models::app::{App, AppMode};
use crate::repositories::file_manager_repository;
use crate::services::calculator_service::CalculatorService;
use crossterm::event::{self, KeyCode};

pub fn handle_editing_keys(
    app: &mut App,
    calculator: &mut CalculatorService,
    terminal: &AppTerminal,
    key: event::KeyEvent,
) {
    if !app.autocomplete_options.is_empty() && handle_autocomplete_keys(app, calculator, key) {
        return;
    }

    match key.code {
        KeyCode::Char('s') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
            if let Some(path) = app.file_path.clone() {
                if file_manager_repository::save(&app.lines, &path).is_ok() {
                    app.set_status("Saved successfully!");
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
            let prefix_lower = prefix.to_lowercase();
            let mut scored_matches: Vec<(i32, String)> = calculator
                .state
                .get_variable_names()
                .into_iter()
                .filter(|name| name != &prefix)
                .filter_map(|name| {
                    fuzzy_score(&name.to_lowercase(), &prefix_lower).map(|score| (score, name))
                })
                .collect();
            scored_matches.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
            let matches: Vec<String> = scored_matches.into_iter().map(|(_, name)| name).collect();
            if !matches.is_empty() {
                app.autocomplete_options = matches;
                app.autocomplete_index = Some(0);
            }
        }
    }
}

fn fuzzy_score(name: &str, prefix: &str) -> Option<i32> {
    let name_chars: Vec<char> = name.chars().collect();
    let prefix_chars: Vec<char> = prefix.chars().collect();
    let mut name_idx = 0;
    let mut score = 0i32;
    let mut last_matched: Option<usize> = None;
    let mut consecutive_run = 0i32;

    for p_char in &prefix_chars {
        let found = name_chars[name_idx..].iter().position(|&c| c == *p_char);
        match found {
            None => return None,
            Some(offset) => {
                let matched_idx = name_idx + offset;

                // Consecutive match bonus
                if last_matched == Some(matched_idx.saturating_sub(1)) {
                    consecutive_run += 1;
                    score += consecutive_run * 10;
                } else {
                    consecutive_run = 1;
                    score += 10;
                }

                // Word boundary bonus (start of name or after '_')
                if matched_idx == 0 || name_chars[matched_idx - 1] == '_' {
                    score += 10;
                }

                // Gap penalty (distance from previous match or start)
                let gap = if let Some(prev) = last_matched {
                    (matched_idx - prev - 1) as i32
                } else {
                    matched_idx as i32
                };
                score -= gap;

                last_matched = Some(matched_idx);
                name_idx = matched_idx + 1;
            }
        }
    }

    // Prefix match bonus (highest priority)
    if name.starts_with(prefix) {
        score += 1000;
    }

    Some(score)
}
