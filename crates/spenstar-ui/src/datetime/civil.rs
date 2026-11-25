use std::{
    fmt,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

/// An invalid date or time string.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseDateTimeError;

impl fmt::Display for ParseDateTimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid date or time")
    }
}

impl std::error::Error for ParseDateTimeError {}

/// A date in the proleptic Gregorian calendar, using astronomical year numbering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CalendarDate {
    /// Civil year, including year zero.
    pub year: i32,
    /// Month, from 1 through 12.
    pub month: u8,
    /// Day of the month, from 1 through the month's length.
    pub day: u8,
}

impl CalendarDate {
    /// First representable civil date.
    pub const MIN: Self = Self {
        year: i32::MIN,
        month: 1,
        day: 1,
    };
    /// Last representable civil date.
    pub const MAX: Self = Self {
        year: i32::MAX,
        month: 12,
        day: 31,
    };

    /// Construct a validated Gregorian date.
    pub const fn new(year: i32, month: u8, day: u8) -> Option<Self> {
        if day == 0 || day > Self::days_in_month(year, month) {
            return None;
        }
        Some(Self { year, month, day })
    }

    /// Return whether this Gregorian year has a February 29.
    pub const fn is_leap_year(year: i32) -> bool {
        year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
    }

    /// Number of days in the month, or zero for an invalid month.
    pub const fn days_in_month(year: i32, month: u8) -> u8 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if Self::is_leap_year(year) => 29,
            2 => 28,
            _ => 0,
        }
    }

    /// Day of the week, with Monday = 0 and Sunday = 6.
    pub fn weekday(self) -> u8 {
        (self.days_since_epoch() + 3).rem_euclid(7) as u8
    }

    /// Add calendar days. Returns `None` for invalid input or a result outside the year range.
    pub fn add_days(self, days: i64) -> Option<Self> {
        Self::new(self.year, self.month, self.day)?;
        Self::from_days(self.days_since_epoch().checked_add(days)?)
    }

    /// Add calendar months, capping the day at the destination month's last day.
    pub fn add_months(self, months: i64) -> Option<Self> {
        Self::new(self.year, self.month, self.day)?;
        let month = (i64::from(self.year) * 12 + i64::from(self.month) - 1).checked_add(months)?;
        let year = i32::try_from(month.div_euclid(12)).ok()?;
        let month = month.rem_euclid(12) as u8 + 1;
        Self::new(year, month, self.day.min(Self::days_in_month(year, month)))
    }

    /// Current UTC civil date using the system clock.
    pub fn today_utc() -> Self {
        let epoch = Self {
            year: 1970,
            month: 1,
            day: 1,
        };
        let now = SystemTime::now();
        let days = match now.duration_since(UNIX_EPOCH) {
            Ok(delta) => (delta.as_secs() / 86_400) as i64,
            Err(delta) => {
                let duration = delta.duration();
                -((duration.as_secs() / 86_400) as i64)
                    - i64::from(duration.as_secs() % 86_400 != 0 || duration.subsec_nanos() != 0)
            }
        };
        epoch.add_days(days).unwrap_or(epoch)
    }

    // Howard Hinnant's civil-days transform. Each era spans 400 Gregorian years.
    fn days_since_epoch(self) -> i64 {
        let year = i64::from(self.year) - i64::from(self.month <= 2);
        let era = year.div_euclid(400);
        let year_of_era = year.rem_euclid(400);
        let month = i64::from(self.month) + if self.month > 2 { -3 } else { 9 };
        let day_of_year = (153 * month + 2) / 5 + i64::from(self.day) - 1;
        era * 146_097 + year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year
            - 719_468
    }

    fn from_days(days: i64) -> Option<Self> {
        if !(Self::MIN.days_since_epoch()..=Self::MAX.days_since_epoch()).contains(&days) {
            return None;
        }
        let march_days = days + 719_468;
        let era = march_days.div_euclid(146_097);
        let day = march_days.rem_euclid(146_097);
        let year_of_era = (day - day / 1460 + day / 36_524 - day / 146_096) / 365;
        let day_of_year = day - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let march_month = (5 * day_of_year + 2) / 153;
        let month = (march_month + if march_month < 10 { 3 } else { -9 }) as u8;
        let year = i32::try_from(year_of_era + era * 400 + i64::from(month <= 2)).ok()?;
        Self::new(
            year,
            month,
            (day_of_year - (153 * march_month + 2) / 5 + 1) as u8,
        )
    }
}

impl fmt::Display for CalendarDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.year < 0 {
            f.write_str("-")?;
        }
        write!(
            f,
            "{:04}-{:02}-{:02}",
            self.year.unsigned_abs(),
            self.month,
            self.day
        )
    }
}

impl FromStr for CalendarDate {
    type Err = ParseDateTimeError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (head, day) = text.rsplit_once('-').ok_or(ParseDateTimeError)?;
        let (year, month) = head.rsplit_once('-').ok_or(ParseDateTimeError)?;
        let year_digits = year.strip_prefix('-').unwrap_or(year);
        if year_digits.len() < 4
            || !digits(year_digits)
            || month.len() != 2
            || day.len() != 2
            || !digits(month)
            || !digits(day)
        {
            return Err(ParseDateTimeError);
        }
        let year = year.parse().map_err(|_| ParseDateTimeError)?;
        let month = month.parse().map_err(|_| ParseDateTimeError)?;
        let day = day.parse().map_err(|_| ParseDateTimeError)?;
        Self::new(year, month, day).ok_or(ParseDateTimeError)
    }
}

fn digits(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit())
}

/// Time display convention.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ClockFormat {
    /// 12-hour clock with AM or PM.
    H12,
    /// 24-hour clock.
    #[default]
    H24,
}

/// A wall-clock time with whole-second precision.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClockTime {
    /// Hour from 0 through 23.
    pub hour: u8,
    /// Minute from 0 through 59.
    pub minute: u8,
    /// Second from 0 through 59.
    pub second: u8,
}

impl ClockTime {
    /// Construct a validated clock time.
    pub const fn new(hour: u8, minute: u8, second: u8) -> Option<Self> {
        if hour < 24 && minute < 60 && second < 60 {
            Some(Self {
                hour,
                minute,
                second,
            })
        } else {
            None
        }
    }

    /// Format the time with an optional seconds component.
    pub fn format(self, format: ClockFormat, seconds: bool) -> String {
        let hour = if format == ClockFormat::H12 {
            self.hour12()
        } else {
            self.hour
        };
        let mut result = format!("{hour:02}:{:02}", self.minute);
        if seconds {
            result.push_str(&format!(":{:02}", self.second));
        }
        if format == ClockFormat::H12 {
            result.push_str(if self.hour < 12 { " AM" } else { " PM" });
        }
        result
    }

    /// Parse hours and minutes, optional seconds, and AM/PM in H12 mode.
    pub fn parse(text: &str, format: ClockFormat) -> Result<Self, ParseDateTimeError> {
        let mut fields = text.split_whitespace();
        let spec = fields.next().ok_or(ParseDateTimeError)?;
        let meridiem = fields.next();
        if fields.next().is_some() {
            return Err(ParseDateTimeError);
        }
        let mut parts = spec.split(':');
        let mut hour = parts
            .next()
            .and_then(parse_component)
            .ok_or(ParseDateTimeError)?;
        let minute = parts
            .next()
            .and_then(parse_component)
            .ok_or(ParseDateTimeError)?;
        let second = parts
            .next()
            .map_or(Some(0), parse_component)
            .ok_or(ParseDateTimeError)?;
        if parts.next().is_some() {
            return Err(ParseDateTimeError);
        }
        match format {
            ClockFormat::H12 => {
                let marker = meridiem.ok_or(ParseDateTimeError)?;
                if !(1..=12).contains(&hour) {
                    return Err(ParseDateTimeError);
                }
                if marker.eq_ignore_ascii_case("PM") {
                    hour = hour % 12 + 12;
                } else if marker.eq_ignore_ascii_case("AM") {
                    hour %= 12;
                } else {
                    return Err(ParseDateTimeError);
                }
            }
            ClockFormat::H24 if meridiem.is_some() => return Err(ParseDateTimeError),
            ClockFormat::H24 => {}
        }
        Self::new(hour, minute, second).ok_or(ParseDateTimeError)
    }

    pub(super) fn hour12(self) -> u8 {
        (self.hour % 12 + 11) % 12 + 1
    }

    pub(super) fn offset(self, seconds: i64) -> Self {
        let current =
            i64::from(self.hour) * 3600 + i64::from(self.minute) * 60 + i64::from(self.second);
        let total = (current + seconds.rem_euclid(86_400)).rem_euclid(86_400);
        Self {
            hour: (total / 3600) as u8,
            minute: ((total / 60) % 60) as u8,
            second: (total % 60) as u8,
        }
    }
}

impl fmt::Display for ClockTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
    }
}

impl FromStr for ClockTime {
    type Err = ParseDateTimeError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::parse(text, ClockFormat::H24)
    }
}

pub(super) fn parse_component(text: &str) -> Option<u8> {
    (text.len() <= 2 && digits(text))
        .then(|| text.parse().ok())
        .flatten()
}
