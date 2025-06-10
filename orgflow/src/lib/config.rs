use std::env;

pub struct Configuration;

impl Configuration {
    pub fn basefolder() -> String {
        env::var("ORGFLOW_BASEFOLDER").unwrap_or_else(|_| {
            // Try to use a more reliable default path
            if let Some(home) = env::var_os("HOME") {
                format!("{}/orgflow", home.to_string_lossy())
            } else {
                // Fallback to current directory if HOME is not available
                "./orgflow".to_string()
            }
        })
    }
}
