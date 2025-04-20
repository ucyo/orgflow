use config::Config;
use std::collections::HashMap;
use std::env;

pub const APP_PREFIX: &str = "ORGMODE";
const APP_DEFAULT_CONFIG_BASENAME: &str = "config";
const APP_CONFIGFILE_SUFFIX: &str = "CONFIG";

pub struct Configuration {
    prefix: String,
    path: String,
    pub content: HashMap<String, String>,
}

impl Configuration {
    fn _base() -> Self {
        Configuration {
            prefix: APP_PREFIX.to_string(),
            path: APP_DEFAULT_CONFIG_BASENAME.to_string(),
            content: HashMap::new(),
        }
    }
    fn parse_config(&mut self) {
        let settings = Config::builder()
            // Add in `./Settings.toml`
            .add_source(config::File::with_name(&self.path))
            // Add in settings from the environment (with a prefix of APP)
            // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
            .add_source(config::Environment::with_prefix(&self.prefix))
            .build()
            .unwrap();
        self.content = settings
            .try_deserialize::<HashMap<String, String>>()
            .unwrap()
    }
    pub fn from_env() -> Self {
        let key = format!("{APP_PREFIX}_{APP_CONFIGFILE_SUFFIX}");
        let path = match env::var(key) {
            Ok(val) => val,
            Err(_) => APP_DEFAULT_CONFIG_BASENAME.to_string(),
        };
        Self::with(&path)
    }
    pub fn with(base: &str) -> Self {
        let mut c = Configuration {
            path: base.to_string(),
            ..Self::_base()
        };
        c.parse_config();
        c
    }
    pub fn get_or(&self, key: &str, default: &str) -> String {
        self.content
            .get(key)
            .unwrap_or(&default.to_string())
            .to_string()
    }
}
