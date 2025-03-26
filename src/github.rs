use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use scraper::{Html, Selector};
use tokio::sync::Semaphore;
use tokio::time::sleep;

use crate::cache::{get_cache_path, is_cache_valid, read_from_cache, write_to_cache};
use crate::client::{create_client, fetch_with_retry};
use crate::config::Config;
use crate::dependent::{Dependent, convert_stars_to_number};
use crate::error::AppError;

const GITHUB_BASE_URL: &str = "https://github.com";
const REPOS_PER_PAGE: usize = 30;

pub async fn cached_fetch(client: &Client, url: &str, use_cache: bool) -> Result<String, AppError> {
    let cache_path = get_cache_path(url);
    
    if use_cache && is_cache_valid(&cache_path) {
        match read_from_cache(&cache_path) {
            Ok(content) => return Ok(content),
            Err(e) => println!("Warning: Cache read error: {}", e),
        }
    }
    
    // Fetch and cache with compression
    let html = fetch_with_retry(client, url, 3).await?;
    
    if use_cache {
        if let Err(e) = write_to_cache(&cache_path, &html) {
            println!("Warning: Cache write error: {}", e);
        }
    }
    
    Ok(html)
}

pub async fn get_max_deps(client: &Client, url: &str, dependent_type: &str, use_cache: bool) -> usize {
    let full_url = format!("{}?dependent_type={}", url, dependent_type);
    
    match cached_fetch(client, &full_url, use_cache).await {
        Ok(html) => {
            let document = Html::parse_document(&html);
            let selector = Selector::parse(".table-list-header-toggle .btn-link.selected").unwrap();
            
            if let Some(element) = document.select(&selector).next() {
                let text = element.text().collect::<Vec<_>>().join("");
                let count_str = text.trim().split_whitespace().next().unwrap_or("0");
                return count_str.replace(',', "").parse::<usize>().unwrap_or(0);
            }
        },
        Err(e) => {
            println!("Error getting max deps: {}", e);
        }
    }
    
    0
}

pub async fn get_repo_description(client: &Client, repo_path: &str, use_cache: bool) -> Option<String> {
    let url = format!("{}/{}", GITHUB_BASE_URL, repo_path);
    
    match cached_fetch(client, &url, use_cache).await {
        Ok(html) => {
            let document = Html::parse_document(&html);
            let selector = Selector::parse("div.BorderGrid-cell p").unwrap();
            
            if let Some(element) = document.select(&selector).next() {
                let text = element.text().collect::<Vec<_>>().join("");
                return Some(text.trim().to_string());
            }
        },
        Err(e) => {
            println!("Error getting description for {}: {}", repo_path, e);
        }
    }
    
    None
}

pub fn parse_page(html: &str) -> (Vec<(String, String)>, Option<String>) {
    let document = Html::parse_document(html);
    let repo_selector = Selector::parse(".flex-items-center").unwrap();
    let link_selector = Selector::parse("a.text-bold").unwrap();
    let stars_selector = Selector::parse("div span").unwrap();
    let next_selector = Selector::parse(".paginate-container a").unwrap();
    
    let mut dependents = Vec::new();
    
    for element in document.select(&repo_selector) {
        if let Some(link_element) = element.select(&link_selector).next() {
            if let Some(href) = link_element.value().attr("href") {
                let repo = href.trim_start_matches('/').to_lowercase();
                
                if let Some(stars_element) = element.select(&stars_selector).next() {
                    let stars = stars_element.text().collect::<String>().trim().to_string();
                    dependents.push((repo, stars));
                }
            }
        }
    }
    
    let next_link = document.select(&next_selector).next()
        .and_then(|el| el.value().attr("href"))
        .map(|href| href.to_string());
    
    (dependents, next_link)
}

pub async fn get_top_dependents(
    config: &Config,
) -> Result<(Vec<Dependent>, usize, usize, usize), Box<dyn std::error::Error>> {
    let base_url = format!("{}/{}/{}/network/dependents", 
        GITHUB_BASE_URL, config.owner, config.repo);
    let mut page_url = format!("{}?dependent_type={}", base_url, config.dependent_type());
    
    let client = create_client()?;
    
    let mut all_dependents = Vec::new();
    let mut page_count = 0;
    
    // Get the maximum number of dependents
    let max_deps = get_max_deps(&client, &base_url, config.dependent_type(), config.use_cache).await;
    let pb = if max_deps > 0 {
        println!("Found {} total dependents", max_deps);
        let total = std::cmp::min(max_deps, config.max_pages * REPOS_PER_PAGE);
        let pb = ProgressBar::new(total as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:50.cyan/blue}] {percent}% ({pos}/{len}) [{eta}]")
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏ "));
        pb
    } else {
        ProgressBar::new_spinner()
    };

    let deps_pb = pb.clone();

    while page_count < config.max_pages {
        page_count += 1;
        
        match cached_fetch(&client, &page_url, config.use_cache).await {
            Ok(html) => {
                let (deps, next_link) = parse_page(&html);
                if deps.is_empty() {
                    break;
                }
                
                all_dependents.extend(deps);
                pb.set_position(page_count as u64);
                deps_pb.set_position(all_dependents.len() as u64);
                
                if let Some(link) = next_link {
                    // Fix the URL construction
                    if link.starts_with("http") {
                        page_url = link;
                    } else if link.starts_with('/') {
                        page_url = format!("{}{}", GITHUB_BASE_URL, link);
                    } else {
                        page_url = format!("{}/{}", GITHUB_BASE_URL, link);
                    }
                    
                    // Add a small delay to be nice to GitHub
                    sleep(Duration::from_secs(1)).await;
                } else {
                    break;
                }
            },
            Err(e) => {
                println!("Error fetching page {}: {}", page_count, e);
                break;
            }
        }
    }
    
    pb.finish_with_message("Download complete");
    
    println!("\nSorting {} repositories by star count...", all_dependents.len());
    
    // Store the length before moving all_dependents
    let total_repos_count = all_dependents.len();
    
    // Remove duplicates while keeping highest star count
    let mut unique_deps: HashMap<String, String> = HashMap::new();
    for (repo, stars) in &all_dependents {
        let current_stars = convert_stars_to_number(stars);
        if !unique_deps.contains_key(repo) || 
           current_stars > convert_stars_to_number(unique_deps.get(repo).unwrap()) {
            unique_deps.insert(repo.clone(), stars.clone());
        }
    }
    
    // Filter by minimum stars and convert to Vec
    let mut filtered_deps: Vec<(String, String)> = unique_deps.into_iter()
        .filter(|(_, stars)| convert_stars_to_number(stars) >= config.min_stars)
        .collect();
    
    // Store the length before moving filtered_deps
    let more_than_zero_count = filtered_deps.len();
    
    // Sort by star count
    filtered_deps.sort_by(|(_, a_stars), (_, b_stars)| {
        convert_stars_to_number(b_stars).partial_cmp(&convert_stars_to_number(a_stars)).unwrap()
    });
    
    // Take top N
    let top_deps = filtered_deps.into_iter().take(config.top_n).collect::<Vec<_>>();
    
    // Add descriptions if requested
    let mut result = Vec::new();
    
    if config.show_desc && !top_deps.is_empty() {
        println!("Fetching repository descriptions...");
        
        let descriptions = get_repo_descriptions(&client, top_deps, config.use_cache).await;
        for (repo, stars, description) in descriptions {
            result.push(Dependent {
                repo,
                stars,
                description,
            });
        }
    } else {
        for (repo, stars) in top_deps {
            result.push(Dependent {
                repo,
                stars,
                description: None,
            });
        }
    }
    
    Ok((result, total_repos_count, more_than_zero_count, max_deps))
}

async fn get_repo_descriptions(
    client: &Client, 
    repos: Vec<(String, String)>, 
    use_cache: bool
) -> Vec<(String, String, Option<String>)> {
    println!("Fetching repository descriptions...");
    
    // Create a rate limiter with max 5 concurrent requests
    let semaphore = Arc::new(Semaphore::new(5));
    
    let client = Arc::new(client.clone());
    
    // Process in parallel with rate limiting
    let results = stream::iter(repos)
        .map(|(repo, stars)| {
            let client = client.clone();
            let semaphore = semaphore.clone();
            
            async move {
                let _permit = semaphore.acquire().await.unwrap();
                let description = get_repo_description(&client, &repo, use_cache).await;
                (repo, stars, description)
            }
        })
        .buffer_unordered(5) // Process up to 5 at a time
        .collect::<Vec<_>>()
        .await;
    
    results
} 