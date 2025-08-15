// Timezone configuration for KBK Attendance System
use chrono::FixedOffset;

/// Australian Eastern Standard Time (UTC+10)
/// Note: This doesn't handle daylight saving time automatically
/// For full DST support, consider using chrono-tz crate
pub const AEST_OFFSET: FixedOffset = match FixedOffset::east_opt(10 * 3600) {
    Some(offset) => offset,
    None => panic!("Invalid timezone offset"),
};

/// Configuration for timezone handling throughout the application
pub struct TimezoneConfig {
    pub local_offset: FixedOffset,
}

impl Default for TimezoneConfig {
    fn default() -> Self {
        Self {
            local_offset: AEST_OFFSET,
        }
    }
}

impl TimezoneConfig {
    /// Get the current local timezone configuration
    pub fn local() -> Self {
        Self::default()
    }
    
    /// Format a UTC datetime as local time for CSV export (without timezone suffix)
    pub fn format_csv_datetime_with_tz(&self, utc_dt: &chrono::DateTime<chrono::Utc>) -> String {
        let local_dt = utc_dt.with_timezone(&self.local_offset);
        local_dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    
    #[test]
    fn test_timezone_conversion() {
        let config = TimezoneConfig::local();
        
        // Test UTC timestamp: 2024-12-04 11:45:20 UTC
        let utc_time = DateTime::parse_from_rfc3339("2024-12-04T11:45:20Z")
            .unwrap()
            .with_timezone(&Utc);
        
        // Should convert to local time: 2024-12-04 21:45:20 (without timezone suffix)
        let csv_str = config.format_csv_datetime_with_tz(&utc_time);
        assert_eq!(csv_str, "2024-12-04 21:45:20");
    }
}
