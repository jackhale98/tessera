use tessera_core::{Id, Entity, Repository, Result, DesignTrackError, format_ron_pretty};
use chrono::{Datelike, NaiveDate, Weekday, DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Calendar {
    pub id: Id,
    pub name: String,
    pub description: Option<String>,
    pub holidays: Vec<Holiday>,
    pub working_days: Vec<Weekday>,
    pub working_hours: WorkingHours,
    pub exceptions: Vec<CalendarException>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Holiday {
    pub id: Id,
    pub name: String,
    pub date: NaiveDate,
    pub description: Option<String>,
    pub recurring: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingHours {
    pub start_time: u8, // Hour of day (0-23)
    pub end_time: u8,   // Hour of day (0-23)
    pub daily_hours: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarException {
    pub id: Id,
    pub date: NaiveDate,
    pub exception_type: ExceptionType,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExceptionType {
    Working,    // Working day that would normally be non-working
    NonWorking, // Non-working day that would normally be working
    HalfDay,    // Half working day
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCalendar {
    pub id: Id,
    pub resource_id: Id,
    pub calendar_id: Id,
    pub overrides: Vec<CalendarException>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

impl Entity for Calendar {
    fn id(&self) -> Id {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(DesignTrackError::Validation("Calendar name cannot be empty".to_string()));
        }
        if self.working_hours.start_time >= 24 || self.working_hours.end_time >= 24 {
            return Err(DesignTrackError::Validation("Working hours must be between 0-23".to_string()));
        }
        if self.working_hours.start_time >= self.working_hours.end_time {
            return Err(DesignTrackError::Validation("Start time must be before end time".to_string()));
        }
        if self.working_hours.daily_hours <= 0.0 || self.working_hours.daily_hours > 24.0 {
            return Err(DesignTrackError::Validation("Daily hours must be between 0-24".to_string()));
        }
        Ok(())
    }
}

impl Entity for ResourceCalendar {
    fn id(&self) -> Id {
        self.id
    }

    fn name(&self) -> &str {
        "Resource Calendar" // Resource calendars don't have names
    }

    fn validate(&self) -> Result<()> {
        // Basic validation - could add checks for valid resource_id and calendar_id
        Ok(())
    }
}

impl Default for WorkingHours {
    fn default() -> Self {
        Self {
            start_time: 9,
            end_time: 17,
            daily_hours: 8.0,
        }
    }
}

impl Default for Calendar {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            name: "Default".to_string(),
            description: None,
            holidays: Vec::new(),
            working_days: vec![
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
            ],
            working_hours: WorkingHours::default(),
            exceptions: Vec::new(),
            created: now,
            updated: now,
            metadata: HashMap::new(),
        }
    }
}

impl Calendar {
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            name,
            description: None,
            holidays: Vec::new(),
            working_days: vec![
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
            ],
            working_hours: WorkingHours::default(),
            exceptions: Vec::new(),
            created: now,
            updated: now,
            metadata: HashMap::new(),
        }
    }

    pub fn with_holidays(mut self, holidays: Vec<Holiday>) -> Self {
        self.holidays = holidays;
        self.updated = Utc::now();
        self
    }

    pub fn with_working_hours(mut self, working_hours: WorkingHours) -> Self {
        self.working_hours = working_hours;
        self.updated = Utc::now();
        self
    }

    pub fn add_holiday(&mut self, holiday: Holiday) {
        self.holidays.push(holiday);
        self.updated = Utc::now();
    }

    pub fn add_exception(&mut self, exception: CalendarException) {
        self.exceptions.push(exception);
        self.updated = Utc::now();
    }

    pub fn is_working_day(&self, date: NaiveDate) -> bool {
        // Check if it's a standard working day
        let is_standard_working_day = self.working_days.contains(&date.weekday());
        
        // Check for exceptions
        for exception in &self.exceptions {
            if exception.date == date {
                return match exception.exception_type {
                    ExceptionType::Working => true,
                    ExceptionType::NonWorking => false,
                    ExceptionType::HalfDay => true, // Still considered working
                };
            }
        }
        
        // Check for holidays
        if self.is_holiday(date) {
            return false;
        }
        
        is_standard_working_day
    }

    pub fn is_holiday(&self, date: NaiveDate) -> bool {
        self.holidays.iter().any(|holiday| {
            if holiday.recurring {
                // For recurring holidays, check month and day
                holiday.date.month() == date.month() && holiday.date.day() == date.day()
            } else {
                holiday.date == date
            }
        })
    }

    pub fn get_working_hours(&self, date: NaiveDate) -> f32 {
        if !self.is_working_day(date) {
            return 0.0;
        }
        
        // Check for half-day exceptions
        for exception in &self.exceptions {
            if exception.date == date {
                if matches!(exception.exception_type, ExceptionType::HalfDay) {
                    return self.working_hours.daily_hours / 2.0;
                }
            }
        }
        
        self.working_hours.daily_hours
    }

    pub fn add_working_days(&self, start_date: NaiveDate, working_days: f64) -> NaiveDate {
        if working_days <= 0.0 {
            return start_date;
        }
        
        let mut current = start_date;
        let mut remaining_days = working_days;
        
        // Add safety limit to prevent infinite loops
        let max_iterations = (working_days * 10.0) as i32 + 365;
        let mut iterations = 0;
        
        // Start from the next day, as start_date is day 0
        while remaining_days > 0.001 && iterations < max_iterations {
            // Move to next day first
            if let Some(next_date) = current.succ_opt() {
                current = next_date;
            } else {
                break; // Prevent infinite loop on date overflow
            }
            
            // Check if this day is a working day
            if self.is_working_day(current) {
                let daily_hours = self.get_working_hours(current);
                if daily_hours > 0.0 {
                    let working_day_fraction = daily_hours as f64 / self.working_hours.daily_hours as f64;
                    remaining_days -= working_day_fraction;
                    
                    // If we've completed the required working days, we're done
                    if remaining_days <= 0.001 {
                        return current;
                    }
                }
            }
            
            iterations += 1;
        }
        
        current
    }

    pub fn subtract_working_days(&self, start_date: NaiveDate, working_days: f64) -> NaiveDate {
        if working_days <= 0.0 {
            return start_date;
        }
        
        let mut current = start_date;
        let mut remaining_days = working_days;
        
        // Add safety limit to prevent infinite loops
        let max_iterations = (working_days * 10.0) as i32 + 365;
        let mut iterations = 0;
        
        // Start from the previous day, as start_date is day 0
        while remaining_days > 0.001 && iterations < max_iterations {
            // Move to previous day first
            if let Some(prev_date) = current.pred_opt() {
                current = prev_date;
            } else {
                break; // Prevent infinite loop on date underflow
            }
            
            // Check if this day is a working day
            if self.is_working_day(current) {
                let daily_hours = self.get_working_hours(current);
                if daily_hours > 0.0 {
                    let working_day_fraction = daily_hours as f64 / self.working_hours.daily_hours as f64;
                    remaining_days -= working_day_fraction;
                    
                    // If we've completed the required working days, we're done
                    if remaining_days <= 0.001 {
                        return current;
                    }
                }
            }
            
            iterations += 1;
        }
        
        current
    }

    pub fn next_working_day(&self, date: NaiveDate) -> NaiveDate {
        let mut next = date;
        let mut iterations = 0;
        const MAX_ITERATIONS: i32 = 3650; // Maximum 10 years to prevent infinite loops
        
        loop {
            if let Some(next_date) = next.succ_opt() {
                next = next_date;
                iterations += 1;
                
                if iterations > MAX_ITERATIONS {
                    // Return original date if we can't find a next working day
                    return date;
                }
                
                if self.is_working_day(next) {
                    return next;
                }
            } else {
                // If we can't find a next working day, return the original date
                return date;
            }
        }
    }

    pub fn previous_working_day(&self, date: NaiveDate) -> NaiveDate {
        let mut prev = date;
        let mut iterations = 0;
        const MAX_ITERATIONS: i32 = 3650;
        
        loop {
            if let Some(prev_date) = prev.pred_opt() {
                prev = prev_date;
                iterations += 1;
                
                if iterations > MAX_ITERATIONS {
                    return date;
                }
                
                if self.is_working_day(prev) {
                    return prev;
                }
            } else {
                // If we can't find a previous working day, return the original date
                return date;
            }
        }
    }

    pub fn working_days_between(&self, start: NaiveDate, end: NaiveDate) -> f32 {
        let mut current = start;
        let mut total_days = 0.0;
        
        while current <= end {
            if self.is_working_day(current) {
                total_days += self.get_working_hours(current) / self.working_hours.daily_hours;
            }
            if let Some(next_date) = current.succ_opt() {
                current = next_date;
            } else {
                break;
            }
        }
        
        total_days
    }

    pub fn get_standard_us_holidays() -> Vec<Holiday> {
        let base_year = 2024;
        vec![
            Holiday::new_recurring("New Year's Day".to_string(), NaiveDate::from_ymd_opt(base_year, 1, 1).unwrap()),
            Holiday::new_recurring("Independence Day".to_string(), NaiveDate::from_ymd_opt(base_year, 7, 4).unwrap()),
            Holiday::new_recurring("Christmas Day".to_string(), NaiveDate::from_ymd_opt(base_year, 12, 25).unwrap()),
            // Note: Memorial Day, Labor Day, Thanksgiving are complex to calculate as they're relative dates
        ]
    }
}

impl Holiday {
    pub fn new(name: String, date: NaiveDate) -> Self {
        Self {
            id: Id::new(),
            name,
            date,
            description: None,
            recurring: false,
        }
    }

    pub fn new_recurring(name: String, date: NaiveDate) -> Self {
        Self {
            id: Id::new(),
            name,
            date,
            description: None,
            recurring: true,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

impl ResourceCalendar {
    pub fn new(resource_id: Id, calendar_id: Id) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            resource_id,
            calendar_id,
            overrides: Vec::new(),
            created: now,
            updated: now,
            metadata: HashMap::new(),
        }
    }

    pub fn with_overrides(mut self, overrides: Vec<CalendarException>) -> Self {
        self.overrides = overrides;
        self.updated = Utc::now();
        self
    }

    pub fn add_override(&mut self, exception: CalendarException) {
        self.overrides.push(exception);
        self.updated = Utc::now();
    }
}

impl CalendarException {
    pub fn new(date: NaiveDate, exception_type: ExceptionType) -> Self {
        Self {
            id: Id::new(),
            date,
            exception_type,
            description: None,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarManager {
    pub calendars: HashMap<Id, Calendar>,
    pub resource_calendars: HashMap<Id, ResourceCalendar>,
    pub default_calendar_id: Option<Id>,
}

impl CalendarManager {
    pub fn new() -> Self {
        Self {
            calendars: HashMap::new(),
            resource_calendars: HashMap::new(),
            default_calendar_id: None,
        }
    }

    pub fn add_calendar(&mut self, calendar: Calendar) -> Id {
        let calendar_id = calendar.id;
        
        // If this is the first calendar, make it the default
        if self.calendars.is_empty() {
            self.default_calendar_id = Some(calendar_id);
        }
        
        self.calendars.insert(calendar_id, calendar);
        calendar_id
    }

    pub fn get_calendar(&self, calendar_id: &Id) -> Option<&Calendar> {
        self.calendars.get(calendar_id)
    }

    pub fn get_default_calendar(&self) -> Option<&Calendar> {
        self.default_calendar_id.and_then(|id| self.calendars.get(&id))
    }

    pub fn set_default_calendar(&mut self, calendar_id: Id) -> Result<()> {
        if self.calendars.contains_key(&calendar_id) {
            self.default_calendar_id = Some(calendar_id);
            Ok(())
        } else {
            Err(DesignTrackError::NotFound("Calendar not found".to_string()))
        }
    }

    pub fn add_resource_calendar(&mut self, resource_calendar: ResourceCalendar) -> Id {
        let resource_calendar_id = resource_calendar.id;
        self.resource_calendars.insert(resource_calendar_id, resource_calendar);
        resource_calendar_id
    }

    pub fn get_resource_calendar(&self, resource_id: &Id) -> Option<&ResourceCalendar> {
        self.resource_calendars.values().find(|rc| rc.resource_id == *resource_id)
    }

    pub fn get_effective_calendar_for_resource(&self, resource_id: &Id) -> Option<&Calendar> {
        // First try to get resource-specific calendar
        if let Some(resource_calendar) = self.get_resource_calendar(resource_id) {
            return self.get_calendar(&resource_calendar.calendar_id);
        }
        
        // Fall back to default calendar
        self.get_default_calendar()
    }
}

impl Default for CalendarManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CalendarRepository {
    calendars: Vec<Calendar>,
}

impl CalendarRepository {
    pub fn new() -> Self {
        Self { calendars: Vec::new() }
    }
}

impl Repository<Calendar> for CalendarRepository {
    fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Vec<Calendar>> {
        if !path.as_ref().exists() {
            return Ok(Vec::new());
        }
        
        let content = std::fs::read_to_string(path)?;
        if content.trim().is_empty() {
            return Ok(Vec::new());
        }
        
        let calendars: Vec<Calendar> = ron::from_str(&content)?;
        Ok(calendars)
    }

    fn save_to_file<P: AsRef<std::path::Path>>(items: &[Calendar], path: P) -> Result<()> {
        let content = format_ron_pretty(items)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    fn find_by_id(&self, id: Id) -> Option<&Calendar> {
        self.calendars.iter().find(|c| c.id == id)
    }

    fn find_by_name(&self, name: &str) -> Option<&Calendar> {
        self.calendars.iter().find(|c| c.name == name)
    }

    fn add(&mut self, item: Calendar) -> Result<()> {
        item.validate()?;
        self.calendars.push(item);
        Ok(())
    }

    fn update(&mut self, item: Calendar) -> Result<()> {
        item.validate()?;
        if let Some(index) = self.calendars.iter().position(|c| c.id == item.id) {
            self.calendars[index] = item;
            Ok(())
        } else {
            Err(DesignTrackError::NotFound("Calendar not found".to_string()))
        }
    }

    fn remove(&mut self, id: Id) -> Result<()> {
        if let Some(index) = self.calendars.iter().position(|c| c.id == id) {
            self.calendars.remove(index);
            Ok(())
        } else {
            Err(DesignTrackError::NotFound("Calendar not found".to_string()))
        }
    }

    fn list(&self) -> &[Calendar] {
        &self.calendars
    }
}

// Display implementations
impl std::fmt::Display for ExceptionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExceptionType::Working => write!(f, "Working"),
            ExceptionType::NonWorking => write!(f, "Non-Working"),
            ExceptionType::HalfDay => write!(f, "Half Day"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_default_calendar() {
        let calendar = Calendar::default();
        assert_eq!(calendar.working_days.len(), 5);
        assert_eq!(calendar.working_hours.daily_hours, 8.0);
    }

    #[test]
    fn test_working_day_detection() {
        let calendar = Calendar::default();
        let monday = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(); // Monday
        let saturday = NaiveDate::from_ymd_opt(2024, 1, 6).unwrap(); // Saturday
        
        assert!(calendar.is_working_day(monday));
        assert!(!calendar.is_working_day(saturday));
    }

    #[test]
    fn test_holiday_detection() {
        let mut calendar = Calendar::default();
        let holiday = Holiday::new("Test Holiday".to_string(), NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        calendar.add_holiday(holiday);
        
        assert!(calendar.is_holiday(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()));
        assert!(!calendar.is_holiday(NaiveDate::from_ymd_opt(2024, 1, 2).unwrap()));
    }

    #[test]
    fn test_recurring_holiday() {
        let mut calendar = Calendar::default();
        let holiday = Holiday::new_recurring("Christmas".to_string(), NaiveDate::from_ymd_opt(2024, 12, 25).unwrap());
        calendar.add_holiday(holiday);
        
        assert!(calendar.is_holiday(NaiveDate::from_ymd_opt(2024, 12, 25).unwrap()));
        assert!(calendar.is_holiday(NaiveDate::from_ymd_opt(2025, 12, 25).unwrap()));
    }

    #[test]
    fn test_working_hours_calculation() {
        let calendar = Calendar::default();
        let working_day = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(); // Monday
        let non_working_day = NaiveDate::from_ymd_opt(2024, 1, 6).unwrap(); // Saturday
        
        assert_eq!(calendar.get_working_hours(working_day), 8.0);
        assert_eq!(calendar.get_working_hours(non_working_day), 0.0);
    }

    #[test]
    fn test_half_day_exception() {
        let mut calendar = Calendar::default();
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        calendar.add_exception(CalendarException::new(date, ExceptionType::HalfDay));
        
        assert_eq!(calendar.get_working_hours(date), 4.0);
    }

    #[test]
    fn test_working_days_between() {
        let calendar = Calendar::default();
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(); // Monday
        let end = NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(); // Friday
        
        assert_eq!(calendar.working_days_between(start, end), 5.0);
    }

    #[test]
    fn test_add_working_days() {
        let calendar = Calendar::default();
        let start = NaiveDate::from_ymd_opt(2024, 1, 8).unwrap(); // Monday
        
        // Adding 3 working days should give us Thursday Jan 11
        let result = calendar.add_working_days(start, 3.0);
        let expected = NaiveDate::from_ymd_opt(2024, 1, 11).unwrap(); // Thursday
        assert_eq!(result, expected);
        
        // Adding 5 working days should give us Monday Jan 15 (skipping weekend)
        let result = calendar.add_working_days(start, 5.0);
        let expected = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(); // Monday
        assert_eq!(result, expected);
    }
}