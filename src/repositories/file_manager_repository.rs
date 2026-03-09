use std::fs;
use std::path::Path;

pub fn save(lines: &[String], file_path: &str) -> Result<(), String> {
    let content = lines.join("\n");
    fs::write(file_path, content).map_err(|error| error.to_string())
}

pub fn load(file_path: &str) -> Result<Vec<String>, String> {
    if !Path::new(file_path).exists() {
        return Err(format!("file not found: {}", file_path));
    }
    let content = fs::read_to_string(file_path).map_err(|error| error.to_string())?;
    Ok(content.lines().map(|line| line.to_string()).collect())
}
