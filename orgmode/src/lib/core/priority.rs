use std::{fmt::Display, str::FromStr};

#[derive(Debug, PartialEq)]
pub enum Priority {
    A,
    B,
    C,
}

impl Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            Priority::A => "(A)".to_string(),
            Priority::B => "(B)".to_string(),
            Priority::C => "(C)".to_string(),
        };
        write!(f, "{}", output)
    }
}

impl FromStr for Priority {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "(A)" => Ok(Priority::A),
            "(B)" => Ok(Priority::B),
            "(C)" => Ok(Priority::C),
            _ => Err(format!("Could not understand priority {s}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let expected = ["(A)".to_string(), "(C)".to_string(), "(B)".to_string()];

        for val in expected {
            let result: String = Priority::from_str(&val).unwrap().to_string();
            assert_eq!(val, result)
        }
    }

    #[test]
    fn roundtrip_bad() {
        let expected = ["".to_string(), "(D)".to_string()];

        for val in expected {
            let result = Priority::from_str(&val);
            assert!(result.is_err())
        }
    }
}
