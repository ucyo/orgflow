use std::{fmt::Display, str::FromStr};

#[derive(PartialEq, Debug)]
pub enum TaskState {
    Todo,
    Next,
    Hold(String),
    Wait(String),
    Done,
    Cancelled(String),
}

impl Default for TaskState {
    fn default() -> Self {
        TaskState::Todo
    }
}

impl Display for TaskState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            TaskState::Todo => "todo".to_string(),
            TaskState::Next => "next".to_string(),
            TaskState::Hold(comment) => format!("hold({comment})"),
            TaskState::Wait(comment) => format!("wait({comment})"),
            TaskState::Done => "done".to_string(),
            TaskState::Cancelled(comment) => format!("cancelled({comment})"),
        };
        write!(f, "{}", output)
    }
}

impl FromStr for TaskState {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("hold(") & s.ends_with(")") {
            let comment = s[..s.len() - 1].replace("hold(", "").to_string();
            Ok(Self::Hold(comment))
        } else if s.starts_with("wait(") & s.ends_with(")") {
            let comment = s[..s.len() - 1].replace("wait(", "").to_string();
            Ok(Self::Wait(comment))
        } else if s.starts_with("cancelled(") & s.ends_with(")") {
            let comment = s[..s.len() - 1].replace("cancelled(", "").to_string();
            Ok(Self::Cancelled(comment))
        } else {
            match s {
                "todo" => Ok(Self::Todo),
                "next" => Ok(Self::Next),
                "done" => Ok(Self::Done),
                _ => Err(format!("Can not understand state {s}")),
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
            "todo".to_string(),
            "cancelled(Problem with customer())".to_string(),
            "wait(Problem with customer)".to_string(),
            "wait()".to_string(),
            "cancelled(Wrong)".to_string(),
            "done".to_string(),
        ];

        for val in expected {
            let roundtrip: String = TaskState::from_str(&val).unwrap().to_string();
            assert_eq!(val, roundtrip);
        }
    }
    #[test]
    fn roundtrip_bad() {
        let expected = [
            "waiting".to_string(),
            "done(Wrong)".to_string(),
            "".to_string(),
        ];

        for val in expected {
            let roundtrip = TaskState::from_str(&val);
            assert!(roundtrip.is_err());
        }
    }
}
