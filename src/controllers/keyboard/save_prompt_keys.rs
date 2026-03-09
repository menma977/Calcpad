use crate::models::app::App;
use crate::repositories::file_manager_repository;
use crossterm::event::{self, KeyCode};

fn perform_save(app: &mut App, path: &str) {
    if file_manager_repository::save(&app.lines, path).is_ok() {
        app.file_path = Some(path.to_string());
        app.set_status("Saved successfully!");
    } else {
        app.set_status("Error: could not save file!");
    }
}

pub fn handle_save_prompt_keys(app: &mut App, key: event::KeyEvent) {
    match key.code {
        KeyCode::Char(c) => app.save_input.push(c),
        KeyCode::Backspace => {
            app.save_input.pop();
        }
        KeyCode::Enter => {
            if !app.save_input.is_empty() {
                let name = app.save_input.trim().to_string();
                let path = file_manager_repository::normalize_cpad_path(&name);
                perform_save(app, &path);
            } else {
                app.set_status("Save cancelled: empty filename.");
            }
            app.cancel_save_prompt();
        }
        KeyCode::Esc => {
            app.cancel_save_prompt();
        }
        _ => {}
    }
}
