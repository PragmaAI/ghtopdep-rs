use reqwest::{Client, header};
use tokio::time::{sleep, Duration};
use crate::error::AppError;

pub fn create_client() -> Result<Client, AppError> {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"),
    );
    
    let client = Client::builder()
        .default_headers(headers)
        .build()?;
    
    Ok(client)
}

pub async fn fetch_with_retry(
    client: &Client, 
    url: &str, 
    max_retries: usize
) -> Result<String, AppError> {
    let mut retries = 0;
    let mut delay = 1;
    
    loop {
        match client.get(url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    return Ok(response.text().await?);
                } else if response.status().as_u16() == 429 {
                    // Rate limited - exponential backoff
                    if retries >= max_retries {
                        return Err(AppError::Other(format!(
                            "Rate limited after {} retries", retries
                        )));
                    }
                    
                    println!("Rate limited, retrying in {} seconds...", delay);
                    sleep(Duration::from_secs(delay)).await;
                    delay *= 2; // Exponential backoff
                    retries += 1;
                    continue;
                } else {
                    return Err(AppError::Other(format!(
                        "HTTP error: {}", response.status()
                    )));
                }
            },
            Err(e) => {
                if retries >= max_retries {
                    return Err(e.into());
                }
                
                println!("Network error, retrying in {} seconds: {}", delay, e);
                sleep(Duration::from_secs(delay)).await;
                delay *= 2;
                retries += 1;
            }
        }
    }
} 