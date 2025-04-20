//! This tests file must run in sequential mode since the environment

use orgmode::APP_PREFIX;
use orgmode::Configuration;
use std::env;

#[test]
fn read_from_file() {
    let loc = "tests/config";
    let config = Configuration::with(loc);

    assert_eq!(config.get_or("estimate", "45m"), "30m");
    assert_eq!(config.get_or("basefolder", ""), "/home/sweet/home");
    assert_eq!(config.get_or("awesome", "rust"), "rust");
}

#[test]
fn read_override_values_from_file() {
    let loc = "tests/config";
    let key = format!("{APP_PREFIX}_ESTIMATE");
    unsafe {
        env::set_var(key.clone(), "50m");
    }
    let config = Configuration::with(loc);
    unsafe {
        env::remove_var(key);
    }
    assert_eq!(config.get_or("estimate", "45m"), "50m");
    assert_eq!(config.get_or("basefolder", ""), "/home/sweet/home");
    assert_eq!(config.get_or("awesome", "rust"), "rust");
}

#[test]
fn change_config_file_via_env() {
    let key = format!("{APP_PREFIX}_CONFIG");
    unsafe {
        env::set_var(key.clone(), "tests/other_config");
    }
    let config = Configuration::from_env();
    unsafe {
        env::remove_var(key);
    }
    assert_eq!(config.get_or("estimate", "45m"), "60m");
    assert_eq!(config.get_or("basefolder", ""), "/home/sweet/home/away");
    assert_eq!(config.get_or("awesome", "rust"), "rust");
}
