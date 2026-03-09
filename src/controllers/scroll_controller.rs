use crate::models::app::App;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

pub type AppTerminal = Terminal<CrosstermBackend<io::Stdout>>;

pub const CHROME_HEIGHT: u16 = 3;

pub fn update_scroll(app: &mut App, terminal: &AppTerminal) -> io::Result<()> {
    let size = terminal.size()?;
    let main_height = size.height.saturating_sub(CHROME_HEIGHT) as usize;

    // Vertical scroll
    if app.cursor_line >= app.scroll_offset as usize + main_height {
        app.scroll_offset = app.cursor_line.saturating_sub(main_height - 1) as u16;
    } else if app.cursor_line < app.scroll_offset as usize {
        app.scroll_offset = app.cursor_line as u16;
    }

    // Horizontal scroll
    let result_panel_width = get_result_panel_width(size.width);
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

pub fn get_result_panel_width(terminal_width: u16) -> u16 {
    (terminal_width as f32 * 0.25) as u16
}
