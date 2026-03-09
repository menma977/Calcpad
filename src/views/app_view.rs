use crate::models::app::{App, AppMode};
use ratatui::{prelude::*, widgets::*};

pub fn render(frame: &mut Frame, app: &App) {
    let screen = frame.area();

    /* Split vertically — main area top, status bar bottom */
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(screen);

    /* Split main area — line numbers, input, results */
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(10),
            Constraint::Percentage(20),
        ])
        .split(rows[0]);

    // Line numbers
    let line_numbers: Vec<Line> = (1..=app.lines.len())
        .map(|number| {
            Line::from(format!("{:>2} ", number))
                .style(Style::default().fg(Color::DarkGray))
        })
        .collect();

    let line_number_panel = Paragraph::new(line_numbers)
        .block(Block::default().borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM));
    frame.render_widget(line_number_panel, columns[0]);

    // Input panel
    let input_lines: Vec<Line> = app.lines.iter()
        .map(|line| highlight_line(line))
        .collect();

    let input_panel = Paragraph::new(input_lines)
        .block(Block::default().title(" Calcpad ").borders(Borders::ALL));
    frame.render_widget(input_panel, columns[1]);

    // Result panel — red for errors, green for values
    let result_lines: Vec<Line> = app.results.iter()
        .map(|result| {
            if result.starts_with("error") {
                Line::from(result.as_str())
                    .style(Style::default().fg(Color::Red))
                    .alignment(Alignment::Right)
            } else if result.is_empty() {
                Line::from("").alignment(Alignment::Right)
            } else {
                Line::from(result.as_str())
                    .style(Style::default().fg(Color::Green))
                    .alignment(Alignment::Right)
            }
        })
        .collect();

    let result_panel = Paragraph::new(result_lines)
        .block(Block::default().title(" Result ").borders(Borders::ALL));
    frame.render_widget(result_panel, columns[2]);

    // Status bar
    let file_name = app.file_path.as_deref().unwrap_or("unsaved");
    let status = format!(
        " {}  |  Line {}:{}  |  Esc to quit  |  Ctrl+S to save",
        file_name,
        app.cursor_line + 1,
        app.cursor_col + 1,
    );
    let status_bar = Paragraph::new(status)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));
    frame.render_widget(status_bar, rows[1]);

    // Save prompt popup
    if let AppMode::SavePrompt = app.mode {
        let popup_area = centered_rect(40, 3, screen);
        let popup = Paragraph::new(format!(" Save as: {}_", app.save_input))
            .style(Style::default().fg(Color::White))
            .block(Block::default()
                .title(" Save File ")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::DarkGray)));
        frame.render_widget(Clear, popup_area);
        frame.render_widget(popup, popup_area);
    } else {
        // Only show the cursor in editing mode
        frame.set_cursor_position(Position {
            x: columns[1].x + app.cursor_col as u16 + 1,
            y: columns[1].y + app.cursor_line as u16 + 1,
        });
    }
}

/// Returns a centered rect for popup dialogs.
fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(height),
            Constraint::Fill(1),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

/// Parses a line and returns it with syntax highlighting.
fn highlight_line(line: &str) -> Line<'_> {
    if line.trim().starts_with("//") {
        return Line::from(Span::styled(line, Style::default().fg(Color::DarkGray)));
    }

    let mut spans: Vec<Span> = Vec::new();

    if let Some(eq_pos) = line.find('=') {
        let var_name = &line[..eq_pos];
        let expression = &line[eq_pos + 1..];

        spans.push(Span::styled(
            var_name.to_string(),
            Style::default().fg(Color::Cyan),
        ));
        spans.push(Span::styled(
            "=".to_string(),
            Style::default().fg(Color::DarkGray),
        ));
        spans.extend(highlight_expression(expression));
        return Line::from(spans);
    }

    spans.extend(highlight_expression(line));
    Line::from(spans)
}

/// Highlights numbers, operators, and variable references in an expression.
fn highlight_expression(expression: &str) -> Vec<Span<'_>> {
    let mut spans: Vec<Span> = Vec::new();
    let mut current = String::new();
    let mut chars = expression.chars().peekable();

    while let Some(character) = chars.next() {
        match character {
            '0'..='9' | '.' => current.push(character),
            '+' | '-' | '*' | '/' | '(' | ')' | ' ' => {
                if !current.is_empty() {
                    spans.push(Span::styled(
                        current.clone(),
                        Style::default().fg(Color::Yellow),
                    ));
                    current.clear();
                }
                spans.push(Span::styled(
                    character.to_string(),
                    Style::default().fg(Color::White),
                ));
            }
            _ => {
                if current.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                    spans.push(Span::styled(
                        current.clone(),
                        Style::default().fg(Color::Yellow),
                    ));
                    current.clear();
                }
                current.push(character);
            }
        }
    }

    if !current.is_empty() {
        let is_number = current.chars().next().map_or(false, |c| c.is_ascii_digit());
        spans.push(Span::styled(
            current,
            Style::default().fg(if is_number { Color::Yellow } else { Color::Cyan }),
        ));
    }

    spans
}