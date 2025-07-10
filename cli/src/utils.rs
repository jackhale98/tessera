pub fn truncate_string(s: &str, max_length: usize) -> String {
    if s.len() <= max_length {
        s.to_string()
    } else {
        format!("{}...", &s[..max_length.saturating_sub(3)])
    }
}

pub fn format_percentage(value: f64) -> String {
    format!("{:.1}%", value * 100.0)
}

pub fn format_risk_score(score: f64) -> String {
    format!("{:.3}", score)
}

pub fn get_risk_level_color(score: f64) -> &'static str {
    if score >= 0.8 {
        "red"
    } else if score >= 0.6 {
        "yellow"
    } else if score >= 0.3 {
        "blue"
    } else {
        "green"
    }
}