use crate::controllers::keyboard::handle_save_prompt_keys;
use crate::controllers::scroll_controller::{get_result_panel_width, AppTerminal};
use crate::models::app::App;
use crate::services::calculator_service::CalculatorService;
use crossterm::event::{self, Event, KeyCode, MouseButton, MouseEventKind};
use std::io;

pub fn handle_event(
    app: &mut App,
    calculator: &mut CalculatorService,
    clipboard: &mut Option<arboard::Clipboard>,
    terminal: &AppTerminal,
    event: Event,
) -> io::Result<bool> {
    match event {
        Event::Mouse(mouse_event) => handle_mouse_event(app, clipboard, terminal, mouse_event)?,
        Event::Key(key_event) => return handle_key_event(app, calculator, terminal, key_event),
        _ => {}
    }
    Ok(true)
}

fn handle_key_event(
    app: &mut App,
    calculator: &mut CalculatorService,
    terminal: &AppTerminal,
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

    if key.code == KeyCode::Esc {
        return Ok(false);
    }

    use crate::controllers::keyboard::handle_editing_keys;
    use crate::models::app::AppMode;

    match app.mode {
        AppMode::SavePrompt => handle_save_prompt_keys(app, key),
        AppMode::Editing => handle_editing_keys(app, calculator, terminal, key),
    }
    Ok(true)
}

fn handle_mouse_event(
    app: &mut App,
    clipboard: &mut Option<arboard::Clipboard>,
    terminal: &AppTerminal,
    mouse_event: event::MouseEvent,
) -> io::Result<()> {
    let size = terminal.size()?;
    let main_height = size.height.saturating_sub(3);

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
                    let result_panel_width = get_result_panel_width(size.width);
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
                app.set_status("Result copied to clipboard!");
            }
        }
    }
}
