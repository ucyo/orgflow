mod estimate;
pub mod guid;
mod recurrence;
mod state;

use std::{fmt::Display, str::FromStr};

use super::dates::Date;
use estimate::TaskEstimate;
use guid::Guid;
use recurrence::TaskRecurrence;
use state::TaskState;

#[derive(PartialEq, Debug)]
pub enum Tag {
    /// Prefix `s:`
    Status(TaskState),
    /// Prefix `est:`
    Estimate(TaskEstimate),
    /// Prefix `rec:+`
    StrictRecurrence(TaskRecurrence),
    /// Prefix `rec:`
    LooseRecurrence(TaskRecurrence),
    /// Prefix `t:`
    Threshold(Date),
    /// Prefix `n:`
    Note(Guid),
    /// Prefix `p:`
    Person(String),
    /// Prefix `!`
    OneOff(String),
    /// Prefix `@`
    Context(String),
    /// Prefix `+`
    Project(String),
    /// Prefix `key:value`
    Custom(String, String),
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            Tag::Status(state) => format!("s:{state}"),
            Tag::Estimate(estimate) => format!("est:{estimate}"),
            Tag::StrictRecurrence(rec) => format!("rec:+{rec}"),
            Tag::LooseRecurrence(rec) => format!("rec:{rec}"),
            Tag::Threshold(date) => format!("t:{date}"),
            Tag::Note(note) => format!("n:{note}"),
            Tag::Person(p) => format!("p:{p}"),
            Tag::OneOff(source) => format!("!{source}"),
            Tag::Context(ctx) => format!("@{ctx}"),
            Tag::Project(project) => format!("+{project}"),
            Tag::Custom(key, value) => format!("{key}:{value}"),
        };
        write!(f, "{}", output)
    }
}

impl FromStr for Tag {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("s:") {
            let s = s.replace("s:", "");
            Ok(Tag::Status(TaskState::from_str(&s)?))
        } else if s.starts_with("est:") {
            let s = s.replace("est:", "");
            let est = TaskEstimate::from_str(&s)?;
            Ok(Tag::Estimate(est))
        } else if s.starts_with("rec:+") {
            let s = s.replace("rec:+", "");
            let r = TaskRecurrence::from_str(&s)?;
            Ok(Tag::StrictRecurrence(r))
        } else if s.starts_with("rec:") {
            let s = s.replace("rec:", "");
            let r = TaskRecurrence::from_str(&s)?;
            Ok(Tag::LooseRecurrence(r))
        } else if s.starts_with("t:") {
            let s = s.replace("t:", "");
            Ok(Tag::Threshold(Date::from_str(&s)?))
        } else if s.starts_with("n:") {
            let s = s.replace("n:", "");
            let n = Guid::from_str(&s)?;
            Ok(Tag::Note(n))
        } else if s.starts_with("p:") {
            Ok(Tag::Person(s.replace("p:", "").to_string()))
        } else if s.starts_with("!") {
            Ok(Tag::OneOff(s.replace("!", "")))
        } else if s.starts_with("@") {
            Ok(Tag::Context(s.replace("@", "")))
        } else if s.starts_with("+") {
            Ok(Tag::Project(s.replace("+", "")))
        } else if s.contains(":") {
            let (key, val) = s.split_once(":").unwrap();
            Ok(Tag::Custom(
                key.to_string().to_lowercase(),
                val.to_string().to_lowercase(),
            ))
        } else {
            Err("No tag found".to_string())
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct TagCollection(Vec<Tag>);

impl TagCollection {
    pub fn new() -> Self {
        TagCollection(Vec::new())
    }
}

impl Default for TagCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for TagCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = self
            .0
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(" ");
        write!(f, "{}", result)
    }
}

impl FromStr for TagCollection {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.replace(" ", "").len() == 0 {
            Err("Empty String error".to_string())
        } else {
            let mut result = Vec::new();
            for x in s.split_whitespace() {
                let t = Tag::from_str(x)?;
                result.push(t)
            }
            Ok(TagCollection(result))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let expected = ["+project1 p:mueller poor:speed".to_string()];

        for val in expected {
            let result = TagCollection::from_str(&val).unwrap();
            let expected = TagCollection(vec![
                Tag::Project("project1".to_string()),
                Tag::Person("mueller".to_string()),
                Tag::Custom("poor".to_string(), "speed".to_string()),
            ]);

            assert_eq!(result, expected);
            let roundtrip: String = expected.to_string();
            assert_eq!(val, roundtrip);
        }
    }
    #[test]
    fn roundtrip_bad() {
        let expected = [
            "Hello".to_string(),
            "This is a task".to_string(),
            "h: sdfasd dsf".to_string(),
            "@ sdfasd dsf".to_string(),
            "+sdfasd dsf".to_string(),
            " ".to_string(),
            "".to_string(),
            "          ".to_string(),
        ];

        for val in expected {
            let result = TagCollection::from_str(&val);
            assert!(result.is_err(), "'{}'", result.unwrap());
        }
    }

    #[test]
    fn empty_tag() {
        let result = Tag::from_str(" ");
        assert!(result.is_err());
        let result = Tag::from_str("");
        assert!(result.is_err());
        let result = Tag::from_str("       ");
        assert!(result.is_err());
    }
}
