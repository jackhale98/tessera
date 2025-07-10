use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    pub hourly_rate: f32,
    #[serde(default = "default_capacity")]
    pub capacity: f32, // 0.0 to 1.0, where 1.0 is full time
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

fn default_capacity() -> f32 {
    1.0
}

impl Resource {
    pub fn new(name: String, hourly_rate: f32) -> Self {
        Self {
            name,
            role: None,
            hourly_rate,
            capacity: 1.0,
            skills: Vec::new(),
            email: None,
        }
    }

    pub fn with_skills(mut self, skills: Vec<String>) -> Self {
        self.skills = skills;
        self
    }

    pub fn has_skill(&self, skill: &str) -> bool {
        self.skills.iter().any(|s| s == skill)
    }

    pub fn daily_hours(&self) -> f32 {
        8.0 * self.capacity
    }

    pub fn weekly_hours(&self) -> f32 {
        40.0 * self.capacity
    }
}
