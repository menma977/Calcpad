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

    fn scan_braces(chars: &[char], start: usize) -> Option<(usize, usize, usize)> {
        if start >= chars.len() || chars[start] != '{' {
            return None;
        }
        let content_start = start + 1;
        let mut cursor = start + 1;
        let mut depth = 1;
        while cursor < chars.len() && depth > 0 {
            if chars[cursor] == '{' {
                depth += 1;
            } else if chars[cursor] == '}' {
                depth -= 1;
            }
            cursor += 1;
        }
        if depth == 0 {
            Some((content_start, cursor - 1, cursor))
        } else {
            None
        }
    }

    fn find_next_block(chars: &[char], start: usize) -> Option<(usize, usize, usize)> {
        let mut cursor = start;
        while cursor < chars.len() {
            // Skip string literals
            if chars[cursor] == '"' {
                cursor += 1;
                while cursor < chars.len() && chars[cursor] != '"' {
                    if chars[cursor] == '\\' && cursor + 1 < chars.len() {
                        cursor += 2; // skip escaped char
                    } else {
                        cursor += 1;
                    }
                }
                cursor += 1; // skip closing quote
                continue;
            }

            if let Some(block) = Self::scan_braces(chars, cursor) {
                return Some(block);
            }
            cursor += 1;
        }
        None
    }

    fn parse_if_block(lines: &[String], start_index: usize) -> (Statement, usize) {
        // PASS 1: Scan original lines to find where the if-else block ends
        let mut block_end_line = start_index;
        let mut depth = 0i32;
        let mut in_string = false;
        let mut escape_next = false;
        let mut found_opening_brace = false;
        let mut found_first_closing_brace = false;

        'scan_outer: for (offset, line) in lines.iter().enumerate().skip(start_index) {
            for c in line.chars() {
                if escape_next {
                    escape_next = false;
                    continue;
                }

                if c == '\\' && in_string {
                    escape_next = true;
                    continue;
                }

                if c == '"' {
                    in_string = !in_string;
                    continue;
                }

                if !in_string {
                    match c {
                        '{' => {
                            depth += 1;
                            found_opening_brace = true;
                        }
                        '}' => {
                            depth -= 1;
                            if found_opening_brace && depth == 0 && !found_first_closing_brace {
                                found_first_closing_brace = true;
                            } else if found_first_closing_brace && depth == 0 {
                                block_end_line = offset;
                                break 'scan_outer;
                            }
                        }
                        _ => {}
                    }
                }
            }

            if found_first_closing_brace && depth == 0 {
                let trimmed = line.trim();
                let next_line_is_else = lines
                    .get(offset + 1)
                    .map(|l| l.trim().starts_with("else"))
                    .unwrap_or(false);
                if !trimmed.starts_with("else") && !next_line_is_else && !trimmed.is_empty() {
                    block_end_line = offset;
                    break 'scan_outer;
                }
            }
        }

        if !found_opening_brace || !found_first_closing_brace {
            block_end_line = lines.len().saturating_sub(1);
        }

        // PASS 2: Concatenate only lines from start_index to block_end_line (inclusive)
        let mut full_text = String::new();
        let mut line_byte_ranges: Vec<(usize, usize, usize)> = Vec::new(); // (byte_start, byte_end, line_idx)

        for (idx, line) in lines
            .iter()
            .enumerate()
            .skip(start_index)
            .take(block_end_line - start_index + 1)
        {
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
            // Skip string literals
            if chars[cursor] == '"' {
                cursor += 1;
                while cursor < chars.len() && chars[cursor] != '"' {
                    if chars[cursor] == '\\' && cursor + 1 < chars.len() {
                        cursor += 2;
                    } else {
                        cursor += 1;
                    }
                }
                cursor += 1;
                continue;
            }

            if chars[cursor] == '(' {
                let start = cursor + 1;
                let mut cond_depth = 1;
                cursor += 1;
                while cursor < chars.len() && cond_depth > 0 {
                    // Skip nested strings in condition
                    if chars[cursor] == '"' {
                        cursor += 1;
                        while cursor < chars.len() && chars[cursor] != '"' {
                            if chars[cursor] == '\\' && cursor + 1 < chars.len() {
                                cursor += 2;
                            } else {
                                cursor += 1;
                            }
                        }
                        cursor += 1;
                        continue;
                    }

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
        if let Some((content_start, content_end, next_cursor)) =
            Self::find_next_block(&chars, cursor)
        {
            true_block_indices = Some((content_start, content_end));
            cursor = next_cursor;
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
            && (temp_cursor + 4 >= chars.len()
                || (!chars[temp_cursor + 4].is_alphanumeric() && chars[temp_cursor + 4] != '_'))
        {
            cursor = temp_cursor + 4;
            if let Some((content_start, content_end, _)) = Self::find_next_block(&chars, cursor) {
                false_block_indices = Some((content_start, content_end));
            }
        }

        let extract_stmts = |bounds: (usize, usize)| -> Vec<Statement> {
            let (start, end) = bounds;
            let mut stmts = Vec::new();
            let mut current_line_idx: Option<usize> = None;
            let mut current_content = String::new();

            // Build char-to-byte lookup table once
            let char_to_byte: Vec<usize> = full_text
                .char_indices()
                .map(|(byte_idx, _)| byte_idx)
                .chain(std::iter::once(full_text.len()))
                .collect();

            for char_idx in start..end {
                // Convert char index to byte index using lookup table
                let byte_idx = if char_idx < char_to_byte.len() {
                    char_to_byte[char_idx]
                } else {
                    full_text.len()
                };

                // Find which line this byte index belongs to via binary search
                let line_idx = line_byte_ranges
                    .binary_search_by_key(&(byte_idx as i32), |&(byte_start, _, _)| {
                        byte_start as i32
                    })
                    .map(|idx| line_byte_ranges[idx].2)
                    .or_else(|idx| {
                        // idx from Err is the insertion point; can be == len()
                        if idx > 0 && idx <= line_byte_ranges.len() {
                            // Check if byte_idx falls within the previous range
                            let prev_idx = idx - 1;
                            if byte_idx < line_byte_ranges[prev_idx].1 {
                                Ok(line_byte_ranges[prev_idx].2)
                            } else {
                                // byte_idx is past all ranges - use last line
                                Ok(line_byte_ranges[prev_idx].2)
                            }
                        } else {
                            Err(0)
                        }
                    })
                    .unwrap_or(0);

                if current_line_idx.is_none() || line_idx != current_line_idx.unwrap() {
                    if let Some(idx) = current_line_idx {
                        if !current_content.is_empty() {
                            Self::push_line_stmts(&mut stmts, idx, &current_content);
                        }
                    }
                    current_content.clear();
                    current_line_idx = Some(line_idx);
                }

                if chars[char_idx] != '\n' {
                    current_content.push(chars[char_idx]);
                }
            }
            if let Some(idx) = current_line_idx {
                if !current_content.is_empty() {
                    Self::push_line_stmts(&mut stmts, idx, &current_content);
                }
            }
            stmts
        };

        let true_statements = true_block_indices.map(extract_stmts).unwrap_or_default();
        let false_statements = false_block_indices.map(extract_stmts);

        (
            Statement::IfBlock {
                condition,
                true_statements,
                false_statements,
            },
            block_end_line + 1,
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
        line == "if"
            || line.starts_with("if(")
            || line.starts_with("if ")
            || line.starts_with("if\t")
    }
}
