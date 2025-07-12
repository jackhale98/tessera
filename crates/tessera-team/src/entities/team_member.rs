use serde::{Deserialize, Serialize};
use tessera_core::{Entity, Id, Result};
use chrono::{DateTime, NaiveDate, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub id: Id,
    pub employee_id: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub job_title: String,
    pub department: String,
    
    // Contact Information
    pub phone: Option<String>,
    pub office_location: Option<String>,
    pub time_zone: Option<String>,
    
    // System Integration
    pub git_username: Option<String>,
    pub slack_username: Option<String>,
    pub active: bool,
    
    // Role Assignments
    pub primary_role: Id,
    pub additional_roles: Vec<Id>,
    pub team_memberships: Vec<Id>,
    
    // Metadata
    pub hire_date: Option<NaiveDate>,
    pub manager_id: Option<Id>,
    pub metadata: HashMap<String, String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

impl TeamMember {
    pub fn new(
        first_name: String,
        last_name: String,
        email: String,
        job_title: String,
        department: String,
        primary_role: Id,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            employee_id: None,
            first_name,
            last_name,
            email,
            job_title,
            department,
            phone: None,
            office_location: None,
            time_zone: None,
            git_username: None,
            slack_username: None,
            active: true,
            primary_role,
            additional_roles: Vec::new(),
            team_memberships: Vec::new(),
            hire_date: None,
            manager_id: None,
            metadata: HashMap::new(),
            created: now,
            updated: now,
        }
    }

    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    pub fn display_name(&self) -> String {
        if let Some(git_username) = &self.git_username {
            format!("{} (@{})", self.full_name(), git_username)
        } else {
            self.full_name()
        }
    }

    pub fn deactivate(&mut self) {
        self.active = false;
        self.updated = Utc::now();
    }

    pub fn update_contact_info(
        &mut self,
        phone: Option<String>,
        office_location: Option<String>,
        time_zone: Option<String>,
    ) {
        self.phone = phone;
        self.office_location = office_location;
        self.time_zone = time_zone;
        self.updated = Utc::now();
    }

    pub fn update_system_integration(
        &mut self,
        git_username: Option<String>,
        slack_username: Option<String>,
    ) {
        self.git_username = git_username;
        self.slack_username = slack_username;
        self.updated = Utc::now();
    }

    pub fn add_role(&mut self, role_id: Id) {
        if !self.additional_roles.contains(&role_id) && self.primary_role != role_id {
            self.additional_roles.push(role_id);
            self.updated = Utc::now();
        }
    }

    pub fn remove_role(&mut self, role_id: &Id) {
        self.additional_roles.retain(|r| r != role_id);
        self.updated = Utc::now();
    }

    pub fn add_team_membership(&mut self, team_id: Id) {
        if !self.team_memberships.contains(&team_id) {
            self.team_memberships.push(team_id);
            self.updated = Utc::now();
        }
    }

    pub fn remove_team_membership(&mut self, team_id: &Id) {
        self.team_memberships.retain(|t| t != team_id);
        self.updated = Utc::now();
    }

    pub fn all_roles(&self) -> Vec<Id> {
        let mut roles = vec![self.primary_role];
        roles.extend(self.additional_roles.clone());
        roles
    }
}

impl Entity for TeamMember {
    fn id(&self) -> Id {
        self.id
    }

    fn name(&self) -> &str {
        &self.first_name
    }

    fn validate(&self) -> Result<()> {
        if self.first_name.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "First name cannot be empty".to_string()
            ));
        }
        if self.last_name.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Last name cannot be empty".to_string()
            ));
        }
        if self.email.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Email cannot be empty".to_string()
            ));
        }
        if self.job_title.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Job title cannot be empty".to_string()
            ));
        }
        if self.department.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Department cannot be empty".to_string()
            ));
        }
        
        Ok(())
    }
}