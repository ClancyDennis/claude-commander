use chrono::{DateTime, Datelike, TimeZone, Utc};
use std::time::{SystemTime, UNIX_EPOCH};

/// Returns the current time in milliseconds since UNIX epoch
pub fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

/// Returns the timestamp for the start of the day (00:00:00 UTC) for a given timestamp
pub fn start_of_day(timestamp: i64) -> i64 {
    // Convert to seconds, divide by seconds in a day, multiply back
    let seconds = timestamp / 1000;
    let day_start_seconds = (seconds / 86400) * 86400;
    day_start_seconds * 1000
}

/// Returns the timestamp for the start of the month (first day, 00:00:00 UTC) for a given timestamp
pub fn start_of_month(timestamp: i64) -> i64 {
    let datetime = DateTime::from_timestamp(timestamp / 1000, 0).unwrap_or_else(Utc::now);

    let year = datetime.year();
    let month = datetime.month();

    // Create first day of month at midnight UTC
    let first_of_month = Utc
        .with_ymd_and_hms(year, month, 1, 0, 0, 0)
        .single()
        .unwrap_or(datetime);

    first_of_month.timestamp() * 1000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now_millis() {
        let now = now_millis();
        assert!(now > 0);
        // Should be a reasonable timestamp (after 2020)
        assert!(now > 1577836800000);
    }

    #[test]
    fn test_start_of_day() {
        // Test with a known timestamp: 2024-01-15 14:30:00 UTC = 1705329000000
        let timestamp = 1705329000000;
        let start = start_of_day(timestamp);

        // Should be 2024-01-15 00:00:00 UTC = 1705276800000
        assert_eq!(start, 1705276800000);
    }

    #[test]
    fn test_start_of_month() {
        // Test with a known timestamp: 2024-01-15 14:30:00 UTC = 1705329000000
        let timestamp = 1705329000000;
        let start = start_of_month(timestamp);

        // Should be 2024-01-01 00:00:00 UTC = 1704067200000
        assert_eq!(start, 1704067200000);
    }
}
