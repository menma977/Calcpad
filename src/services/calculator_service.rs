use std::collections::HashMap;

/// Handles mathematical expression evaluation and variable storage.
pub struct CalculatorService {
    pub variables: HashMap<String, f64>,
}

impl CalculatorService {
    /// Creates a new instance with empty variable storage.
    pub fn new() -> Self {
        CalculatorService {
            variables: HashMap::new(),
        }
    }

    /// Evaluates a single line of input.
    /// Supports variable assignment (x = 10), expressions (1 + 2), and comments (//).
    pub fn evaluate_line(&mut self, line: &str) -> String {
        let line = line.trim();

        if line.is_empty() {
            return String::new();
        }

        if line.starts_with("//") {
            return String::new();
        }

        // Check for an if expression (statement style: if ... then x = ... endif)
        if line.starts_with("if ") {
            return match crate::services::condition_service::evaluate_if(
                line,
                |expr| self.evaluate_expression(expr),
            ) {
                Some(Ok(branch)) => {
                    if branch.is_empty() {
                        String::new()
                    } else {
                        self.evaluate_line(&branch)
                    }
                }
                Some(Err(error)) => format!("error: {}", error),
                None => String::new(),
            };
        }

        /* Variable assignment: detect '=' sign then store a result into a variable map */
        if let Some(position) = line.find('=') {
            let variable_name = line[..position].trim().to_string();
            let expression = line[position + 1..].trim();

            return match self.evaluate_expression(expression) {
                Ok(value) => {
                    self.variables.insert(variable_name, value);
                    format_number(value)
                }
                Err(error) => format!("error: {}", error),
            }
        }

        match self.evaluate_expression(line) {
            Ok(value) => format_number(value),
            Err(error) => format!("error: {}", error),
        }
    }

    fn evaluate_expression(&self, expression: &str) -> Result<f64, String> {
        let trimmed = expression.trim();

        // Check for expression-style if: if x > 20 then x - 1 else 0 endif
        if trimmed.starts_with("if ") {
            return match crate::services::condition_service::evaluate_if(
                trimmed,
                |expr| self.evaluate_expression(expr),
            ) {
                Some(Ok(branch)) => {
                    if branch.is_empty() {
                        Ok(0.0)
                    } else {
                        self.evaluate_expression(&branch)
                    }
                }
                Some(Err(error)) => Err(error),
                None => Err("invalid if expression".to_string()),
            };
        }

        let expression = self.replace_variables(trimmed)?;
        parse_expression(expression.trim())
    }

    fn replace_variables(&self, expression: &str) -> Result<String, String> {
        let mut result = expression.to_string();
        for (variable_name, value) in &self.variables {
            result = result.replace(variable_name.as_str(), &value.to_string());
        }
        Ok(result)
    }
}

/// Formats a number — removes decimal if whole number, 2 decimal places otherwise.
fn format_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{:.0}", value)
    } else {
        format!("{:.2}", value)
    }
}

/// Recursively parses and evaluates a mathematical expression string.
fn parse_expression(expression: &str) -> Result<f64, String> {
    let expression = expression.trim();

    if let Ok(number) = expression.parse::<f64>() {
        return Ok(number);
    }

    if let Some(result) = try_split_operator(expression, &['+', '-']) {
        return result;
    }

    if let Some(result) = try_split_operator(expression, &['*', '/']) {
        return result;
    }

    Err(format!("cannot parse: {}", expression))
}

/// Splits expression by operator and evaluates both sides recursively.
/// Scans right-to-left so left operators are evaluated first.
fn try_split_operator(expression: &str, operators: &[char]) -> Option<Result<f64, String>> {
    let chars: Vec<char> = expression.chars().collect();
    let mut depth = 0i32;

    let mut index = chars.len();
    while index > 0 {
        index -= 1;
        match chars[index] {
            ')' => depth += 1,
            '(' => depth -= 1,
            character if depth == 0 && operators.contains(&character) && index > 0 => {
                let left = &expression[..index];
                let right = &expression[index + 1..];

                let left_value = match parse_expression(left) {
                    Ok(value) => value,
                    Err(error) => return Some(Err(error)),
                };
                let right_value = match parse_expression(right) {
                    Ok(value) => value,
                    Err(error) => return Some(Err(error)),
                };

                return Some(match character {
                    '+' => Ok(left_value + right_value),
                    '-' => Ok(left_value - right_value),
                    '*' => Ok(left_value * right_value),
                    '/' => {
                        if right_value == 0.0 {
                            Err("division by zero".to_string())
                        } else {
                            Ok(left_value / right_value)
                        }
                    }
                    _ => Err("unknown operator".to_string()),
                });
            }
            _ => {}
        }
    }
    None
}