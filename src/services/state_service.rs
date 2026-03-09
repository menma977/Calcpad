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

        let mut sorted_vars: Vec<(&String, &f64)> = self.variables.iter().collect();
        sorted_vars.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        for (variable_name, value) in sorted_vars {
            result = result.replace(variable_name.as_str(), &value.to_string());
        }
        Ok(result)
    }
}
