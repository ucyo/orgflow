use std::fmt::Display;
use std::str::FromStr;

use super::dates::Date;
use super::priority::Priority;
use super::tags::Tag;
use super::tags::TagCollection;

#[derive(Debug, PartialEq)]
pub struct Task {
    is_completed: bool,
    priority_level: Option<Priority>,
    completion_date: Option<Date>,
    creation_date: Option<Date>,
    description: String,
    tags: Option<TagCollection>,
}

impl Task {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn is_completed(&self) -> bool {
        self.is_completed
    }
    
    pub fn priority_level(&self) -> &Option<Priority> {
        &self.priority_level
    }
    
    pub fn completion_date(&self) -> &Option<Date> {
        &self.completion_date
    }
    
    pub fn creation_date(&self) -> &Option<Date> {
        &self.creation_date
    }
    
    pub fn description(&self) -> &str {
        &self.description
    }
    
    pub fn tags(&self) -> &Option<TagCollection> {
        &self.tags
    }
    pub fn with_task(description: String) -> Self {
        Self {
            description,
            ..Default::default()
        }
    }
    pub fn with_today(description: &str) -> Self {
        let mut t = Self::from_str(description).unwrap();
        t.creation_date = Some(Date::now());
        t
    }
}

fn _is_prefix(s: &str) -> bool {
    Priority::from_str(s).is_ok() | Date::from_str(s).is_ok() | (s == "x")
}

fn _is_suffix(s: &str) -> bool {
    Tag::from_str(s).is_ok()
}

impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = Vec::new();
        if self.is_completed {
            result.push("x".to_string());
        }
        match &self.priority_level {
            Some(prio) => result.push(prio.to_string()),
            None => {}
        }
        match &self.completion_date {
            Some(d) => result.push(d.to_string()),
            None => {}
        }
        match &self.creation_date {
            Some(cd) => result.push(cd.to_string()),
            None => {}
        }
        result.push(self.description.clone());
        match &self.tags {
            Some(tags) => result.push(tags.to_string()),
            None => {}
        }

        write!(f, "{}", result.join(" "))
    }
}

impl Default for Task {
    fn default() -> Self {
        Task {
            is_completed: false,
            priority_level: None,
            completion_date: None,
            creation_date: Some(Date::now()),
            description: String::new(),
            tags: None,
        }
    }
}

impl FromStr for Task {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().is_empty() {
            return Err("Empty String error".to_string());
        }
        let mut task = Task::default();
        let mut prefix: Vec<&str> = Vec::new();
        let mut description: Vec<&str> = Vec::new();
        let mut suffix: Vec<&str> = Vec::new();

        let mut remaining = s.trim();
        while let Some((head, tail)) = remaining.split_once(" ") {
            if _is_prefix(head) & description.is_empty() {
                prefix.push(head);
                remaining = tail.trim();
            } else if _is_suffix(head) {
                suffix.push(head);
                let mut remainder: Vec<&str> = tail.split(" ").collect();
                suffix.append(&mut remainder);
                remaining = "";
            } else {
                description.push(head);
                remaining = tail.trim();
            }
        }
        if _is_prefix(remaining) {
            prefix.push(remaining);
        } else if _is_suffix(remaining) {
            suffix.push(remaining);
        } else {
            description.push(remaining);
        }
        if description.is_empty() {
            return Err("There must be a task description!".to_string());
        }
        let _ = process_prefix(&mut prefix, &mut task)?;
        task.description = description.join(" ").trim().to_string();
        if !suffix.is_empty() {
            task.tags = Some(TagCollection::from_str(&suffix.join(" "))?);
        }
        Ok(task)
    }
}

fn process_prefix(prefix: &mut Vec<&str>, task: &mut Task) -> Result<(), String> {
    let mut iter = prefix.iter();
    let mut completion_date: Option<Date> = None;
    let mut creation_date: Option<Date> = None;
    let mut priority: Option<Priority> = None;
    let mut is_done = false;

    while let Some(val) = iter.next() {
        if (val == &"x") & !is_done {
            is_done = true;
        } else if Priority::from_str(val).is_ok() {
            priority = Priority::from_str(val).ok();
        } else if Date::from_str(val).is_ok() & creation_date.is_some() & completion_date.is_none()
        {
            // one date was parsed, now we have a second date parsing
            // the first must have been the completion date
            completion_date = creation_date;
            creation_date = Date::from_str(val).ok();
        } else if Date::from_str(val).is_ok() & completion_date.is_none() {
            creation_date = Date::from_str(val).ok();
        } else {
            return Err(format!("Error parsing prefix '{}'", val));
        }
    }

    task.is_completed = is_done;
    task.priority_level = priority;
    task.completion_date = completion_date;
    task.creation_date = creation_date;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let expected = [
            "(A) Try to fix a mistake in the code p:freddy".to_string(),
            "x (A) 2025-03-12 Try to fix a mistake in the code".to_string(),
            "x 2025-11-12 Try to fix a mistake in the code p:pes @phone +aid".to_string(),
            // it cannot react to bad formatted tag if it is the first entry
            // here it will think it is part of the description
            "x 2025-11-12 2022-11-12 Task description rec:+24".to_string(),
            // it can also not recognize badly formatted date
            // because it will simply use it as part of the text
            "x 2025-44-44 2022-11-12 Task description rec:+24".to_string(),
        ];

        for val in expected {
            println!("{}", val);
            let task = Task::from_str(&val).unwrap();
            let roundtrip = task.to_string();
            assert_eq!(val, roundtrip, "Roundtrip error w/ '{:?}'", task);
        }
    }

    #[test]
    fn roundtrip_bad() {
        let expected = [
            "".to_string(),
            "x (A) @phone".to_string(),
            "x 2025-11-12 2022-11-12 Task description p:pes @phone +aid rec:+24".to_string(),
            "x 2025-11-12 2022-11-12 Task description p:pes rec:+24w rec:23o".to_string(),
            "x 2025-11-12 2022-11-12 Task description p:pes rec:+24".to_string(),
            "x x x 2022-11-12 Task description rec:+24".to_string(),
            "x 2026-11-12 2027-11-12 2028-11-12 2029-11-12 Task description rec:+24".to_string(),
        ];

        for val in expected {
            let result = Task::from_str(&val);
            assert!(result.is_err(), "\n{}\n=>\n{:?}\n", val, result);
        }
    }
}
