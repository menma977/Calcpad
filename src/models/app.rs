pub enum AppMode {
    Editing,        // Normal mode
    SavePrompt,     // Prompting for file name to save
}

pub struct App {
    pub lines: Vec<String>,
    pub results: Vec<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub file_path: Option<String>,
    pub mode: AppMode,
    pub save_input: String,
}

impl App {
    pub fn new() -> App {
        App {
            lines: vec![String::new()],
            results: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            file_path: None,
            mode: AppMode::Editing,
            save_input: String::new(),
        }
    }
}