use std::collections::HashMap;

pub struct StateService {
    pub variables: HashMap<String, f64>,
}

impl StateService {
    pub fn new() -> Self {
        StateService {
            variables: HashMap::new(),
        }
    }
}

impl Default for StateService {
    fn default() -> Self {
        Self::new()
    }
}

impl StateService {
    pub fn insert(&mut self, name: String, value: f64) {
        self.variables.insert(name, value);
    }

    pub fn clear(&mut self) {
        self.variables.clear();
    }

    pub fn get_variable_names(&self) -> Vec<String> {
        self.variables.keys().cloned().collect()
    }

    pub fn replace_variables(&self, expression: &str) -> Result<String, String> {
        let mut result = expression.to_string();

        // Sort variables by length descending (longer names first)
        let mut sorted_vars: Vec<(&String, &f64)> = self.variables.iter().collect();
        sorted_vars.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        for (variable_name, value) in sorted_vars {
            // Reject non-finite values early
            if !value.is_finite() {
                return Err(format!(
                    "variable '{}' has non-finite value ({})",
                    variable_name, value
                ));
            }
            result = Self::replace_variable_token(&result, variable_name, &value.to_string());
        }
        Ok(result)
    }

    fn replace_variable_token(expr: &str, var_name: &str, var_value: &str) -> String {
        let mut result = String::new();
        let expr_chars: Vec<char> = expr.chars().collect();
        let var_chars: Vec<char> = var_name.chars().collect();
        let mut i = 0;

        while i < expr_chars.len() {
            // Try to match the variable name at the current position
            let can_match = i + var_chars.len() <= expr_chars.len()
                && expr_chars[i..i + var_chars.len()] == var_chars[..];

            if can_match {
                // Check the word boundary before
                let before_valid = i == 0 || {
                    let prev_char = expr_chars[i - 1];
                    !prev_char.is_alphanumeric() && prev_char != '_'
                };

                // Check the word boundary after
                let after_valid = i + var_chars.len() == expr_chars.len() || {
                    let next_char = expr_chars[i + var_chars.len()];
                    !next_char.is_alphanumeric() && next_char != '_'
                };

                if before_valid && after_valid {
                    // Match found, replace it
                    result.push_str(var_value);
                    i += var_chars.len();
                    continue;
                }
            }

            // No match, copy character
            result.push(expr_chars[i]);
            i += 1;
        }

        result
    }
}
