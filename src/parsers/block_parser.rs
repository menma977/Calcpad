pub enum Statement {
    Line {
        index: usize,
        content: String,
    },
    IfBlock {
        condition: String,
        true_statements: Vec<Statement>,
        false_statements: Option<Vec<Statement>>,
    },
}

pub struct BlockParser;

impl BlockParser {
    pub fn parse(lines: &[String]) -> Vec<Statement> {
        let mut statements = Vec::new();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();
            if line.is_empty() {
                i += 1;
                continue;
            }

            if Self::is_if_keyword(line) {
                let (stmt, next_i) = Self::parse_if_block(lines, i);
                statements.push(stmt);
                i = next_i;
            } else {
                statements.push(Statement::Line {
                    index: i,
                    content: line.to_string(),
                });
                i += 1;
            }
        }

        statements
    }

    fn parse_if_block(lines: &[String], start_index: usize) -> (Statement, usize) {
        let mut full_text = String::new();
        let mut line_byte_ranges: Vec<(usize, usize, usize)> = Vec::new(); // (byte_start, byte_end, line_idx)

        // Concatenate lines into a single stream, tracking byte ranges for each line
        for (idx, line) in lines.iter().enumerate().skip(start_index) {
            let byte_start = full_text.len();
            let line_with_newline = format!("{}\n", line);
            full_text.push_str(&line_with_newline);
            let byte_end = full_text.len();
            line_byte_ranges.push((byte_start, byte_end, idx));
        }

        let chars: Vec<char> = full_text.chars().collect();
        let mut cursor = 0;

        let mut condition = String::new();
        let mut true_block_indices = None;
        let mut false_block_indices = None;

        // 1. Extract Condition: if ( ... )
        while cursor < chars.len() {
            if chars[cursor] == '(' {
                let start = cursor + 1;
                let mut cond_depth = 1;
                cursor += 1;
                while cursor < chars.len() && cond_depth > 0 {
                    if chars[cursor] == '(' {
                        cond_depth += 1;
                    } else if chars[cursor] == ')' {
                        cond_depth -= 1;
                    }
                    cursor += 1;
                }
                condition = chars[start..cursor - 1]
                    .iter()
                    .collect::<String>()
                    .trim()
                    .to_string();
                break;
            }
            cursor += 1;
        }

        // 2. Extract True Block: { ... }
        while cursor < chars.len() {
            if chars[cursor] == '{' {
                let start = cursor + 1;
                let mut depth = 1;
                cursor += 1;
                while cursor < chars.len() && depth > 0 {
                    if chars[cursor] == '{' {
                        depth += 1;
                    } else if chars[cursor] == '}' {
                        depth -= 1;
                    }
                    cursor += 1;
                }
                true_block_indices = Some((start, cursor - 1));
                break;
            }
            cursor += 1;
        }

        // 3. Extract Else Block: else { ... } (optional)
        let mut temp_cursor = cursor;
        while temp_cursor < chars.len() && chars[temp_cursor].is_whitespace() {
            temp_cursor += 1;
        }
        if temp_cursor + 4 <= chars.len()
            && chars[temp_cursor..temp_cursor + 4]
                .iter()
                .collect::<String>()
                == "else"
        {
            cursor = temp_cursor + 4;
            while cursor < chars.len() {
                if chars[cursor] == '{' {
                    let start = cursor + 1;
                    let mut depth = 1;
                    cursor += 1;
                    while cursor < chars.len() && depth > 0 {
                        if chars[cursor] == '{' {
                            depth += 1;
                        } else if chars[cursor] == '}' {
                            depth -= 1;
                        }
                        cursor += 1;
                    }
                    false_block_indices = Some((start, cursor - 1));
                    break;
                }
                cursor += 1;
            }
        }

        let extract_stmts = |bounds: (usize, usize)| -> Vec<Statement> {
            let (start, end) = bounds;
            let mut stmts = Vec::new();
            let mut current_line_idx = 0;
            let mut current_content = String::new();

            for char_idx in start..end {
                // Find which line this character index belongs to via binary search
                let line_idx = line_byte_ranges
                    .binary_search_by_key(&(char_idx as i32), |&(byte_start, _, _)| {
                        byte_start as i32
                    })
                    .map(|idx| line_byte_ranges[idx].2)
                    .or_else(|idx| {
                        if idx > 0 && char_idx < line_byte_ranges[idx].1 {
                            Ok(line_byte_ranges[idx - 1].2)
                        } else {
                            Err(0)
                        }
                    })
                    .unwrap_or(0);

                if current_line_idx == 0 || line_idx != current_line_idx {
                    if !current_content.is_empty() {
                        Self::push_line_stmts(&mut stmts, current_line_idx, &current_content);
                    }
                    current_content.clear();
                    current_line_idx = line_idx;
                }

                if chars[char_idx] != '\n' {
                    current_content.push(chars[char_idx]);
                }
            }
            if !current_content.is_empty() {
                Self::push_line_stmts(&mut stmts, current_line_idx, &current_content);
            }
            stmts
        };

        let true_statements = true_block_indices.map(extract_stmts).unwrap_or_default();
        let false_statements = false_block_indices.map(extract_stmts);

        // Find end_line using binary search on ranges
        let end_line = match line_byte_ranges
            .binary_search_by_key(&(cursor as i32), |&(byte_start, _, _)| byte_start as i32)
        {
            Ok(idx) => line_byte_ranges[idx].2 + 1,
            Err(idx) => {
                if idx > 0 && idx <= line_byte_ranges.len() {
                    line_byte_ranges[idx - 1].2 + 1
                } else {
                    start_index + 1
                }
            }
        };

        (
            Statement::IfBlock {
                condition,
                true_statements,
                false_statements,
            },
            end_line,
        )
    }

    fn push_line_stmts(stmts: &mut Vec<Statement>, idx: usize, content: &str) {
        for part in content.split(';') {
            let p = part.trim();
            if !p.is_empty() {
                stmts.push(Statement::Line {
                    index: idx,
                    content: p.to_string(),
                });
            }
        }
    }

    fn is_if_keyword(line: &str) -> bool {
        line == "if" || line.starts_with("if(") || line.starts_with("if ")
    }
}
