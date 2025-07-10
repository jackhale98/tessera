use crate::Result;
use std::path::Path;

pub fn ensure_directory<P: AsRef<Path>>(path: P) -> Result<()> {
    std::fs::create_dir_all(path)?;
    Ok(())
}

pub fn file_exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}

pub fn read_file_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    let content = std::fs::read_to_string(path)?;
    Ok(content)
}

pub fn write_string_to_file<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
    std::fs::write(path, content)?;
    Ok(())
}

pub fn format_ron_pretty<T: serde::Serialize + ?Sized>(data: &T) -> Result<String> {
    let config = ron::ser::PrettyConfig::default();
    let content = ron::ser::to_string_pretty(data, config)?;
    Ok(content)
}

pub fn parse_ron<T: serde::de::DeserializeOwned>(content: &str) -> Result<T> {
    let data: T = ron::from_str(content)?;
    Ok(data)
}

pub fn truncate_string(s: &str, max_length: usize) -> String {
    if s.len() <= max_length {
        s.to_string()
    } else {
        format!("{}...", &s[..max_length.saturating_sub(3)])
    }
}

pub fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}