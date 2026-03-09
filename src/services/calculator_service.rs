use crate::parsers::block_parser::{BlockParser, Statement};
use crate::services::expression_service::ExpressionService;
use crate::services::state_service::StateService;

pub struct CalculatorService {
    pub state: StateService,
}

impl CalculatorService {
    pub fn new() -> Self {
        CalculatorService {
            state: StateService::new(),
        }
    }
}

impl Default for CalculatorService {
    fn default() -> Self {
        Self::new()
    }
}

impl CalculatorService {
    pub fn evaluate_document(&mut self, lines: &[String]) -> Vec<String> {
        self.state.clear();
        let mut results = vec![String::new(); lines.len()];
        let statements = BlockParser::parse(lines);

        self.execute_statements(&statements, &mut results);

        results
    }

    fn execute_statements(&mut self, statements: &[Statement], results: &mut Vec<String>) {
        for stmt in statements {
            match stmt {
                Statement::Line { index, content } => {
                    let res = self.evaluate_single_line(content);
                    // Only update the result if it's not empty, otherwise we might overwrite
                    // results from previous statements on the same line (if using `;`)
                    if !res.is_empty() {
                        results[*index] = res;
                    }
                }
                Statement::IfBlock {
                    condition,
                    true_statements,
                    false_statements,
                } => {
                    let cond_val =
                        ExpressionService::evaluate(&self.state, condition).unwrap_or(0.0);

                    if cond_val.abs() >= f64::EPSILON {
                        self.execute_statements(true_statements, results);
                    } else if let Some(false_stmts) = false_statements {
                        self.execute_statements(false_stmts, results);
                    }
                }
            }
        }
    }

    fn evaluate_single_line(&mut self, line: &str) -> String {
        let line = line.trim();

        if line.is_empty() {
            return String::new();
        }

        if line.starts_with("//") {
            return String::new();
        }

        if line.chars().all(|c| matches!(c, '=' | '-' | '_' | '*')) {
            return String::new();
        }

        if let Some(position) = line.find('=') {
            let variable_name = line[..position].trim().to_string();

            if !variable_name.is_empty()
                && !line[position..].starts_with("==")
                && !variable_name.ends_with('<')
                && !variable_name.ends_with('>')
                && !variable_name.ends_with('!')
            {
                let expression = line[position + 1..].trim();

                return match ExpressionService::evaluate(&self.state, expression) {
                    Ok(value) => {
                        self.state.insert(variable_name, value);
                        Self::format_number(value)
                    }
                    Err(error) => format!("error: {}", error),
                };
            }
        }

        match ExpressionService::evaluate(&self.state, line) {
            Ok(value) => Self::format_number(value),
            Err(error) => format!("error: {}", error),
        }
    }

    fn format_number(value: f64) -> String {
        // Formatting function to use '.' as thousands separator and ',' as decimal separator
        let is_negative = value < 0.0;
        let abs_val = value.abs();

        // Guard: if value overflows u64, use scientific notation
        if abs_val >= u64::MAX as f64 {
            return format!("{:.6e}", value);
        }

        let integer_part = abs_val.trunc() as u64;
        let int_str = integer_part.to_string();

        // Add thousands separator (dot)
        let mut formatted_int = String::new();
        let chars: Vec<char> = int_str.chars().rev().collect();
        for (i, c) in chars.iter().enumerate() {
            if i > 0 && i % 3 == 0 {
                formatted_int.push('.');
            }
            formatted_int.push(*c);
        }
        formatted_int = formatted_int.chars().rev().collect();

        if is_negative {
            formatted_int.insert(0, '-');
        }

        // Check if fractional part is negligible (within epsilon threshold)
        if abs_val.fract().abs() < f64::EPSILON * abs_val.max(1.0) {
            formatted_int
        } else {
            // Format entire number first to handle rounding correctly
            let rounded_str = format!("{:.2}", abs_val);
            let parts: Vec<&str> = rounded_str.split('.').collect();

            let integer_formatted = if parts[0].len() > 3 {
                // Add thousands separator to the integer part
                let mut result = String::new();
                let reversed: Vec<char> = parts[0].chars().rev().collect();
                for (i, c) in reversed.iter().enumerate() {
                    if i > 0 && i % 3 == 0 {
                        result.push('.');
                    }
                    result.push(*c);
                }
                result.chars().rev().collect::<String>()
            } else {
                parts[0].to_string()
            };

            let result = format!("{},{}", integer_formatted, parts[1]);
            if is_negative {
                format!("-{}", result)
            } else {
                result
            }
        }
    }
}
