use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

// Custom Colors based on request
const COLOR_PRIMARY: Color = Color::Rgb(214, 113, 158);   // #d6719e (Pinkish)
const COLOR_SECONDARY: Color = Color::Rgb(97, 175, 239); // #61afef (Blueish)

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
    let mut current = String::new();

    let push_current = |spans: &mut Vec<Span<'_>>, current: &mut String| {
        if !current.is_empty() {
            let is_number = current.chars().all(|c| c.is_ascii_digit() || c == '.');
            let is_keyword = current == "if"
                || current == "else"
                || current == "true"
                || current == "false";

            let color = if is_number {
                Color::LightYellow
            } else if is_keyword {
                COLOR_PRIMARY
            } else {
                COLOR_SECONDARY // variables
            };

            spans.push(Span::styled(current.clone(), Style::default().fg(color)));
            current.clear();
        }
    };

    for character in expression.chars() {
        match character {
            '0'..='9' | '.' | 'a'..='z' | 'A'..='Z' | '_' => {
                current.push(character);
            }
            '+' | '-' | '*' | '/' | '%' | '=' | '!' | '>' | '<' | '&' | '|' | '^' | '?' | ':' => {
                push_current(&mut spans, &mut current);
                spans.push(Span::styled(
                    character.to_string(),
                    Style::default().fg(COLOR_PRIMARY), // Operators use primary
                ));
            }
            '(' | ')' | '{' | '}' | '[' | ']' => {
                push_current(&mut spans, &mut current);
                spans.push(Span::styled(
                    character.to_string(),
                    Style::default().fg(Color::DarkGray), // Brackets
                ));
            }
            ' ' | ',' | ';' => {
                push_current(&mut spans, &mut current);
                spans.push(Span::styled(
                    character.to_string(),
                    Style::default().fg(Color::Reset),
                ));
            }
            _ => {
                current.push(character);
            }
        }
    }
    push_current(&mut spans, &mut current);
    spans
}
