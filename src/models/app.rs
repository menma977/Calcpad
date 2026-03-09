use std::time::Instant;

pub enum AppMode {
    Editing,        // Normal mode
    SavePrompt,     // Prompting for file name to save
}

pub struct App {
    pub lines: Vec<String>,
    pub results: Vec<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub scroll_offset: u16,
    pub scroll_x: u16,
    pub autocomplete_options: Vec<String>,
    pub autocomplete_index: Option<usize>,
    pub file_path: Option<String>,
    pub mode: AppMode,
    pub save_input: String,
    pub status_message: Option<String>,
    pub status_timer: Option<Instant>,
}

impl App {
    pub fn new() -> App {
        App {
            lines: vec![String::new()],
            results: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
            scroll_x: 0,
            autocomplete_options: Vec::new(),
            autocomplete_index: None,
            file_path: None,
            mode: AppMode::Editing,
            save_input: String::new(),
            status_message: None,
            status_timer: None,
        }
    }
}