use std::{fmt::Display, str::FromStr};

use chrono::{Datelike, Local, NaiveDate};
#[derive(PartialEq, Debug)]
pub struct Date(NaiveDate);

impl Date {
    pub fn now() -> Self {
        Date(Local::now().date_naive())
    }
}

impl Default for Date {
    fn default() -> Self {
        Self::now()
    }
}

impl Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let y = self.0.year();
        let m = self.0.month();
        let d = self.0.day();
        write!(f, "{:02}-{:02}-{:02}", y, m, d)
    }
}

impl FromStr for Date {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let fmt = "%Y-%m-%d";
        let nd = NaiveDate::parse_from_str(s, fmt);
        match nd {
            Ok(v) => Ok(Date(v)),
            Err(msg) => Err(format!("Only '{fmt}' format allowed: {msg}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let expected = [
            "2024-01-09".to_string(),
            "2002-12-14".to_string(),
            "2012-11-02".to_string(),
            "1928-01-21".to_string(),
            "1983-11-23".to_string(),
        ];
        for val in expected {
            let result: String = Date::from_str(&val).unwrap().to_string();
            assert_eq!(result, val)
        }
    }
    #[test]
    fn roundtrip_bad() {
        let expected = [
            "2029-14-09".to_string(),
            "2024/01/09".to_string(),
            "2024".to_string(),
        ];
        for val in expected {
            let result = Date::from_str(&val);
            assert!(result.is_err())
        }
    }
}
