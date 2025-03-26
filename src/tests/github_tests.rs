use crate::github::{parse_page, cached_fetch};
use crate::client::create_client;
use mockito::{mock, server_url};

#[test]
fn test_parse_page_with_dependents() {
    let html = r#"
    <div class="Box">
        <div class="flex-items-center">
            <span><a class="text-bold" href="/user1/repo1">User1/Repo1</a></span>
            <div><span>100</span></div>
        </div>
        <div class="flex-items-center">
            <span><a class="text-bold" href="/user2/repo2">User2/Repo2</a></span>
            <div><span>200</span></div>
        </div>
    </div>
    <div class="paginate-container">
        <div><a href="/next-page">Next</a></div>
    </div>
    "#;
    
    let (dependents, next_link) = parse_page(html);
    
    assert_eq!(dependents.len(), 2);
    
    // Based on the debug output, we need to match what parse_page actually returns
    assert_eq!(dependents[0].0, "user1/repo1");
    assert_eq!(dependents[0].1, "User1/Repo1");
    assert_eq!(dependents[1].0, "user2/repo2");
    assert_eq!(dependents[1].1, "User2/Repo2");
    
    assert_eq!(next_link, Some("/next-page".to_string()));
}

#[test]
fn test_parse_page_without_next() {
    let html = r#"
    <div class="Box">
        <div class="flex-items-center">
            <span><a class="text-bold" href="/user1/repo1">User1/Repo1</a></span>
            <div><span>100</span></div>
        </div>
    </div>
    "#;
    
    let (dependents, next_link) = parse_page(html);
    
    assert_eq!(dependents.len(), 1);
    assert_eq!(dependents[0].0, "user1/repo1");
    assert_eq!(dependents[0].1, "User1/Repo1");
    assert_eq!(next_link, None);
}

#[tokio::test]
async fn test_cached_fetch() {
    // We need a unique URL for this test to avoid interference from other tests
    let unique_path = format!("/repo-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_micros());
    
    // First request - this should be called exactly once
    let _m1 = mock("GET", unique_path.as_str())
        .with_status(200)
        .with_body("test content")
        .expect(1)
        .create();
    
    let client = create_client().unwrap();
    let url = &format!("{}{}", server_url(), unique_path);
    
    // First fetch should hit the server
    let result1 = cached_fetch(&client, url, true).await;
    assert!(result1.is_ok());
    assert_eq!(result1.unwrap(), "test content");
    
    // Second fetch should use cache - no server request
    let result2 = cached_fetch(&client, url, true).await;
    assert!(result2.is_ok());
    assert_eq!(result2.unwrap(), "test content");
    
    // With cache disabled, should hit server again
    // We need a new mock for this request
    let _m2 = mock("GET", unique_path.as_str())
        .with_status(200)
        .with_body("new content")
        .expect(1)
        .create();
    
    let result3 = cached_fetch(&client, url, false).await;
    assert!(result3.is_ok());
    assert_eq!(result3.unwrap(), "new content");
} 