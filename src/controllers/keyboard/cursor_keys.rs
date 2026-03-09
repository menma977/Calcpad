use crate::models::app::App;

pub fn move_cursor_up(app: &mut App) {
    if app.cursor_line > 0 {
        app.cursor_line -= 1;
        app.cursor_col = app.cursor_col.min(app.get_current_line().chars().count());
    }
    app.clear_autocomplete();
}

pub fn move_cursor_down(app: &mut App) {
    if app.cursor_line < app.lines.len().saturating_sub(1) {
        app.cursor_line += 1;
        app.cursor_col = app.cursor_col.min(app.get_current_line().chars().count());
    }
    app.clear_autocomplete();
}

pub fn move_cursor_left(app: &mut App) {
    if app.cursor_col > 0 {
        app.cursor_col -= 1;
    } else if app.cursor_line > 0 {
        app.cursor_line -= 1;
        app.cursor_col = app.get_current_line().chars().count();
    }
    app.clear_autocomplete();
}

pub fn move_cursor_right(app: &mut App) {
    if app.cursor_col < app.get_current_line().chars().count() {
        app.cursor_col += 1;
    } else if app.cursor_line < app.lines.len().saturating_sub(1) {
        app.cursor_line += 1;
        app.cursor_col = 0;
    }
    app.clear_autocomplete();
}

pub fn move_page(app: &mut App, jump: i32) {
    if jump < 0 {
        app.cursor_line = app.cursor_line.saturating_sub((-jump) as usize);
    } else {
        app.cursor_line = (app.cursor_line + jump as usize).min(app.lines.len().saturating_sub(1));
    }
    app.cursor_col = app.cursor_col.min(app.get_current_line().chars().count());
    app.clear_autocomplete();
}
