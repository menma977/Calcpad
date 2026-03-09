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
            
            if line.starts_with("if") {
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
        let mut index_map = Vec::new();
        
        for (i, line) in lines.iter().enumerate().skip(start_index) {
            let chunk = format!("{} \n", line);
            for _ in 0..chunk.len() {
                index_map.push(i);
            }
            full_text.push_str(&chunk);
        }

        let mut chars = full_text.char_indices().peekable();
        let mut cond_start = None;
        let mut cond_end = None;
        let mut block1_start = None;
        let mut block1_end = None;
        let mut else_start = None;
        let mut block2_start = None;
        let mut block2_end = None;

        let mut depth = 0;
        let mut in_condition = false;
        let mut in_block1 = false;
        let mut in_block2 = false;

        while let Some((byte_idx, c)) = chars.next() {
            match c {
                '(' if !in_block1 && !in_block2 => {
                    if !in_condition {
                        cond_start = Some(byte_idx);
                        in_condition = true;
                    }
                    depth += 1;
                }
                ')' if in_condition => {
                    depth -= 1;
                    if depth == 0 {
                        cond_end = Some(byte_idx);
                        in_condition = false;
                    }
                }
                '{' => {
                    if !in_block1 && block1_end.is_none() {
                        in_block1 = true;
                        block1_start = Some(byte_idx);
                    } else if !in_block2 && else_start.is_some() {
                        in_block2 = true;
                        block2_start = Some(byte_idx);
                    }
                    depth += 1;
                }
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        if in_block1 {
                            in_block1 = false;
                            block1_end = Some(byte_idx);
                            
                            let mut lookahead = chars.clone();
                            let mut found_else = false;
                            while let Some((la_idx, la_c)) = lookahead.next() {
                                if la_c.is_whitespace() { continue; }
                                if full_text[la_idx..].starts_with("else") {
                                    else_start = Some(la_idx);
                                    found_else = true;
                                    break;
                                } else {
                                    break;
                                }
                            }
                            if !found_else {
                                break;
                            }
                        } else if in_block2 {
                            block2_end = Some(byte_idx);
                            break;
                        }
                    }
                }
                _ => {}
            }
        }

        let condition = if let (Some(s), Some(e)) = (cond_start, cond_end) {
            full_text[s + 1..e].trim().to_string()
        } else {
            String::new()
        };

        let extract_statements = |start: usize, end: usize| -> Vec<Statement> {
            let inner_text = &full_text[start + 1..end];
            let inner_start_line = index_map[start + 1];
            
            let mut sub_lines = Vec::new();
            let mut current_sub_line = String::new();
            let mut last_line_idx = inner_start_line;
            
            for (i, c) in inner_text.char_indices() {
                let current_line_idx = index_map[start + 1 + i];
                if current_line_idx != last_line_idx {
                    sub_lines.push(current_sub_line.clone());
                    current_sub_line.clear();
                    
                    while last_line_idx + 1 < current_line_idx {
                        sub_lines.push(String::new());
                        last_line_idx += 1;
                    }
                    last_line_idx = current_line_idx;
                }
                if c != '\n' {
                    current_sub_line.push(c);
                }
            }
            sub_lines.push(current_sub_line);
            
            let mut stmts = Vec::new();
            for (offset, line) in sub_lines.into_iter().enumerate() {
                let actual_idx = inner_start_line + offset;
                for part in line.split(';') {
                    let p = part.trim();
                    if !p.is_empty() {
                        stmts.push(Statement::Line { index: actual_idx, content: p.to_string() });
                    }
                }
            }
            stmts
        };

        let true_statements = if let (Some(s), Some(e)) = (block1_start, block1_end) {
            extract_statements(s, e)
        } else {
            Vec::new()
        };

        let false_statements = if let (Some(s), Some(e)) = (block2_start, block2_end) {
            Some(extract_statements(s, e))
        } else {
            None
        };

        let end_char_idx = block2_end.or(block1_end).unwrap_or(0);
        let end_line_idx = *index_map.get(end_char_idx).unwrap_or(&start_index);

        (Statement::IfBlock {
            condition,
            true_statements,
            false_statements,
        }, end_line_idx + 1)
    }
}
