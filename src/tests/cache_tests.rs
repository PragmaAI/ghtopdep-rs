use std::fs;
use crate::cache::{get_cache_path, is_cache_valid, write_to_cache, read_from_cache};

#[test]
fn test_cache_path_generation() {
    let url = "https://github.com/test/repo";
    let path = get_cache_path(url);
    
    // The path should be a valid file path
    assert!(path.is_absolute() || path.starts_with("./cache"));
    
    // Different URLs should generate different paths
    let url2 = "https://github.com/test/repo2";
    let path2 = get_cache_path(url2);
    assert_ne!(path, path2);
}

#[test]
fn test_cache_write_read() {
    let test_content = "Test content";
    let test_url = "https://test.example.com/test";
    let cache_path = get_cache_path(test_url);
    
    // Clean up any existing test file
    if cache_path.exists() {
        fs::remove_file(&cache_path).unwrap();
    }
    
    // Write to cache
    write_to_cache(&cache_path, test_content).unwrap();
    
    // Verify the file exists
    assert!(cache_path.exists());
    
    // Read from cache
    let content = read_from_cache(&cache_path).unwrap();
    assert_eq!(content, test_content);
    
    // Clean up
    fs::remove_file(&cache_path).unwrap();
}

#[test]
fn test_cache_validity() {
    let test_url = "https://test.example.com/validity";
    let cache_path = get_cache_path(test_url);
    
    // Clean up any existing test file
    if cache_path.exists() {
        fs::remove_file(&cache_path).unwrap();
    }
    
    // Non-existent file should be invalid
    assert!(!is_cache_valid(&cache_path));
    
    // Write to cache
    write_to_cache(&cache_path, "test").unwrap();
    
    // Fresh cache should be valid
    assert!(is_cache_valid(&cache_path));
    
    // Clean up
    fs::remove_file(&cache_path).unwrap();
}