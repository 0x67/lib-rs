use logger::{utc_offset_hms, utc_offset_hours};
use time::UtcOffset;

#[test]
fn test_utc_offset_hours_valid() {
    let utc = utc_offset_hours(0);
    assert_eq!(utc, UtcOffset::UTC);

    let utc_plus_8 = utc_offset_hours(8);
    assert_eq!(utc_plus_8, UtcOffset::from_hms(8, 0, 0).unwrap());

    let utc_minus_5 = utc_offset_hours(-5);
    assert_eq!(utc_minus_5, UtcOffset::from_hms(-5, 0, 0).unwrap());

    let utc_plus_12 = utc_offset_hours(12);
    assert_eq!(utc_plus_12, UtcOffset::from_hms(12, 0, 0).unwrap());

    let utc_minus_12 = utc_offset_hours(-12);
    assert_eq!(utc_minus_12, UtcOffset::from_hms(-12, 0, 0).unwrap());
}

#[test]
fn test_utc_offset_hms_valid() {
    let india = utc_offset_hms(5, 30, 0);
    assert_eq!(india, UtcOffset::from_hms(5, 30, 0).unwrap());

    let nepal = utc_offset_hms(5, 45, 0);
    assert_eq!(nepal, UtcOffset::from_hms(5, 45, 0).unwrap());

    let australia = utc_offset_hms(9, 30, 0);
    assert_eq!(australia, UtcOffset::from_hms(9, 30, 0).unwrap());

    let negative = utc_offset_hms(-3, -30, 0);
    assert_eq!(negative, UtcOffset::from_hms(-3, -30, 0).unwrap());
}

#[test]
#[should_panic(expected = "invalid UTC offset")]
fn test_utc_offset_hms_invalid_minutes() {
    utc_offset_hms(5, 60, 0);
}

#[test]
#[should_panic(expected = "invalid UTC offset")]
fn test_utc_offset_hms_invalid_seconds() {
    utc_offset_hms(5, 30, 60);
}
