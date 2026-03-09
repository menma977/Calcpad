use crate::views::thames::{COLOR_PRIMARY, COLOR_SECONDARY};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

/// Parses a line and returns it with syntax highlighting.
pub fn highlight_line(line: &str) -> Line<'_> {
    if line.trim().starts_with("//") {
        return Line::from(Span::styled(line, Style::default().fg(Color::DarkGray)));
    }

    let mut spans: Vec<Span> = Vec::new();

    if let Some(eq_pos) = line.find('=') {
        // Simple heuristic to not color `==` as variable assignment
        if !line[eq_pos..].starts_with("==")
            && !line[..eq_pos].ends_with('>')
            && !line[..eq_pos].ends_with('<')
            && !line[..eq_pos].ends_with('!')
        {
            let var_name = &line[..eq_pos];
            let expression = &line[eq_pos + 1..];

            spans.push(Span::styled(
                var_name.to_string(),
                Style::default()
                    .fg(COLOR_SECONDARY)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(
                "=".to_string(),
                Style::default().fg(COLOR_PRIMARY),
            ));
            spans.extend(highlight_expression(expression));
            return Line::from(spans);
        }
    }

    spans.extend(highlight_expression(line));
    Line::from(spans)
}

/// Highlights numbers, operators, and variable references in an expression.
fn highlight_expression(expression: &str) -> Vec<Span<'_>> {
    let mut spans: Vec<Span> = Vec::new();
    let mut start_idx = 0;

    let chars: Vec<(usize, char)> = expression.char_indices().collect();
    let mut i = 0;

    while i < chars.len() {
        let (byte_idx, c) = chars[i];

        match c {
            '+' | '-' | '*' | '/' | '%' | '=' | '!' | '>' | '<' | '&' | '|' | '^' | '?' | ':'
            | '(' | ')' | '{' | '}' | '[' | ']' | ' ' | ',' | ';' => {
                // Push previous word if exists
                if byte_idx > start_idx {
                    spans.push(create_word_span(&expression[start_idx..byte_idx]));
                }

                // Style for the operator/symbol
                let color = match c {
                    '(' | ')' | '{' | '}' | '[' | ']' => Color::DarkGray,
                    ' ' | ',' | ';' => Color::Reset,
                    _ => COLOR_PRIMARY,
                };
                spans.push(Span::styled(c.to_string(), Style::default().fg(color)));

                start_idx = byte_idx + c.len_utf8();
            }
            _ => {}
        }
        i += 1;
    }

    if start_idx < expression.len() {
        spans.push(create_word_span(&expression[start_idx..]));
    }

    spans
}

fn create_word_span(word: &str) -> Span<'_> {
    let is_number = word.chars().all(|c| c.is_ascii_digit() || c == '.');
    let is_keyword = word == "if" || word == "else" || word == "true" || word == "false";

    let color = if is_number {
        Color::LightYellow
    } else if is_keyword {
        COLOR_PRIMARY
    } else {
        COLOR_SECONDARY // variables
    };

    Span::styled(word.to_string(), Style::default().fg(color))
}
