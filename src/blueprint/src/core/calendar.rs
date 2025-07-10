use chrono::{Datelike, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Calendar {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub holidays: Vec<Holiday>,
    #[serde(default = "default_working_days")]
    pub working_days: Vec<Weekday>,
    #[serde(default)]
    pub working_hours: WorkingHours,
    #[serde(default)]
    pub exceptions: Vec<CalendarException>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Holiday {
    pub name: String,
    pub date: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub recurring: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingHours {
    #[serde(default = "default_start_time")]
    pub start_time: u8, // Hour of day (0-23)
    #[serde(default = "default_end_time")]
    pub end_time: u8,   // Hour of day (0-23)
    #[serde(default = "default_daily_hours")]
    pub daily_hours: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarException {
    pub date: NaiveDate,
    pub exception_type: ExceptionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExceptionType {
    #[serde(rename = "working")]
    Working,    // Working day that would normally be non-working
    #[serde(rename = "non_working")]
    NonWorking, // Non-working day that would normally be working
    #[serde(rename = "half_day")]
    HalfDay,    // Half working day
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCalendar {
    pub resource_id: String,
    pub calendar_id: String,
    #[serde(default)]
    pub overrides: Vec<CalendarException>,
}

fn default_start_time() -> u8 {
    9
}

fn default_end_time() -> u8 {
    17
}

fn default_daily_hours() -> f32 {
    8.0
}

fn default_working_days() -> Vec<Weekday> {
    vec![
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
    ]
}

impl Default for WorkingHours {
    fn default() -> Self {
        Self {
            start_time: default_start_time(),
            end_time: default_end_time(),
            daily_hours: default_daily_hours(),
        }
    }
}

impl Default for Calendar {
    fn default() -> Self {
        Self {
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
        }
    }
}

impl Calendar {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    pub fn with_holidays(mut self, holidays: Vec<Holiday>) -> Self {
        self.holidays = holidays;
        self
    }

    pub fn add_holiday(&mut self, holiday: Holiday) {
        self.holidays.push(holiday);
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

    pub fn add_working_days(&self, start_date: NaiveDate, working_days: f32) -> NaiveDate {
        if working_days <= 0.0 {
            return start_date;
        }
        
        
        let mut current = start_date;
        let mut remaining_days = working_days;
        
        // Add safety limit to prevent infinite loops
        let max_iterations = (working_days * 10.0) as i32 + 365; // Allow for weekends and holidays
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
                    let working_day_fraction = daily_hours / self.working_hours.daily_hours;
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

    pub fn subtract_working_days(&self, start_date: NaiveDate, working_days: f32) -> NaiveDate {
        if working_days <= 0.0 {
            return start_date;
        }
        
        let mut current = start_date;
        let mut remaining_days = working_days;
        
        // Add safety limit to prevent infinite loops
        let max_iterations = (working_days * 10.0) as i32 + 365; // Allow for weekends and holidays
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
                    let working_day_fraction = daily_hours / self.working_hours.daily_hours;
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
        loop {
            if let Some(prev_date) = prev.pred_opt() {
                prev = prev_date;
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
}

impl Holiday {
    pub fn new(name: String, date: NaiveDate) -> Self {
        Self {
            name,
            date,
            description: None,
            recurring: false,
        }
    }

    pub fn recurring(mut self) -> Self {
        self.recurring = true;
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

impl ResourceCalendar {
    pub fn new(resource_id: String, calendar_id: String) -> Self {
        Self {
            resource_id,
            calendar_id,
            overrides: Vec::new(),
        }
    }

    pub fn with_overrides(mut self, overrides: Vec<CalendarException>) -> Self {
        self.overrides = overrides;
        self
    }

    pub fn add_override(&mut self, exception: CalendarException) {
        self.overrides.push(exception);
    }
}

impl CalendarException {
    pub fn new(date: NaiveDate, exception_type: ExceptionType) -> Self {
        Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_default_calendar() {
        let calendar = Calendar::default();
        assert_eq!(calendar.name, "Default");
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
        let holiday = Holiday::new("Christmas".to_string(), NaiveDate::from_ymd_opt(2024, 12, 25).unwrap())
            .recurring();
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
        calendar.exceptions.push(CalendarException::new(date, ExceptionType::HalfDay));
        
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