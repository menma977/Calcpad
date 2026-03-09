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
                    let cond_val = ExpressionService::evaluate(&self.state, condition).unwrap_or(0.0);
                    
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
        if value.fract() == 0.0 {
            format!("{:.0}", value)
        } else {
            format!("{:.2}", value)
        }
    }
}
