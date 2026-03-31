use std::fmt::{Display, Formatter, Result as FmtResult};

use chrono::{DateTime, Datelike, Local};
use serde::{Deserialize, Serialize};

/// Duration display format modes.
///
/// Determines how a `chrono::Duration` is rendered as a human-readable string.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DurationFormat {
  /// `01:02:30` — zero-padded `HH:MM:SS` clock format.
  Clock,
  /// `1d 2h 30m` — abbreviated with spaces.
  Dhm,
  /// `02:30` — hours and minutes clock format (days folded into hours).
  Hm,
  /// `90` — total minutes as a plain number.
  M,
  /// `about an hour and a half` — fuzzy natural language approximation.
  Natural,
  /// `1 hour 30 minutes` — exact natural language.
  #[default]
  Text,
}

impl DurationFormat {
  /// Parse a format name from a config string value.
  ///
  /// Unrecognized values fall back to [`DurationFormat::Text`].
  pub fn from_config(s: &str) -> Self {
    match s.trim().to_lowercase().as_str() {
      "clock" => Self::Clock,
      "dhm" => Self::Dhm,
      "hm" => Self::Hm,
      "m" => Self::M,
      "natural" => Self::Natural,
      _ => Self::Text,
    }
  }
}

/// A formatted duration that implements [`Display`].
#[derive(Clone, Debug)]
pub struct FormattedDuration {
  days: i64,
  format: DurationFormat,
  hours: i64,
  minutes: i64,
  seconds: i64,
}

impl FormattedDuration {
  /// Create a new formatted duration from a `chrono::Duration` and format mode.
  pub fn new(duration: chrono::Duration, format: DurationFormat) -> Self {
    let total_seconds = duration.num_seconds();
    let total_minutes = total_seconds / 60;
    let total_hours = total_minutes / 60;

    let days = total_hours / 24;
    let hours = total_hours % 24;
    let minutes = total_minutes % 60;
    let seconds = total_seconds % 60;

    Self {
      days,
      format,
      hours,
      minutes,
      seconds,
    }
  }

  /// Total duration expressed as whole minutes.
  fn total_minutes(&self) -> i64 {
    self.days * 24 * 60 + self.hours * 60 + self.minutes
  }
}

impl Display for FormattedDuration {
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    match self.format {
      DurationFormat::Clock => {
        let total_hours = self.days * 24 + self.hours;
        write!(f, "{:02}:{:02}:{:02}", total_hours, self.minutes, self.seconds)
      }
      DurationFormat::Dhm => write!(
        f,
        "{}",
        format_parts(self.days, self.hours, self.minutes, dhm_component)
      ),
      DurationFormat::Hm => {
        let total_hours = self.days * 24 + self.hours;
        write!(f, "{:02}:{:02}", total_hours, self.minutes)
      }
      DurationFormat::M => write!(f, "{}", self.total_minutes()),
      DurationFormat::Natural => write!(f, "{}", natural_duration(self.total_minutes())),
      DurationFormat::Text => write!(
        f,
        "{}",
        format_parts(self.days, self.hours, self.minutes, text_component)
      ),
    }
  }
}

/// A formatted short date that implements [`Display`].
#[derive(Clone, Debug)]
pub struct FormattedShortdate {
  formatted: String,
}

impl FormattedShortdate {
  /// Format a datetime using config-driven relative date buckets.
  ///
  /// Dates from today use the `today` format, dates within the last week use
  /// `this_week`, dates within the same year use `this_month`, and older dates
  /// use the `older` format.
  pub fn new(datetime: DateTime<Local>, config: &ShortdateFormatConfig) -> Self {
    let now = Local::now();
    let today = now.date_naive();

    let fmt = if datetime.date_naive() == today {
      &config.today
    } else if datetime.date_naive() > today - chrono::Duration::days(7) {
      &config.this_week
    } else if datetime.year() == today.year() {
      &config.this_month
    } else {
      &config.older
    };

    Self {
      formatted: datetime.format(fmt).to_string(),
    }
  }
}

impl Display for FormattedShortdate {
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    write!(f, "{}", self.formatted)
  }
}

/// Date format strings for relative time display.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct ShortdateFormatConfig {
  pub older: String,
  pub this_month: String,
  pub this_week: String,
  pub today: String,
}

impl Default for ShortdateFormatConfig {
  fn default() -> Self {
    Self {
      older: "%m/%d/%y %_I:%M%P".into(),
      this_month: "%m/%d %_I:%M%P".into(),
      this_week: "%a %_I:%M%P".into(),
      today: "%_I:%M%P".into(),
    }
  }
}

/// Format a tag total duration as `DD:HH:MM`.
pub fn format_tag_total(duration: chrono::Duration) -> String {
  let total_minutes = duration.num_minutes();
  let total_hours = total_minutes / 60;

  let days = total_hours / 24;
  let hours = total_hours % 24;
  let minutes = total_minutes % 60;

  format!("{days:02}:{hours:02}:{minutes:02}")
}

fn dhm_component(value: i64, _unit: &str, suffix: &str) -> String {
  format!("{value}{suffix}")
}

fn format_parts(days: i64, hours: i64, minutes: i64, fmt: fn(i64, &str, &str) -> String) -> String {
  let mut parts = Vec::new();
  if days > 0 {
    parts.push(fmt(days, "day", "d"));
  }
  if hours > 0 {
    parts.push(fmt(hours, "hour", "h"));
  }
  if minutes > 0 || parts.is_empty() {
    parts.push(fmt(minutes, "minute", "m"));
  }
  parts.join(" ")
}

fn natural_duration(total_minutes: i64) -> String {
  if total_minutes == 0 {
    return "0 minutes".into();
  }

  let hours = total_minutes / 60;
  let minutes = total_minutes % 60;
  let days = hours / 24;
  let remaining_hours = hours % 24;

  if days > 0 {
    if remaining_hours == 0 && minutes == 0 {
      return if days == 1 {
        "about a day".into()
      } else {
        format!("about {days} days")
      };
    }
    if remaining_hours >= 12 {
      return format!("about {} days", days + 1);
    }
    return format!("about {days} and a half days");
  }

  if remaining_hours > 0 {
    if minutes <= 15 {
      return if remaining_hours == 1 {
        "about an hour".into()
      } else {
        format!("about {remaining_hours} hours")
      };
    }
    if minutes >= 45 {
      let rounded = remaining_hours + 1;
      return format!("about {rounded} hours");
    }
    return if remaining_hours == 1 {
      "about an hour and a half".into()
    } else {
      format!("about {remaining_hours} and a half hours")
    };
  }

  if minutes == 1 {
    "about a minute".into()
  } else if minutes < 5 {
    "a few minutes".into()
  } else if minutes < 15 {
    format!("about {minutes} minutes")
  } else if minutes < 18 {
    "about 15 minutes".into()
  } else if minutes < 23 {
    "about 20 minutes".into()
  } else if minutes < 35 {
    "about half an hour".into()
  } else if minutes < 50 {
    "about 45 minutes".into()
  } else {
    "about an hour".into()
  }
}

fn pluralize(count: i64, word: &str) -> String {
  if count == 1 {
    format!("{count} {word}")
  } else {
    format!("{count} {word}s")
  }
}

fn text_component(value: i64, unit: &str, _suffix: &str) -> String {
  pluralize(value, unit)
}

#[cfg(test)]
mod test {
  use chrono::Duration;

  use super::*;

  mod duration_format {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_defaults_unknown_to_text() {
      assert_eq!(DurationFormat::from_config("unknown"), DurationFormat::Text);
    }

    #[test]
    fn it_is_case_insensitive() {
      assert_eq!(DurationFormat::from_config("CLOCK"), DurationFormat::Clock);
    }

    #[test]
    fn it_parses_clock_from_config() {
      assert_eq!(DurationFormat::from_config("clock"), DurationFormat::Clock);
    }

    #[test]
    fn it_parses_dhm_from_config() {
      assert_eq!(DurationFormat::from_config("dhm"), DurationFormat::Dhm);
    }

    #[test]
    fn it_parses_hm_from_config() {
      assert_eq!(DurationFormat::from_config("hm"), DurationFormat::Hm);
    }

    #[test]
    fn it_parses_m_from_config() {
      assert_eq!(DurationFormat::from_config("m"), DurationFormat::M);
    }

    #[test]
    fn it_parses_natural_from_config() {
      assert_eq!(DurationFormat::from_config("natural"), DurationFormat::Natural);
    }

    #[test]
    fn it_parses_text_from_config() {
      assert_eq!(DurationFormat::from_config("text"), DurationFormat::Text);
    }
  }

  mod format_tag_total {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_formats_zero() {
      assert_eq!(format_tag_total(Duration::zero()), "00:00:00");
    }

    #[test]
    fn it_formats_hours_and_minutes() {
      assert_eq!(format_tag_total(Duration::seconds(5400)), "00:01:30");
    }

    #[test]
    fn it_formats_days_hours_minutes() {
      let duration = Duration::seconds(93600 + 1800);

      assert_eq!(format_tag_total(duration), "01:02:30");
    }
  }

  mod formatted_duration {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_formats_clock() {
      let fd = FormattedDuration::new(Duration::seconds(93600), DurationFormat::Clock);

      assert_eq!(fd.to_string(), "26:00:00");
    }

    #[test]
    fn it_formats_clock_with_minutes() {
      let fd = FormattedDuration::new(Duration::seconds(5400), DurationFormat::Clock);

      assert_eq!(fd.to_string(), "01:30:00");
    }

    #[test]
    fn it_formats_clock_with_seconds() {
      let fd = FormattedDuration::new(Duration::seconds(3661), DurationFormat::Clock);

      assert_eq!(fd.to_string(), "01:01:01");
    }

    #[test]
    fn it_formats_dhm() {
      let fd = FormattedDuration::new(Duration::seconds(93600 + 1800), DurationFormat::Dhm);

      assert_eq!(fd.to_string(), "1d 2h 30m");
    }

    #[test]
    fn it_formats_dhm_hours_only() {
      let fd = FormattedDuration::new(Duration::hours(3), DurationFormat::Dhm);

      assert_eq!(fd.to_string(), "3h");
    }

    #[test]
    fn it_formats_dhm_zero_duration() {
      let fd = FormattedDuration::new(Duration::zero(), DurationFormat::Dhm);

      assert_eq!(fd.to_string(), "0m");
    }

    #[test]
    fn it_formats_hm() {
      let fd = FormattedDuration::new(Duration::seconds(93600 + 1800), DurationFormat::Hm);

      assert_eq!(fd.to_string(), "26:30");
    }

    #[test]
    fn it_formats_m() {
      let fd = FormattedDuration::new(Duration::seconds(5400), DurationFormat::M);

      assert_eq!(fd.to_string(), "90");
    }

    #[test]
    fn it_formats_natural_about_hours() {
      let fd = FormattedDuration::new(Duration::hours(3), DurationFormat::Natural);

      assert_eq!(fd.to_string(), "about 3 hours");
    }

    #[test]
    fn it_formats_natural_about_20_minutes() {
      let fd = FormattedDuration::new(Duration::minutes(18), DurationFormat::Natural);

      assert_eq!(fd.to_string(), "about 20 minutes");
    }

    #[test]
    fn it_formats_natural_few_minutes() {
      let fd = FormattedDuration::new(Duration::minutes(3), DurationFormat::Natural);

      assert_eq!(fd.to_string(), "a few minutes");
    }

    #[test]
    fn it_formats_natural_half_hour() {
      let fd = FormattedDuration::new(Duration::minutes(30), DurationFormat::Natural);

      assert_eq!(fd.to_string(), "about half an hour");
    }

    #[test]
    fn it_formats_natural_hour_and_half() {
      let fd = FormattedDuration::new(Duration::minutes(90), DurationFormat::Natural);

      assert_eq!(fd.to_string(), "about an hour and a half");
    }

    #[test]
    fn it_formats_text() {
      let fd = FormattedDuration::new(Duration::seconds(5400), DurationFormat::Text);

      assert_eq!(fd.to_string(), "1 hour 30 minutes");
    }

    #[test]
    fn it_formats_text_plural() {
      let fd = FormattedDuration::new(Duration::seconds(93600 + 1800), DurationFormat::Text);

      assert_eq!(fd.to_string(), "1 day 2 hours 30 minutes");
    }

    #[test]
    fn it_formats_text_singular() {
      let fd = FormattedDuration::new(Duration::hours(1), DurationFormat::Text);

      assert_eq!(fd.to_string(), "1 hour");
    }

    #[test]
    fn it_formats_text_zero_duration() {
      let fd = FormattedDuration::new(Duration::zero(), DurationFormat::Text);

      assert_eq!(fd.to_string(), "0 minutes");
    }
  }

  mod formatted_shortdate {
    use chrono::TimeZone;
    use pretty_assertions::assert_eq;

    use super::*;

    fn config() -> ShortdateFormatConfig {
      ShortdateFormatConfig {
        today: "%H:%M".into(),
        this_week: "%a %H:%M".into(),
        this_month: "%m/%d %H:%M".into(),
        older: "%m/%d/%y %H:%M".into(),
      }
    }

    #[test]
    fn it_formats_older() {
      let datetime = Local.with_ymd_and_hms(2020, 6, 15, 14, 30, 0).unwrap();

      let result = FormattedShortdate::new(datetime, &config());

      assert_eq!(result.to_string(), "06/15/20 14:30");
    }

    #[test]
    fn it_formats_this_month() {
      let old = Local::now() - Duration::days(20);
      let datetime = Local
        .with_ymd_and_hms(old.year(), old.month(), old.day(), 14, 30, 0)
        .unwrap();

      let result = FormattedShortdate::new(datetime, &config());

      let expected = datetime.format("%m/%d %H:%M").to_string();
      assert_eq!(result.to_string(), expected);
    }

    #[test]
    fn it_formats_this_week() {
      let yesterday = Local::now() - Duration::days(2);
      let datetime = Local
        .with_ymd_and_hms(yesterday.year(), yesterday.month(), yesterday.day(), 14, 30, 0)
        .unwrap();

      let result = FormattedShortdate::new(datetime, &config());

      let expected = datetime.format("%a %H:%M").to_string();
      assert_eq!(result.to_string(), expected);
    }

    #[test]
    fn it_formats_cross_year_dates_as_older() {
      let now = Local::now();
      let last_year = now.year() - 1;
      let datetime = Local.with_ymd_and_hms(last_year, 11, 15, 14, 30, 0).unwrap();

      let result = FormattedShortdate::new(datetime, &config());

      assert_eq!(result.to_string(), format!("11/15/{} 14:30", last_year % 100));
    }

    #[test]
    fn it_formats_today() {
      let now = Local::now();
      let datetime = Local
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 14, 30, 0)
        .unwrap();

      let result = FormattedShortdate::new(datetime, &config());

      assert_eq!(result.to_string(), "14:30");
    }
  }
}
