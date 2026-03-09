use crate::enums::operator::Operator;
use crate::services::state_service::StateService;

pub struct ExpressionService;

impl ExpressionService {
    pub fn evaluate(state: &StateService, expression: &str) -> Result<f64, String> {
        let trimmed = expression.trim();
        let replaced = state.replace_variables(trimmed)?;
        Self::parse_expression(replaced.trim())
    }

    fn parse_expression(expression: &str) -> Result<f64, String> {
        let expression = expression.trim();

        if expression.starts_with('(') && expression.ends_with(')') {
            let mut depth = 0;
            let mut valid_outer = true;
            let chars: Vec<char> = expression.chars().collect();

            for i in 0..chars.len() - 1 {
                match chars[i] {
                    '(' => depth += 1,
                    ')' => depth -= 1,
                    _ => {}
                }
                if depth == 0 && i < chars.len() - 1 {
                    valid_outer = false;
                    break;
                }
            }

            if valid_outer {
                return Self::parse_expression(&expression[1..expression.len() - 1]);
            }
        }

        if let Ok(number) = expression.parse::<f64>() {
            return Ok(number);
        }

        if let Some(result) = Self::try_split_ternary(expression) {
            return result;
        }

        if let Some(result) = Self::try_split_operator(expression, &[Operator::Or, Operator::And]) {
            return result;
        }

        if let Some(result) = Self::try_split_operator(
            expression,
            &[Operator::BitOr, Operator::BitXor, Operator::BitAnd],
        ) {
            return result;
        }

        if let Some(result) = Self::try_split_operator(
            expression,
            &[
                Operator::Equal,
                Operator::NotEqual,
                Operator::GreaterEqual,
                Operator::LessEqual,
                Operator::GreaterThan,
                Operator::LessThan,
            ],
        ) {
            return result;
        }

        if let Some(result) = Self::try_split_operator(
            expression,
            &[Operator::ShiftLeft, Operator::ShiftRight],
        ) {
            return result;
        }

        if let Some(result) = Self::try_split_operator(
            expression,
            &[Operator::Add, Operator::Subtract],
        ) {
            return result;
        }

        if let Some(result) = Self::try_split_operator(
            expression,
            &[Operator::Multiply, Operator::Divide, Operator::Modulo],
        ) {
            return result;
        }

        Err(format!("cannot parse: {}", expression))
    }

    fn try_split_ternary(expression: &str) -> Option<Result<f64, String>> {
        let mut depth = 0i32;
        let chars: Vec<char> = expression.chars().collect();

        for i in 0..chars.len() {
            match chars[i] {
                '(' => depth += 1,
                ')' => depth -= 1,
                '?' if depth == 0 => {
                    let condition_str = chars[..i].iter().collect::<String>();

                    let mut j = i + 1;
                    let mut inner_ternary = 0;
                    let mut colon_idx = None;
                    let mut temp_depth = 0;

                    while j < chars.len() {
                        match chars[j] {
                            '(' => temp_depth += 1,
                            ')' => temp_depth -= 1,
                            '?' if temp_depth == 0 => inner_ternary += 1,
                            ':' if temp_depth == 0 => {
                                if inner_ternary == 0 {
                                    colon_idx = Some(j);
                                    break;
                                } else {
                                    inner_ternary -= 1;
                                }
                            }
                            _ => {}
                        }
                        j += 1;
                    }

                    return if let Some(c_idx) = colon_idx {
                        let true_branch = chars[i + 1..c_idx].iter().collect::<String>();
                        let false_branch = chars[c_idx + 1..].iter().collect::<String>();

                        let condition_val = match Self::parse_expression(&condition_str) {
                            Ok(v) => v,
                            Err(e) => return Some(Err(e)),
                        };

                        Some(if condition_val.abs() >= f64::EPSILON {
                            Self::parse_expression(&true_branch)
                        } else {
                            Self::parse_expression(&false_branch)
                        })
                    } else {
                        Some(Err("missing ':' in ternary expression".to_string()))
                    };
                }
                _ => {}
            }
        }
        None
    }

    fn try_split_operator(expression: &str, operators: &[Operator]) -> Option<Result<f64, String>> {
        let chars: Vec<char> = expression.chars().collect();
        let mut depth = 0i32;

        let mut index = chars.len();
        while index > 0 {
            index -= 1;
            match chars[index] {
                ')' => depth += 1,
                '(' => depth -= 1,
                _ => {
                    if depth == 0 {
                        for op in operators {
                            let op_str: &'static str = (*op).into();
                            let op_chars: Vec<char> = op_str.chars().collect();
                            let op_len = op_chars.len();

                            if index + op_len <= chars.len()
                                && &chars[index..index + op_len] == op_chars.as_slice()
                            {
                                if index == 0
                                    && (*op == Operator::Add || *op == Operator::Subtract)
                                {
                                    continue;
                                }

                                let left = chars[..index].iter().collect::<String>();
                                let right = chars[index + op_len..].iter().collect::<String>();

                                let left_value = match Self::parse_expression(&left) {
                                    Ok(value) => value,
                                    Err(error) => return Some(Err(error)),
                                };
                                let right_value = match Self::parse_expression(&right) {
                                    Ok(value) => value,
                                    Err(error) => return Some(Err(error)),
                                };

                                return Some(match op {
                                    Operator::Add => Ok(left_value + right_value),
                                    Operator::Subtract => Ok(left_value - right_value),
                                    Operator::Multiply => Ok(left_value * right_value),
                                    Operator::Modulo => {
                                        if right_value == 0.0 {
                                            Err("modulo by zero".to_string())
                                        } else {
                                            Ok(left_value % right_value)
                                        }
                                    }
                                    Operator::Divide => {
                                        if right_value == 0.0 {
                                            Err("division by zero".to_string())
                                        } else {
                                            Ok(left_value / right_value)
                                        }
                                    }
                                    Operator::Equal => Ok(if (left_value - right_value).abs() < f64::EPSILON { 1.0 } else { 0.0 }),
                                    Operator::NotEqual => Ok(if (left_value - right_value).abs() >= f64::EPSILON { 1.0 } else { 0.0 }),
                                    Operator::GreaterEqual => Ok(if left_value >= right_value { 1.0 } else { 0.0 }),
                                    Operator::LessEqual => Ok(if left_value <= right_value { 1.0 } else { 0.0 }),
                                    Operator::GreaterThan => Ok(if left_value > right_value { 1.0 } else { 0.0 }),
                                    Operator::LessThan => Ok(if left_value < right_value { 1.0 } else { 0.0 }),
                                    Operator::And => Ok(if left_value != 0.0 && right_value != 0.0 { 1.0 } else { 0.0 }),
                                    Operator::Or => Ok(if left_value != 0.0 || right_value != 0.0 { 1.0 } else { 0.0 }),
                                    Operator::BitAnd => Ok((left_value as i64 & right_value as i64) as f64),
                                    Operator::BitOr => Ok((left_value as i64 | right_value as i64) as f64),
                                    Operator::BitXor => Ok((left_value as i64 ^ right_value as i64) as f64),
                                    Operator::ShiftLeft => Ok((left_value as i64).wrapping_shl(right_value as u32) as f64),
                                    Operator::ShiftRight => Ok((left_value as i64).wrapping_shr(right_value as u32) as f64),
                                });
                            }
                        }
                    }
                }
            }
        }
        None
    }
}
