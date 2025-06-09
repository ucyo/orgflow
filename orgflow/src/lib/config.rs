use std::env;

pub struct Configuration;

impl Configuration {
    pub fn basefolder() -> String {
        env::var("ORGFLOW_BASEFOLDER").unwrap_or_else(|_| "/home/sweet/home".to_string())
    }
}
