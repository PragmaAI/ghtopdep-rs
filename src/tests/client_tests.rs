use crate::client::create_client;
use mockito::{mock, server_url};

#[test]
fn test_client_creation() {
    let client = create_client();
    assert!(client.is_ok());
}

#[tokio::test]
async fn test_fetch_with_retry_success() {
    use crate::client::fetch_with_retry;
    
    let _m = mock("GET", "/test")
        .with_status(200)
        .with_header("content-type", "text/plain")
        .with_body("test response")
        .create();
    
    let client = create_client().unwrap();
    let url = &format!("{}/test", server_url());
    
    let result = fetch_with_retry(&client, url, 1).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "test response");
}

#[tokio::test]
async fn test_fetch_with_retry_error() {
    use crate::client::fetch_with_retry;
    
    let _m = mock("GET", "/error")
        .with_status(404)
        .create();
    
    let client = create_client().unwrap();
    let url = &format!("{}/error", server_url());
    
    let result = fetch_with_retry(&client, url, 1).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_fetch_with_retry_rate_limit() {
    use crate::client::fetch_with_retry;
    use tokio::time::timeout;
    use std::time::Duration;
    
    let _m = mock("GET", "/rate-limit")
        .with_status(429)
        .create();
    
    let client = create_client().unwrap();
    let url = &format!("{}/rate-limit", server_url());
    
    // This should timeout because it will retry with backoff
    let result = timeout(Duration::from_millis(100), fetch_with_retry(&client, url, 1)).await;
    assert!(result.is_err()); // Timeout error
} 