use std::{fmt::Display, str::FromStr};

use chrono::TimeDelta;

#[derive(PartialEq, Debug)]
pub struct TaskRecurrence(TimeDelta, char);

impl Display for TaskRecurrence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.1 == 'y' {
            let years = self.0.num_weeks() / 52;
            write!(f, "{}y", years)
        } else if self.1 == 'w' {
            let weeks = self.0.num_weeks();
            write!(f, "{}w", weeks)
        } else {
            let days = self.0.num_days();
            write!(f, "{}d", days)
        }
    }
}

impl TaskRecurrence {
    fn with_days(days: u64) -> Self {
        Self(TimeDelta::days(days as i64), 'd')
    }
    fn with_weeks(weeks: u64) -> Self {
        Self(TimeDelta::weeks(weeks as i64), 'w')
    }
    fn with_years(years: u64) -> Self {
        Self(TimeDelta::weeks(years as i64 * 52), 'y')
    }
}

fn get_u64_or_err(val: &str, unit: &str) -> Result<u64, String> {
    if !val.ends_with(unit) {
        Err(format!("Expected unit '{unit}', found '{val}'."))
    } else {
        let endlength = val.len() - unit.len();
        match val[..endlength].parse() {
            Ok(val) => Ok(val),
            Err(msg) => Err(format!("Parsing number error: '{msg}'")),
        }
    }
}

impl FromStr for TaskRecurrence {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.ends_with("d") {
            let days = get_u64_or_err(s, "d")?;
            Ok(TaskRecurrence::with_days(days))
        } else if s.ends_with("w") {
            let weeks = get_u64_or_err(s, "w")?;
            Ok(TaskRecurrence::with_weeks(weeks))
        } else if s.ends_with("y") {
            let years = get_u64_or_err(s, "y")?;
            Ok(TaskRecurrence::with_years(years))
        } else {
            Err("Only [y]ears, [w]eeks and [d]ays are allowed for recurring tasks".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let expected = [
            "42w".to_string(),
            "21d".to_string(),
            "1d".to_string(),
            "9y".to_string(),
            "2w".to_string(),
            "7d".to_string(),
        ];

        for val in expected {
            let roundtrip: String = TaskRecurrence::from_str(&val).unwrap().to_string();
            assert_eq!(val, roundtrip);
        }
    }
    #[test]
    fn roundtrip_bad() {
        let expected = [
            "42min".to_string(),
            "21min".to_string(),
            "1sasdf".to_string(),
            "9ss".to_string(),
            "2asda".to_string(),
            ".4d".to_string(),
            ".4w".to_string(),
            "123.24y".to_string(),
            ".4w".to_string(),
        ];

        for val in expected {
            let result = TaskRecurrence::from_str(&val);
            assert!(result.is_err());
        }
    }
}
