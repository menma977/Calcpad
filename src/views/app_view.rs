use crate::models::app::{App, AppMode};
use crate::services::syntax_service::highlight_line;
use ratatui::{prelude::*, widgets::*};

// Custom Colors based on request
const COLOR_PRIMARY: Color = Color::Rgb(214, 113, 158);   // #d6719e (Pinkish)
const COLOR_SECONDARY: Color = Color::Rgb(97, 175, 239); // #61afef (Blueish)
const COLOR_BG_DARK: Color = Color::Rgb(30, 34, 42);     // #1e222a

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
            Constraint::Length(6),
            Constraint::Min(10),
            Constraint::Percentage(25),
        ])
        .split(rows[0]);

    // Line numbers
    let line_numbers: Vec<Line> = (1..=app.lines.len())
        .map(|number| {
            Line::from(format!("{:>3} ", number))
                .style(Style::default().fg(Color::DarkGray))
        })
        .collect();

    let line_number_panel = Paragraph::new(line_numbers)
        .block(
            Block::default()
                .title(
                    Line::from(Span::styled(
                        " No ",
                        Style::default()
                            .fg(COLOR_SECONDARY)
                            .add_modifier(Modifier::BOLD),
                    ))
                    .alignment(Alignment::Center),
                )
                .borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM)
                .border_style(Style::default().fg(COLOR_BG_DARK)),
        )
        .scroll((app.scroll_offset, 0));
    frame.render_widget(line_number_panel, columns[0]);

    // Input panel
    let input_lines: Vec<Line> = app.lines.iter().map(|line| highlight_line(line)).collect();

    let input_panel = Paragraph::new(input_lines)
        .block(
            Block::default()
                .title(Span::styled(
                    " Code ",
                    Style::default()
                        .fg(COLOR_SECONDARY)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_BG_DARK)),
        )
        .scroll((app.scroll_offset, app.scroll_x)); // Added horizontal scroll
    frame.render_widget(input_panel, columns[1]);

    // Result panel — primary color for values
    let result_lines: Vec<Line> = app.results
        .iter()
        .map(|result| {
            if result.starts_with("error") {
                Line::from(format!("{} ", result))
                    .style(Style::default().fg(Color::Red))
                    .alignment(Alignment::Right)
            } else if result.is_empty() {
                Line::from("").alignment(Alignment::Right)
            } else {
                Line::from(format!("= {} ", result))
                    .style(
                        Style::default()
                            .fg(COLOR_PRIMARY)
                            .add_modifier(Modifier::BOLD),
                    )
                    .alignment(Alignment::Right)
            }
        })
        .collect();

    let result_panel = Paragraph::new(result_lines)
        .block(
            Block::default()
                .title(Span::styled(
                    " Result ",
                    Style::default()
                        .fg(COLOR_PRIMARY)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_BG_DARK)),
        )
        .scroll((app.scroll_offset, 0));
    frame.render_widget(result_panel, columns[2]);

    // Status bar
    let file_name = app.file_path.as_deref().unwrap_or("unsaved");
    let status = match &app.status_message {
        Some(msg) => format!(" {}  |  {}", file_name, msg),
        None => format!(
            " {}  |  Line {}:{}  |  Esc/Ctrl+C to quit  |  Ctrl+S to save",
            file_name,
            app.cursor_line + 1,
            app.cursor_col + 1,
        ),
    };
    let status_bar = Paragraph::new(status).style(
        Style::default()
            .bg(COLOR_BG_DARK)
            .fg(COLOR_SECONDARY),
    );
    frame.render_widget(status_bar, rows[1]);

    // Save prompt popup
    if let AppMode::SavePrompt = app.mode {
        let popup_area = centered_rect(40, 3, screen);
        let popup = Paragraph::new(format!(" Save as: {}_", app.save_input))
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .title(" Save File ")
                    .borders(Borders::ALL)
                    .style(Style::default().bg(COLOR_BG_DARK)));
        frame.render_widget(Clear, popup_area);
        frame.render_widget(popup, popup_area);
    } else {
        // Render Autocomplete Popup
        if !app.autocomplete_options.is_empty() {
            let display_y = app.cursor_line.saturating_sub(app.scroll_offset as usize);
            let display_x = app.cursor_col.saturating_sub(app.scroll_x as usize);
            
            // Limit the popup size and prevent it from going out of bounds
            let popup_height = (app.autocomplete_options.len() as u16 + 2).min(5); 
            let mut popup_width = 20;
            
            for option in &app.autocomplete_options {
                if option.len() as u16 + 4 > popup_width {
                    popup_width = option.len() as u16 + 4;
                }
            }

            // Calculate x and y taking into account the borders
            let x = columns[1].x + display_x as u16 + 1; // +1 for left border
            let mut y = columns[1].y + display_y as u16 + 2; // +1 for top border, +1 for next line

            // Flip above if not enough space below
            if y + popup_height > screen.height && y > popup_height + 2 {
                y = y.saturating_sub(popup_height + 1);
            }

            let popup_area = Rect {
                x,
                y,
                width: popup_width.min(columns[1].width.saturating_sub(display_x as u16)),
                height: popup_height,
            };

            let items: Vec<ListItem> = app.autocomplete_options.iter().map(|option| {
                ListItem::new(option.clone()).style(Style::default().fg(COLOR_SECONDARY)) // Unselected is Blue
            }).collect();

            let popup = List::new(items)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(COLOR_SECONDARY)) // Border is Blue
                    .bg(COLOR_BG_DARK)) // Background remains Dark #1e222a
                .highlight_style(Style::default()
                    .fg(COLOR_PRIMARY) // Selected item turns Pink
                    .add_modifier(Modifier::BOLD)) // Bold for emphasis
                .highlight_symbol("> "); 
            
            let mut state = ListState::default().with_selected(app.autocomplete_index);
            
            frame.render_widget(Clear, popup_area);
            frame.render_stateful_widget(popup, popup_area, &mut state);
        }

        // Set cursor position (ensure it only shows when editing, not saving)
        let display_y = app.cursor_line.saturating_sub(app.scroll_offset as usize);
        let display_x = app.cursor_col.saturating_sub(app.scroll_x as usize);
        if display_y < columns[1].height.saturating_sub(2) as usize && display_x < columns[1].width.saturating_sub(2) as usize {
            frame.set_cursor_position(Position {
                x: columns[1].x + display_x as u16 + 1,
                y: columns[1].y + display_y as u16 + 1,
            });
        }
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
