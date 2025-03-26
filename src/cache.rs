use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use dirs::cache_dir;
use crypto::digest::Digest;
use crypto::md5::Md5;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::{Read, Write};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

const CACHE_EXPIRY_HOURS: u64 = 24;

#[derive(Debug, Serialize, Deserialize)]
pub struct CachedResponse {
    pub timestamp: u64,
    pub content: String,
}

pub fn get_cache_dir() -> PathBuf {
    let mut cache_path = cache_dir().unwrap_or_else(|| PathBuf::from("./cache"));
    cache_path.push("gh_get_dependent");
    fs::create_dir_all(&cache_path).unwrap_or_else(|_| {
        println!("Warning: Could not create cache directory");
    });
    cache_path
}

pub fn get_cache_path(url: &str) -> PathBuf {
    let mut hasher = Md5::new();
    hasher.input_str(url);
    let url_hash = hasher.result_str();
    
    let mut path = get_cache_dir();
    path.push(format!("{}.json", url_hash));
    path
}

pub fn is_cache_valid(cache_path: &PathBuf) -> bool {
    if !cache_path.exists() {
        return false;
    }
    
    let metadata = match fs::metadata(cache_path) {
        Ok(m) => m,
        Err(_) => return false,
    };
    
    let modified = match metadata.modified() {
        Ok(time) => time,
        Err(_) => return false,
    };
    
    let now = SystemTime::now();
    match now.duration_since(modified) {
        Ok(duration) => duration.as_secs() < CACHE_EXPIRY_HOURS * 3600,
        Err(_) => false,
    }
}

pub fn read_from_cache(cache_path: &PathBuf) -> Result<String, AppError> {
    let file = fs::File::open(cache_path)?;
    let mut decoder = GzDecoder::new(file);
    let mut cached_data = String::new();
    decoder.read_to_string(&mut cached_data)?;
    
    let cached: CachedResponse = serde_json::from_str(&cached_data)?;
    Ok(cached.content)
}

pub fn write_to_cache(cache_path: &PathBuf, content: &str) -> Result<(), AppError> {
    let cached = CachedResponse {
        timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        content: content.to_string(),
    };
    
    let json = serde_json::to_string(&cached)?;
    let file = fs::File::create(cache_path)?;
    let mut encoder = GzEncoder::new(file, Compression::default());
    encoder.write_all(json.as_bytes())?;
    
    Ok(())
} 