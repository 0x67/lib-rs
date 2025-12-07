use time::UtcOffset;

/// Helper function to create a UtcOffset from hours
///
/// # Panics
///
/// Panics if hours is not in the range -23..=23
///
/// # Examples
///
/// ```
/// use logger::utc_offset_hours;
///
/// let utc_plus_7 = utc_offset_hours(7);   // UTC+7 (Jakarta, Bangkok)
/// let utc_plus_8 = utc_offset_hours(8);   // UTC+8 (Singapore, Kuala Lumpur)
/// let utc = utc_offset_hours(0);          // UTC
/// ```
pub fn utc_offset_hours(hours: i8) -> UtcOffset {
    UtcOffset::from_hms(hours, 0, 0).unwrap_or_else(|_| {
        panic!(
            "invalid UTC offset hours: {}, must be in range -23..=23",
            hours
        )
    })
}

/// Helper function to create a UtcOffset from hours and minutes
///
/// # Panics
///
/// Panics if the offset is invalid (hours must be -23..=23, minutes -59..=59, seconds -59..=59)
///
/// # Examples
///
/// ```
/// use logger::utc_offset_hms;
///
/// let india = utc_offset_hms(5, 30, 0);      // UTC+5:30 (India)
/// let nepal = utc_offset_hms(5, 45, 0);      // UTC+5:45 (Nepal)
/// let australia = utc_offset_hms(9, 30, 0);  // UTC+9:30 (Adelaide)
/// ```
pub fn utc_offset_hms(hours: i8, minutes: i8, seconds: i8) -> UtcOffset {
    UtcOffset::from_hms(hours, minutes, seconds).unwrap_or_else(|_| {
        panic!(
            "invalid UTC offset: hours={}, minutes={}, seconds={}",
            hours, minutes, seconds
        )
    })
}
