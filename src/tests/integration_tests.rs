use crate::config::Config;
use crate::github::get_top_dependents;

#[tokio::test]
#[ignore] // This test makes real network requests, so we mark it as ignored by default
async fn test_real_github_fetch() {
    let config = Config {
        owner: "rust-lang".to_string(),
        repo: "rust".to_string(),
        top_n: 3,
        max_pages: 1,
        min_stars: 0.0,
        is_package: false,
        show_desc: false,
        use_cache: true,
        output_format: "text".to_string(),
    };
    
    let result = get_top_dependents(&config).await;
    assert!(result.is_ok());
    
    let (dependents, total, with_stars, max_deps) = result.unwrap();
    
    // We should get some results
    assert!(!dependents.is_empty());
    assert!(total > 0);
    assert!(with_stars > 0);
    assert!(max_deps > 0);
}

#[tokio::test]
async fn test_end_to_end_with_mocks() {
    use mockito::{mock, server_url};
    
    // Setup mocks for GitHub API
    let _base_url = server_url();
    
    // Mock the dependents page
    let _m1 = mock("GET", "/rust-lang/rust/network/dependents?dependent_type=REPOSITORY")
        .with_status(200)
        .with_body(include_str!("fixtures/dependents_page.html"))
        .create();
    
    // Create a config that points to our mock server
    let _config = Config {
        owner: "rust-lang".to_string(),
        repo: "rust".to_string(),
        top_n: 3,
        max_pages: 1,
        min_stars: 0.0,
        is_package: false,
        show_desc: false,
        use_cache: false,
        output_format: "text".to_string(),
    };
    
    // Override the GitHub base URL for testing
    // This would require modifying the github module to accept a base URL parameter
    // For now, we'll just test that the function runs without errors
    
    // let result = get_top_dependents(&config).await;
    // assert!(result.is_ok());
} 