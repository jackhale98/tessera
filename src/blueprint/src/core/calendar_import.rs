use crate::core::{Calendar, Holiday, CalendarException, ExceptionType};
use anyhow::Result;
use chrono::NaiveDate;
use ical::IcalParser;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub struct CalendarImporter;

impl CalendarImporter {
    pub fn new() -> Self {
        Self
    }

    pub fn import_from_ics<P: AsRef<Path>>(
        &self,
        path: P,
        calendar_name: String,
    ) -> Result<Calendar> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let parser = IcalParser::new(reader);

        let mut calendar = Calendar::new(calendar_name);
        let mut holidays = Vec::new();

        for line in parser {
            let cal = line?;
            
            for event in cal.events {
                if let Some(summary) = event.get_summary() {
                    // Parse the event dates
                    if let (Some(start_date), Some(end_date)) = (
                        self.parse_date_from_event(&event, "DTSTART")?,
                        self.parse_date_from_event(&event, "DTEND")?,
                    ) {
                        // Check if this is a holiday (all-day event)
                        if self.is_all_day_event(&event) {
                            let holiday = Holiday::new(summary.clone(), start_date);
                            holidays.push(holiday);
                        }
                        
                        // Check if this should be treated as a non-working day
                        if self.is_non_working_event(&event) {
                            let mut current_date = start_date;
                            while current_date <= end_date {
                                let exception = CalendarException::new(
                                    current_date,
                                    ExceptionType::NonWorking,
                                ).with_description(summary.clone());
                                calendar.exceptions.push(exception);
                                current_date = current_date.succ_opt().unwrap();
                            }
                        }
                    }
                }
            }
        }

        calendar.holidays = holidays;
        Ok(calendar)
    }

    pub fn import_holidays_from_ics<P: AsRef<Path>>(
        &self,
        path: P,
        mut calendar: Calendar,
    ) -> Result<Calendar> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let parser = IcalParser::new(reader);

        for line in parser {
            let cal = line?;
            
            for event in cal.events {
                if let Some(summary) = event.get_summary() {
                    if let Some(start_date) = self.parse_date_from_event(&event, "DTSTART")? {
                        // Check if this is a recurring event
                        let recurring = event.get_property("RRULE").is_some();
                        
                        let mut holiday = Holiday::new(summary.clone(), start_date);
                        if recurring {
                            holiday = holiday.recurring();
                        }
                        
                        if let Some(description) = event.get_description() {
                            holiday = holiday.with_description(description.clone());
                        }
                        
                        calendar.add_holiday(holiday);
                    }
                }
            }
        }

        Ok(calendar)
    }

    fn parse_date_from_event(
        &self,
        event: &ical::parser::ical::component::IcalEvent,
        property_name: &str,
    ) -> Result<Option<NaiveDate>> {
        if let Some(property) = event.get_property(property_name) {
            if let Some(value) = property.value.as_ref() {
                // Handle different date formats
                if value.len() == 8 {
                    // YYYYMMDD format
                    let year = value[0..4].parse::<i32>()?;
                    let month = value[4..6].parse::<u32>()?;
                    let day = value[6..8].parse::<u32>()?;
                    
                    if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                        return Ok(Some(date));
                    }
                } else if value.len() >= 15 {
                    // YYYYMMDDTHHMMSS format - extract date part
                    let year = value[0..4].parse::<i32>()?;
                    let month = value[4..6].parse::<u32>()?;
                    let day = value[6..8].parse::<u32>()?;
                    
                    if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                        return Ok(Some(date));
                    }
                }
            }
        }
        Ok(None)
    }

    fn is_all_day_event(&self, event: &ical::parser::ical::component::IcalEvent) -> bool {
        // Check if DTSTART has only date (no time)
        if let Some(property) = event.get_property("DTSTART") {
            if let Some(value) = property.value.as_ref() {
                return value.len() == 8; // YYYYMMDD format indicates all-day
            }
        }
        false
    }

    fn is_non_working_event(&self, event: &ical::parser::ical::component::IcalEvent) -> bool {
        // Check if event is marked as a holiday or non-working day
        if let Some(summary) = event.get_summary() {
            let summary_lower = summary.to_lowercase();
            return summary_lower.contains("holiday") 
                || summary_lower.contains("vacation")
                || summary_lower.contains("off")
                || summary_lower.contains("closed");
        }
        false
    }

    pub fn export_to_ics<P: AsRef<Path>>(&self, calendar: &Calendar, path: P) -> Result<()> {
        let mut ical_content = String::new();
        
        // ICS header
        ical_content.push_str("BEGIN:VCALENDAR\r\n");
        ical_content.push_str("VERSION:2.0\r\n");
        ical_content.push_str("PRODID:-//Blueprint Project Management//Calendar//EN\r\n");
        ical_content.push_str(&format!("X-WR-CALNAME:{}\r\n", calendar.name));
        
        if let Some(description) = &calendar.description {
            ical_content.push_str(&format!("X-WR-CALDESC:{}\r\n", description));
        }
        
        // Add holidays as events
        for holiday in &calendar.holidays {
            ical_content.push_str("BEGIN:VEVENT\r\n");
            ical_content.push_str(&format!("DTSTART;VALUE=DATE:{}\r\n", 
                holiday.date.format("%Y%m%d")));
            ical_content.push_str(&format!("DTEND;VALUE=DATE:{}\r\n", 
                holiday.date.succ_opt().unwrap().format("%Y%m%d")));
            ical_content.push_str(&format!("SUMMARY:{}\r\n", holiday.name));
            
            if let Some(description) = &holiday.description {
                ical_content.push_str(&format!("DESCRIPTION:{}\r\n", description));
            }
            
            if holiday.recurring {
                ical_content.push_str("RRULE:FREQ=YEARLY\r\n");
            }
            
            ical_content.push_str(&format!("UID:holiday-{}-{}\r\n", 
                holiday.name.replace(" ", "-").to_lowercase(),
                holiday.date.format("%Y%m%d")));
            ical_content.push_str("END:VEVENT\r\n");
        }
        
        // Add calendar exceptions as events
        for exception in &calendar.exceptions {
            ical_content.push_str("BEGIN:VEVENT\r\n");
            ical_content.push_str(&format!("DTSTART;VALUE=DATE:{}\r\n", 
                exception.date.format("%Y%m%d")));
            ical_content.push_str(&format!("DTEND;VALUE=DATE:{}\r\n", 
                exception.date.succ_opt().unwrap().format("%Y%m%d")));
            
            let summary = match exception.exception_type {
                ExceptionType::Working => "Working Day",
                ExceptionType::NonWorking => "Non-Working Day",
                ExceptionType::HalfDay => "Half Day",
            };
            ical_content.push_str(&format!("SUMMARY:{}\r\n", summary));
            
            if let Some(description) = &exception.description {
                ical_content.push_str(&format!("DESCRIPTION:{}\r\n", description));
            }
            
            ical_content.push_str(&format!("UID:exception-{}-{}\r\n", 
                summary.replace(" ", "-").to_lowercase(),
                exception.date.format("%Y%m%d")));
            ical_content.push_str("END:VEVENT\r\n");
        }
        
        // ICS footer
        ical_content.push_str("END:VCALENDAR\r\n");
        
        std::fs::write(path, ical_content)?;
        Ok(())
    }
}

impl Default for CalendarImporter {
    fn default() -> Self {
        Self::new()
    }
}

// Extension trait for ical events
trait IcalEventExt {
    fn get_summary(&self) -> Option<String>;
    fn get_description(&self) -> Option<String>;
    fn get_property(&self, name: &str) -> Option<&ical::property::Property>;
}

impl IcalEventExt for ical::parser::ical::component::IcalEvent {
    fn get_summary(&self) -> Option<String> {
        self.get_property("SUMMARY")
            .and_then(|prop| prop.value.as_ref())
            .map(|s| s.clone())
    }

    fn get_description(&self) -> Option<String> {
        self.get_property("DESCRIPTION")
            .and_then(|prop| prop.value.as_ref())
            .map(|s| s.clone())
    }

    fn get_property(&self, name: &str) -> Option<&ical::property::Property> {
        self.properties.iter().find(|prop| prop.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_calendar_import_export() {
        let importer = CalendarImporter::new();
        
        // Create a simple ICS content
        let ics_content = r#"BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Test//Test//EN
BEGIN:VEVENT
DTSTART;VALUE=DATE:20240101
DTEND;VALUE=DATE:20240102
SUMMARY:New Year's Day
DESCRIPTION:Holiday
UID:newyear-2024
END:VEVENT
END:VCALENDAR
"#;

        // Write to temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", ics_content).unwrap();
        
        // Import calendar
        let calendar = importer.import_from_ics(
            temp_file.path(),
            "Test Calendar".to_string()
        ).unwrap();
        
        assert_eq!(calendar.name, "Test Calendar");
        assert_eq!(calendar.holidays.len(), 1);
        assert_eq!(calendar.holidays[0].name, "New Year's Day");
        assert_eq!(calendar.holidays[0].date, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
    }

    #[test]
    fn test_export_calendar() {
        let importer = CalendarImporter::new();
        let mut calendar = Calendar::new("Test Export".to_string());
        
        let holiday = Holiday::new("Test Holiday".to_string(), NaiveDate::from_ymd_opt(2024, 7, 4).unwrap());
        calendar.add_holiday(holiday);
        
        let temp_file = NamedTempFile::new().unwrap();
        importer.export_to_ics(&calendar, temp_file.path()).unwrap();
        
        let exported_content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(exported_content.contains("BEGIN:VCALENDAR"));
        assert!(exported_content.contains("Test Holiday"));
        assert!(exported_content.contains("20240704"));
    }
}