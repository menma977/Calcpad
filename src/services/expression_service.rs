use crate::enums::operator::Operator;
use crate::services::state_service::StateService;

pub struct ExpressionService;

impl ExpressionService {
    pub fn is_truthy(val: f64) -> bool {
        val.abs() >= f64::EPSILON
    }

    pub fn evaluate(state: &StateService, expression: &str) -> Result<f64, String> {
        let trimmed = expression.trim();
        let replaced = state.replace_variables(trimmed)?;
        Self::parse_expression(replaced.trim())
    }

    fn parse_expression(expression: &str) -> Result<f64, String> {
        let expression = expression.trim();
        if expression.is_empty() {
            return Ok(0.0);
        }

        // Handle parentheses
        if expression.starts_with('(') && expression.ends_with(')') {
            let mut depth = 0;
            let mut valid_outer = true;
            let char_count = expression.chars().count();
            for (_, c) in expression.char_indices().take(char_count - 1) {
                match c {
                    '(' => depth += 1,
                    ')' => depth -= 1,
                    _ => {}
                }
                if depth == 0 {
                    valid_outer = false;
                    break;
                }
            }
            if valid_outer {
                // Safe: '(' and ')' are always ASCII (1 byte each), so byte-index boundaries are correct
                return Self::parse_expression(&expression[1..expression.len() - 1]);
            }
        }

        if let Ok(number) = expression.parse::<f64>() {
            return Ok(number);
        }

        // Operator precedence groups (lowest to highest)
        let op_groups = [
            vec![Operator::Or],
            vec![Operator::And],
            vec![Operator::BitOr],
            vec![Operator::BitXor],
            vec![Operator::BitAnd],
            vec![Operator::Equal, Operator::NotEqual],
            vec![
                Operator::GreaterEqual,
                Operator::LessEqual,
                Operator::GreaterThan,
                Operator::LessThan,
            ],
            vec![Operator::ShiftLeft, Operator::ShiftRight],
            vec![Operator::Add, Operator::Subtract],
            vec![Operator::Multiply, Operator::Divide, Operator::Modulo],
        ];

        if let Some(result) = Self::try_split_ternary(expression) {
            return result;
        }

        for group in &op_groups {
            if let Some(result) = Self::try_split_operator(expression, group) {
                return result;
            }
        }

        Err(format!("cannot parse: {}", expression))
    }

    fn try_split_ternary(expression: &str) -> Option<Result<f64, String>> {
        let mut depth = 0i32;
        for (i, c) in expression.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => depth -= 1,
                '?' if depth == 0 => {
                    let condition_str = &expression[..i];
                    let mut inner_ternary = 0;
                    let mut colon_idx = None;
                    let mut temp_depth = 0;

                    for (idx, cur_c) in expression[i + 1..].char_indices() {
                        match cur_c {
                            '(' => temp_depth += 1,
                            ')' => temp_depth -= 1,
                            '?' if temp_depth == 0 => inner_ternary += 1,
                            ':' if temp_depth == 0 => {
                                if inner_ternary == 0 {
                                    colon_idx = Some(i + 1 + idx);
                                    break;
                                } else {
                                    inner_ternary -= 1;
                                }
                            }
                            _ => {}
                        }
                    }

                    return if let Some(c_idx) = colon_idx {
                        let true_branch = &expression[i + 1..c_idx];
                        let false_branch = &expression[c_idx + 1..];

                        match Self::parse_expression(condition_str) {
                            Ok(cond_val) => {
                                if Self::is_truthy(cond_val) {
                                    Some(Self::parse_expression(true_branch))
                                } else {
                                    Some(Self::parse_expression(false_branch))
                                }
                            }
                            Err(e) => Some(Err(e)),
                        }
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
        let mut depth = 0i32;
        let chars: Vec<(usize, char)> = expression.char_indices().collect();
        let mut i = chars.len();

        while i > 0 {
            i -= 1;
            let (byte_idx, c) = chars[i];
            match c {
                ')' => depth += 1,
                '(' => depth -= 1,
                _ if depth == 0 => {
                    for op in operators {
                        let op_str: &'static str = (*op).into();
                        if expression[byte_idx..].starts_with(op_str) {
                            // Reject if this single-char operator is a prefix of a longer operator
                            if op_str == ">" && expression[byte_idx..].starts_with(">>") {
                                continue;
                            }
                            if op_str == "<" && expression[byte_idx..].starts_with("<<") {
                                continue;
                            }

                            if byte_idx == 0 && (*op == Operator::Add || *op == Operator::Subtract)
                            {
                                continue;
                            }

                            let left = &expression[..byte_idx];
                            let right = &expression[byte_idx + op_str.len()..];

                            let left_val = match Self::parse_expression(left) {
                                Ok(v) => v,
                                Err(e) => return Some(Err(e)),
                            };
                            let right_val = match Self::parse_expression(right) {
                                Ok(v) => v,
                                Err(e) => return Some(Err(e)),
                            };

                            return Some(Self::apply_operator(*op, left_val, right_val));
                        }
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn apply_operator(op: Operator, left: f64, right: f64) -> Result<f64, String> {
        match op {
            Operator::Add => Ok(left + right),
            Operator::Subtract => Ok(left - right),
            Operator::Multiply => Ok(left * right),
            Operator::Divide => {
                if right == 0.0 {
                    Err("division by zero".to_string())
                } else {
                    Ok(left / right)
                }
            }
            Operator::Modulo => {
                if right == 0.0 {
                    Err("modulo by zero".to_string())
                } else {
                    Ok(left % right)
                }
            }
            Operator::Equal => Ok(if (left - right).abs() < f64::EPSILON {
                1.0
            } else {
                0.0
            }),
            Operator::NotEqual => Ok(if (left - right).abs() >= f64::EPSILON {
                1.0
            } else {
                0.0
            }),
            Operator::GreaterEqual => Ok(if left >= right { 1.0 } else { 0.0 }),
            Operator::LessEqual => Ok(if left <= right { 1.0 } else { 0.0 }),
            Operator::GreaterThan => Ok(if left > right { 1.0 } else { 0.0 }),
            Operator::LessThan => Ok(if left < right { 1.0 } else { 0.0 }),
            Operator::And => Ok(if left != 0.0 && right != 0.0 {
                1.0
            } else {
                0.0
            }),
            Operator::Or => Ok(if left != 0.0 || right != 0.0 {
                1.0
            } else {
                0.0
            }),
            Operator::BitAnd => {
                let left_i64 = Self::safe_to_i64(left)?;
                let right_i64 = Self::safe_to_i64(right)?;
                Ok((left_i64 & right_i64) as f64)
            }
            Operator::BitOr => {
                let left_i64 = Self::safe_to_i64(left)?;
                let right_i64 = Self::safe_to_i64(right)?;
                Ok((left_i64 | right_i64) as f64)
            }
            Operator::BitXor => {
                let left_i64 = Self::safe_to_i64(left)?;
                let right_i64 = Self::safe_to_i64(right)?;
                Ok((left_i64 ^ right_i64) as f64)
            }
            Operator::ShiftLeft => {
                let left_i64 = Self::safe_to_i64(left)?;
                let shift_amount = Self::safe_shift_amount(right)?;
                Ok(left_i64.wrapping_shl(shift_amount) as f64)
            }
            Operator::ShiftRight => {
                let left_i64 = Self::safe_to_i64(left)?;
                let shift_amount = Self::safe_shift_amount(right)?;
                Ok(left_i64.wrapping_shr(shift_amount) as f64)
            }
        }
    }

    fn safe_to_i64(val: f64) -> Result<i64, String> {
        if !val.is_finite() {
            Err(format!("value must be finite, got {}", val))
        } else if val > i64::MAX as f64 || val < i64::MIN as f64 {
            Err(format!(
                "value {} is out of range for bitwise operation (must be between {} and {})",
                val,
                i64::MIN,
                i64::MAX
            ))
        } else {
            Ok(val as i64)
        }
    }

    fn safe_shift_amount(val: f64) -> Result<u32, String> {
        if !val.is_finite() {
            Err(format!("shift amount must be finite, got {}", val))
        } else if val < 0.0 || val > 63.0 {
            Err(format!(
                "shift amount {} is out of range (must be 0..=63)",
                val
            ))
        } else {
            Ok(val as u32)
        }
    }
}
