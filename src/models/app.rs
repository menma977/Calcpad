use std::time::Instant;

#[derive(PartialEq)]
pub enum AppMode {
    Editing,    // Normal mode
    SavePrompt, // Prompting for file name to save
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

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn get_current_line(&self) -> &String {
        &self.lines[self.cursor_line]
    }

    pub fn get_current_line_mut(&mut self) -> &mut String {
        &mut self.lines[self.cursor_line]
    }

    pub fn get_current_word_bounds(&self) -> (usize, usize) {
        let line = self.get_current_line();
        let mut start = self.cursor_col;
        let mut end = self.cursor_col;
        let chars: Vec<char> = line.chars().collect();

        if self.cursor_col > 0 && self.cursor_col <= chars.len() {
            while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
                start -= 1;
            }
            while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
                end += 1;
            }
        }
        (start, end)
    }

    pub fn insert_char(&mut self, character: char) {
        let byte_index = self
            .get_current_line()
            .char_indices()
            .nth(self.cursor_col)
            .map(|(index, _)| index)
            .unwrap_or(self.get_current_line().len());
        self.get_current_line_mut().insert(byte_index, character);
        self.cursor_col += 1;
    }

    pub fn backspace(&mut self) -> bool {
        if self.cursor_col > 0 {
            let line_length = self.get_current_line().chars().count();
            let safe_col = self.cursor_col.min(line_length);
            if let Some((byte_index, _)) = self.get_current_line().char_indices().nth(safe_col - 1)
            {
                self.get_current_line_mut().remove(byte_index);
                self.cursor_col = safe_col - 1;
                return true;
            }
        } else if self.cursor_line > 0 {
            let current_line_content = self.lines.remove(self.cursor_line);
            self.results.remove(self.cursor_line);
            self.cursor_line -= 1;
            self.cursor_col = self.get_current_line().chars().count();
            self.get_current_line_mut().push_str(&current_line_content);
            return true;
        }
        false
    }

    pub fn delete(&mut self) -> bool {
        let line_length = self.get_current_line().chars().count();
        if let Some((byte_index, _)) = (self.cursor_col < line_length)
            .then(|| self.get_current_line().char_indices().nth(self.cursor_col))
            .flatten()
        {
            self.get_current_line_mut().remove(byte_index);
            return true;
        }
        false
    }

    pub fn confirm_autocomplete(&mut self) -> bool {
        if let Some(idx) = self.autocomplete_index {
            if idx < self.autocomplete_options.len() {
                let word = self.autocomplete_options[idx].clone();
                let (start, _) = self.get_current_word_bounds();
                let current_line = self.get_current_line();

                let byte_start = current_line
                    .char_indices()
                    .nth(start)
                    .map(|(i, _)| i)
                    .unwrap_or(current_line.len());
                let byte_cursor = current_line
                    .char_indices()
                    .nth(self.cursor_col)
                    .map(|(i, _)| i)
                    .unwrap_or(current_line.len());

                let mut new_line = String::new();
                new_line.push_str(&current_line[..byte_start]);
                new_line.push_str(&word);
                new_line.push_str(&current_line[byte_cursor..]);

                self.lines[self.cursor_line] = new_line;
                self.cursor_col = start + word.len();
                self.clear_autocomplete();
                return true;
            }
        }
        false
    }

    pub fn clear_autocomplete(&mut self) {
        self.autocomplete_options.clear();
        self.autocomplete_index = None;
    }
}
