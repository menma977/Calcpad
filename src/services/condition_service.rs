/// Evaluates an if expression and returns the result as a string.
/// Supports two styles:
/// - Expression: `if x > 20 then x - 1 else 0 endif`
/// - Statement:  `if x > 20 then x = x - 1 else x = 0 endif`
pub fn evaluate_if(
    expression: &str,
    evaluate_expr: impl Fn(&str) -> Result<f64, String>,
) -> Option<Result<String, String>> {
    let trimmed = expression.trim();

    if !trimmed.starts_with("if ") {
        return None;
    }

    let without_if = &trimmed[3..];

    let then_pos = without_if.find(" then ")?;
    let condition_str = without_if[..then_pos].trim();
    let after_then = without_if[then_pos + 6..].trim();

    let (true_branch, false_branch) = if let Some(else_pos) = after_then.find(" else ") {
        let true_part = after_then[..else_pos].trim();
        let false_part = after_then[else_pos + 6..].trim();
        let false_part = false_part.trim_end_matches("endif").trim();
        (true_part, Some(false_part))
    } else {
        let true_part = after_then.trim_end_matches("endif").trim();
        (true_part, None)
    };

    let condition_result = match evaluate_condition(condition_str, &evaluate_expr) {
        Ok(result) => result,
        Err(error) => return Some(Err(error)),
    };

    let branch = if condition_result {
        true_branch
    } else {
        false_branch.unwrap_or("")
    };

    if branch.is_empty() {
        return Some(Ok(String::new()));
    }

    Some(Ok(branch.to_string()))
}

/// Parses and evaluates a condition like `x > 20` or `total == 100`.
fn evaluate_condition(
    condition: &str,
    evaluate_expr: &impl Fn(&str) -> Result<f64, String>,
) -> Result<bool, String> {
    let operators = [">=", "<=", "!=", ">", "<", "=="];

    for op in operators {
        if let Some(pos) = condition.find(op) {
            let left = condition[..pos].trim();
            let right = condition[pos + op.len()..].trim();

            let left_val = evaluate_expr(left)?;
            let right_val = evaluate_expr(right)?;

            return Ok(match op {
                ">" => left_val > right_val,
                "<" => left_val < right_val,
                ">=" => left_val >= right_val,
                "<=" => left_val <= right_val,
                "==" => (left_val - right_val).abs() < f64::EPSILON,
                "!=" => (left_val - right_val).abs() >= f64::EPSILON,
                _ => return Err("unknown operator".to_string()),
            });
        }
    }

    Err(format!("cannot parse condition: {}", condition))
}