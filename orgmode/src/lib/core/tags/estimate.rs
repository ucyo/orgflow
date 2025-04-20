use std::{fmt::Display, str::FromStr, time::Duration};

#[derive(PartialEq, Debug)]
pub struct TaskEstimate(Duration);

impl TaskEstimate {
    fn new(minutes: u64) -> Self {
        let d = Duration::from_secs(minutes * 60);
        TaskEstimate(d)
    }
}

impl Display for TaskEstimate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}min", self.0.as_secs() / 60)
    }
}

impl FromStr for TaskEstimate {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.ends_with("min") {
            Err("String has to end with `min`".to_string())
        } else {
            match s.replace("min", "").parse() {
                Ok(min) => Ok(TaskEstimate::new(min)),
                Err(msg) => Err(format!("Could not convert number to u64: {msg}")),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let expected = [
            "13min".to_string(),
            "120min".to_string(),
            "11min".to_string(),
            "114min".to_string(),
            "121min".to_string(),
        ];
        for val in expected {
            let result: String = TaskEstimate::from_str(&val).unwrap().to_string();
            assert_eq!(result, val)
        }
    }
    #[test]
    fn roundtrip_bad() {
        let expected = [
            "213.12min".to_string(),
            "31.1min".to_string(),
            ".1min".to_string(),
            "213".to_string(),
            "2sec".to_string(),
            "23weeks".to_string(),
            "21h".to_string(),
        ];
        for val in expected {
            let result = TaskEstimate::from_str(&val);
            assert!(result.is_err())
        }
    }
}
