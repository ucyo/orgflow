use orgflow::Configuration;
use std::env;

#[test]
fn test_basefolder_default() {
    // Save current env var
    let original = env::var("ORGFLOW_BASEFOLDER").ok();
    
    // Remove env var if set
    unsafe {
        env::remove_var("ORGFLOW_BASEFOLDER");
    }
    
    assert_eq!(Configuration::basefolder(), "/home/sweet/home");
    
    // Restore original env var if it existed
    if let Some(value) = original {
        unsafe {
            env::set_var("ORGFLOW_BASEFOLDER", value);
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
