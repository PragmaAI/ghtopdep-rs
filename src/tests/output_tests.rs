use crate::config::Config;
use crate::dependent::Dependent;
use crate::output::display_results;
use std::str;

// Helper function to create a test config
fn create_test_config(format: &str) -> Config {
    Config {
        owner: "test".to_string(),
        repo: "repo".to_string(),
        top_n: 10,
        max_pages: 1,
        min_stars: 0.0,
        is_package: false,
        show_desc: false,
        use_cache: false,
        output_format: format.to_string(),
    }
}

// Helper function to create test dependents
fn create_test_dependents() -> Vec<Dependent> {
    vec![
        Dependent {
            repo: "user1/repo1".to_string(),
            stars: "100".to_string(),
            description: None,
        },
        Dependent {
            repo: "user2/repo2".to_string(),
            stars: "200".to_string(),
            description: Some("Test description".to_string()),
        },
    ]
}

#[test]
fn test_json_output() {
    let config = create_test_config("json");
    let dependents = create_test_dependents();
    
    let result = display_results(
        &config,
        &dependents,
        10,
        5,
        20,
        1.5,
    );
    
    assert!(result.is_ok());
    // We can't easily capture stdout, but we can verify no errors occurred
}

#[test]
fn test_table_output() {
    let config = create_test_config("table");
    let dependents = create_test_dependents();
    
    let result = display_results(
        &config,
        &dependents,
        10,
        5,
        20,
        1.5,
    );
    
    assert!(result.is_ok());
}

#[test]
fn test_text_output() {
    let config = create_test_config("text");
    let dependents = create_test_dependents();
    
    let result = display_results(
        &config,
        &dependents,
        10,
        5,
        20,
        1.5,
    );
    
    assert!(result.is_ok());
}

#[test]
fn test_empty_dependents() {
    let config = create_test_config("text");
    let dependents: Vec<Dependent> = vec![];
    
    let result = display_results(
        &config,
        &dependents,
        0,
        0,
        0,
        1.5,
    );
    
    assert!(result.is_ok());
} 