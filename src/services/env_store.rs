use std::fs;

const SYSTEM_PROMPT_FILE: &str = "system_prompt.txt";

pub fn read_system_prompt() -> String {
    fs::read_to_string(SYSTEM_PROMPT_FILE).unwrap_or_default()
}

pub fn write_system_prompt(prompt: &str) -> std::io::Result<()> {
    fs::write(SYSTEM_PROMPT_FILE, prompt)
}

pub fn read_env_value(key: &str) -> Option<String> {
    let content = fs::read_to_string(".env").ok()?;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=')
            && k.trim() == key
        {
            return Some(v.trim().to_string());
        }
    }
    None
}

pub fn write_env_file(values: &[(&str, &str)]) -> std::io::Result<()> {
    let mut content = String::new();
    for (k, v) in values {
        content.push_str(&format!("{}={}\n", k, v));
    }
    fs::write(".env", content)
}
