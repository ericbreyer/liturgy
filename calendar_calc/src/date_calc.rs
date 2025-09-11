use chrono::{Datelike, NaiveDate};

pub(crate) fn get_preceding_sunday(date: NaiveDate) -> NaiveDate {
    let weekday = date.weekday();
    date - chrono::Days::new(((weekday.num_days_from_sunday()) % 7).into())
}

pub(crate) fn get_following_sunday(date: NaiveDate) -> NaiveDate {
    let weekday = date.weekday();
    date + chrono::Days::new(((7 - weekday.num_days_from_sunday()) % 7).into())
}

pub(crate) fn num_sundays_after_date_inclusive(my_date: NaiveDate, other: NaiveDate) -> i32 {
    let preceding_sunday = get_preceding_sunday(my_date);

    if other.weekday() != chrono::Weekday::Sun {
        panic!(
            "The date {:?} is not a Sunday (it's a {:?})",
            other,
            other.weekday()
        );
    }

    let days_diff = (other - preceding_sunday).num_days();
    if days_diff < 0 {
        return 0i32;
    }
    ((days_diff / 7) + 1) as i32
}

pub fn num_weeks_after_date(my_date: NaiveDate, other: NaiveDate) -> i32 {
    let first_sunday_after =
        my_date + chrono::Days::new((7 - my_date.weekday().num_days_from_sunday()) as u64);
    let first_sunday_before =
        my_date - chrono::Days::new(my_date.weekday().num_days_from_sunday() as u64);
    if other < my_date {
        return 0;
    }
    if other < first_sunday_after {
        return 1;
    }

    let days_diff = (other - first_sunday_before).num_days();
    (days_diff / 7 + 1).try_into().unwrap()
}

pub fn to_roman_numeral(mut n: i32) -> String {
    if n <= 0 {
        return String::new();
    }
    let mut result = String::new();
    let roman_numerals = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];

    for &(value, symbol) in roman_numerals.iter() {
        while n >= value {
            result.push_str(symbol);
            n -= value;
        }
    }
    result
}

pub fn to_month_string(month: u32) -> String {
    match month {
        1 => "January".to_string(),
        2 => "February".to_string(),
        3 => "March".to_string(),
        4 => "April".to_string(),
        5 => "May".to_string(),
        6 => "June".to_string(),
        7 => "July".to_string(),
        8 => "August".to_string(),
        9 => "September".to_string(),
        10 => "October".to_string(),
        11 => "November".to_string(),
        12 => "December".to_string(),
        _ => panic!("Invalid month: {}", month),
    }
}

#[cfg(test)]
mod test {
    use chrono::NaiveDate;
    use test_case::test_case;

    use super::*;

    // Helper function to create dates more concisely in test cases
    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test_case(date(2025, 8, 24) => date(2025, 8, 24); "sunday returns same date")]
    #[test_case(date(2025, 8, 25) => date(2025, 8, 24); "monday returns previous day")]
    #[test_case(date(2025, 8, 30) => date(2025, 8, 24); "saturday returns 6 days earlier")]
    #[test_case(date(2025, 8, 27) => date(2025, 8, 24); "wednesday middle of week")]
    #[test_case(date(2025, 1, 1) => date(2024, 12, 29); "across year boundary")]
    fn test_get_preceding_sunday(input_date: NaiveDate) -> NaiveDate {
        get_preceding_sunday(input_date)
    }

    #[test_case(date(2025, 8, 24) => date(2025, 8, 24); "sunday returns same date")]
    #[test_case(date(2025, 8, 25) => date(2025, 8, 31); "monday returns 6 days later")]
    #[test_case(date(2025, 8, 30) => date(2025, 8, 31); "saturday returns next day")]
    #[test_case(date(2025, 8, 27) => date(2025, 8, 31); "wednesday middle of week")]
    #[test_case(date(2024, 12, 31) => date(2025, 1, 5); "across year boundary")]
    fn test_get_following_sunday(input_date: NaiveDate) -> NaiveDate {
        get_following_sunday(input_date)
    }

    #[test_case(date(2025, 8, 25), date(2025, 8, 24) => 1; "same week sunday")]
    #[test_case(date(2025, 8, 25), date(2025, 8, 31) => 2; "next week sunday")]
    #[test_case(date(2025, 8, 25), date(2025, 9, 14) => 4; "three weeks later")]
    #[test_case(date(2025, 8, 25), date(2025, 9, 7) => 3; "two weeks later")]
    fn test_num_sundays_after_date_inclusive(base_date: NaiveDate, sunday_date: NaiveDate) -> i32 {
        num_sundays_after_date_inclusive(base_date, sunday_date)
    }

    #[test_case(date(2025, 8, 26); "tuesday")]
    #[test_case(date(2025, 8, 27); "wednesday")]
    #[test_case(date(2025, 8, 28); "thursday")]
    #[test_case(date(2025, 8, 29); "friday")]
    #[test_case(date(2025, 8, 30); "saturday")]
    #[test_case(date(2025, 8, 25); "monday")]
    #[should_panic(expected = "is not a Sunday")]
    fn test_num_sundays_after_date_inclusive_with_non_sunday(not_sunday: NaiveDate) {
        let base_date = date(2025, 8, 25); // Monday
        num_sundays_after_date_inclusive(base_date, not_sunday);
    }

    #[test_case(date(2025, 8, 25), date(2025, 8, 20) => 0; "date before base returns 0")]
    #[test_case(date(2025, 8, 25), date(2025, 8, 25) => 1; "same date as base returns 1")]
    #[test_case(date(2025, 8, 25), date(2025, 8, 27) => 1; "same week after base")]
    #[test_case(date(2025, 8, 25), date(2025, 9, 1) => 2; "next week")]
    #[test_case(date(2025, 8, 25), date(2025, 9, 15) => 4; "several weeks later")]
    #[test_case(date(2025, 8, 25), date(2025, 8, 31) => 2; "end of same week")]
    fn test_num_weeks_after_date(base_date: NaiveDate, other_date: NaiveDate) -> i32 {
        num_weeks_after_date(base_date, other_date)
    }

    // Keep some edge case tests as separate functions for clarity
    #[test]
    fn test_edge_cases_with_year_boundary() {
        // Test get_preceding_sunday across year boundary
        let new_years_day = date(2025, 1, 1); // Wednesday
        let expected_preceding = date(2024, 12, 29); // Sunday
        assert_eq!(get_preceding_sunday(new_years_day), expected_preceding);

        // Test get_following_sunday across year boundary
        let new_years_eve = date(2024, 12, 31); // Tuesday
        let expected_following = date(2025, 1, 5); // Sunday
        assert_eq!(get_following_sunday(new_years_eve), expected_following);
    }
}
