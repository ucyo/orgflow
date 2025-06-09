use std::{fmt::Display, str::FromStr};

use uuid::Uuid;
#[derive(PartialEq, Debug)]
pub struct Guid(Uuid);

impl Guid {
    pub fn new() -> Self {
        Guid(Uuid::new_v4())
    }
}

impl Display for Guid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

impl FromStr for Guid {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tmp = Uuid::parse_str(s);
        match tmp {
            Ok(uid) => Ok(Guid(uid)),
            Err(msg) => Err(format!("Can not parse uuid: {msg}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let expected = ["a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8".to_string()];
        for val in expected {
            let result: String = Guid::from_str(&val).unwrap().to_string();
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
            let result = Guid::from_str(&val);
            assert!(result.is_err())
        }
    }
}
