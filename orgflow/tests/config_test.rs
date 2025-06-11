use orgflow::Configuration;
use std::env;

#[test]
fn test_basefolder_default() {
    // Save current env vars
    let original_basefolder = env::var("ORGFLOW_BASEFOLDER").ok();
    let original_home = env::var("HOME").ok();

    // Remove env vars if set
    unsafe {
        env::remove_var("ORGFLOW_BASEFOLDER");
        env::remove_var("HOME");
    }

    // With no env vars, it should use ./orgflow
    let result = Configuration::basefolder();
    assert_eq!(result, "./orgflow");

    // Restore original env vars if they existed
    if let Some(value) = original_basefolder {
        unsafe {
            env::set_var("ORGFLOW_BASEFOLDER", value);
        }
    }
    if let Some(value) = original_home {
        unsafe {
            env::set_var("HOME", value);
        }
    }
}

#[test]
fn test_basefolder_from_env() {
    // Save current env var
    let original = env::var("ORGFLOW_BASEFOLDER").ok();

    unsafe {
        env::set_var("ORGFLOW_BASEFOLDER", "/custom/path");
    }
    assert_eq!(Configuration::basefolder(), "/custom/path");

    // Restore original env var or remove if it didn't exist
    if let Some(value) = original {
        unsafe {
            env::set_var("ORGFLOW_BASEFOLDER", value);
        }
    } else {
        unsafe {
            env::remove_var("ORGFLOW_BASEFOLDER");
        }
    }
}
