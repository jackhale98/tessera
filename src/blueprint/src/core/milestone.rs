use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub name: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Milestone {
    pub fn new(name: String) -> Self {
        Self {
            name,
            dependencies: Vec::new(),
            target_date: None,
            description: None,
        }
    }

    pub fn with_dependencies(mut self, deps: Vec<String>) -> Self {
        self.dependencies = deps;
        self
    }

    pub fn with_target_date(mut self, date: NaiveDate) -> Self {
        self.target_date = Some(date);
        self
    }
}
