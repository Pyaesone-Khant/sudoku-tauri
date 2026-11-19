//! Pure Rust Gregorian calendar date arithmetic for Daily Challenges and Daily Streaks.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::DomainError;
use crate::model::Difficulty;

/// Day of the week with Monday as the start of the rotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

/// Local calendar date (`YYYY-MM-DD`) identifying a Daily Challenge or completion day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DailyDate {
    year: i32,
    month: u8,
    day: u8,
}

impl DailyDate {
    /// Constructs a validated calendar date.
    pub fn new(year: i32, month: u8, day: u8) -> Result<Self, DomainError> {
        if !(1..=12).contains(&month) {
            return Err(DomainError::InvalidDate(format!(
                "{year:04}-{month:02}-{day:02}"
            )));
        }
        let max_day = Self::days_in_month(year, month);
        if day < 1 || day > max_day {
            return Err(DomainError::InvalidDate(format!(
                "{year:04}-{month:02}-{day:02}"
            )));
        }
        Ok(Self { year, month, day })
    }

    /// Returns the four-digit Gregorian year.
    pub const fn year(self) -> i32 {
        self.year
    }

    /// Returns the month (`1..=12`).
    pub const fn month(self) -> u8 {
        self.month
    }

    /// Returns the day of month (`1..=31`).
    pub const fn day(self) -> u8 {
        self.day
    }

    /// Returns whether `year` is a Gregorian leap year.
    pub const fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    /// Returns the number of days in `(year, month)` for `month` in `1..=12`.
    pub const fn days_in_month(year: i32, month: u8) -> u8 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if Self::is_leap_year(year) => 29,
            2 => 28,
            _ => 0,
        }
    }

    /// Converts this date to days since 1970-01-01 using Howard Hinnant's civil algorithm.
    pub fn days_since_epoch(self) -> i64 {
        let y = i64::from(self.year) - i64::from(self.month <= 2);
        let era = if y >= 0 { y } else { y - 399 } / 400;
        let yoe = y - era * 400;
        let m = i64::from(self.month);
        let d = i64::from(self.day);
        let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    /// Reconstructs a `DailyDate` from days since 1970-01-01.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn from_days_since_epoch(z: i64) -> Self {
        let z = z + 719_468;
        let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
        let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = mp + if mp < 10 { 3 } else { -9 };
        let year = (y + i64::from(m <= 2)) as i32;
        Self {
            year,
            month: m as u8,
            day: d as u8,
        }
    }

    /// Returns the previous calendar date.
    #[must_use]
    pub fn prev_day(self) -> Self {
        Self::from_days_since_epoch(self.days_since_epoch() - 1)
    }

    /// Returns the next calendar date.
    #[must_use]
    pub fn next_day(self) -> Self {
        Self::from_days_since_epoch(self.days_since_epoch() + 1)
    }

    /// Returns signed day difference `self - other`.
    pub fn days_since(self, other: Self) -> i64 {
        self.days_since_epoch() - other.days_since_epoch()
    }

    /// Returns the Gregorian day of the week (1970-01-01 was Thursday).
    pub fn weekday(self) -> Weekday {
        // 1970-01-01 (epoch day 0) was Thursday (index 3 when Monday = 0).
        match (self.days_since_epoch() + 3).rem_euclid(7) {
            0 => Weekday::Monday,
            1 => Weekday::Tuesday,
            2 => Weekday::Wednesday,
            3 => Weekday::Thursday,
            4 => Weekday::Friday,
            5 => Weekday::Saturday,
            _ => Weekday::Sunday,
        }
    }

    /// Deterministic weekly Difficulty schedule for Daily Challenges.
    pub fn daily_difficulty(self) -> Difficulty {
        match self.weekday() {
            Weekday::Monday => Difficulty::Easy,
            Weekday::Tuesday | Weekday::Wednesday => Difficulty::Medium,
            Weekday::Thursday | Weekday::Friday => Difficulty::Hard,
            Weekday::Saturday | Weekday::Sunday => Difficulty::Expert,
        }
    }

    /// Formats the date as canonical `YYYY-MM-DD`.
    pub fn to_iso_string(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl fmt::Display for DailyDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl FromStr for DailyDate {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('-');
        let (Some(y), Some(m), Some(d), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return Err(DomainError::InvalidDate(s.to_owned()));
        };
        let year = y
            .parse::<i32>()
            .map_err(|_| DomainError::InvalidDate(s.to_owned()))?;
        let month = m
            .parse::<u8>()
            .map_err(|_| DomainError::InvalidDate(s.to_owned()))?;
        let day = d
            .parse::<u8>()
            .map_err(|_| DomainError::InvalidDate(s.to_owned()))?;
        Self::new(year, month, day)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(2026, 11, 16, Weekday::Monday, Difficulty::Easy)]
    #[case(2026, 11, 17, Weekday::Tuesday, Difficulty::Medium)]
    #[case(2026, 11, 18, Weekday::Wednesday, Difficulty::Medium)]
    #[case(2026, 11, 19, Weekday::Thursday, Difficulty::Hard)]
    #[case(2026, 11, 20, Weekday::Friday, Difficulty::Hard)]
    #[case(2026, 11, 21, Weekday::Saturday, Difficulty::Expert)]
    #[case(2026, 11, 22, Weekday::Sunday, Difficulty::Expert)]
    fn weekday_and_daily_difficulty_follow_weekly_schedule(
        #[case] year: i32,
        #[case] month: u8,
        #[case] day: u8,
        #[case] expected_weekday: Weekday,
        #[case] expected_difficulty: Difficulty,
    ) {
        let date = DailyDate::new(year, month, day).unwrap();
        assert_eq!(date.weekday(), expected_weekday);
        assert_eq!(date.daily_difficulty(), expected_difficulty);
    }

    #[test]
    fn leap_year_and_month_rollovers_work() {
        let feb28 = DailyDate::new(2024, 2, 28).unwrap();
        let feb29 = feb28.next_day();
        assert_eq!(feb29, DailyDate::new(2024, 2, 29).unwrap());
        assert_eq!(feb29.next_day(), DailyDate::new(2024, 3, 1).unwrap());
        assert!(DailyDate::new(2025, 2, 29).is_err());

        let dec31 = DailyDate::new(2025, 12, 31).unwrap();
        let jan1 = dec31.next_day();
        assert_eq!(jan1, DailyDate::new(2026, 1, 1).unwrap());
        assert_eq!(jan1.prev_day(), dec31);
    }

    #[test]
    fn iso_string_roundtrips() {
        let parsed: DailyDate = "2026-11-19".parse().unwrap();
        assert_eq!(parsed.to_iso_string(), "2026-11-19");
        assert!("2026-13-01".parse::<DailyDate>().is_err());
        assert!("not-a-date".parse::<DailyDate>().is_err());
    }

    proptest! {
        #[test]
        fn epoch_day_roundtrip_is_lossless(days in -50_000_i64..50_000_i64) {
            let date = DailyDate::from_days_since_epoch(days);
            assert_eq!(date.days_since_epoch(), days);
            assert_eq!(date.next_day().prev_day(), date);
        }
    }
}
