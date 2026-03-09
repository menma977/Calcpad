use crate::models::app::App;
use crate::services::calculator_service::CalculatorService;
use crossterm::event::{self, KeyCode};

pub fn handle_autocomplete_keys(
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
                app.autocomplete_options.len().saturating_sub(1)
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
