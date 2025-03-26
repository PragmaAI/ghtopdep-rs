use clap::{App, Arg};
use futures::stream::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{Client, header};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tokio::time::sleep;
use dirs::cache_dir;
use crypto::digest::Digest;
use crypto::md5::Md5;

const GITHUB_BASE_URL: &str = "https://github.com";
const REPOS_PER_PAGE: usize = 30;
const CACHE_EXPIRY_HOURS: u64 = 24;

#[derive(Debug, Serialize, Deserialize)]
struct CachedResponse {
    timestamp: u64,
    content: String,
}

#[derive(Debug, Clone)]
struct Dependent {
    repo: String,
    stars: String,
    description: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("gh_get_dependent")
        .version("1.0")
        .about("Get top dependent repositories for a GitHub project")
        .arg(Arg::with_name("owner")
            .help("GitHub repository owner (e.g., 'facebook' for facebook/react)")
            .required(true)
            .index(1))
        .arg(Arg::with_name("repo")
            .help("GitHub repository name (e.g., 'react' for facebook/react)")
            .required(true)
            .index(2))
        .arg(Arg::with_name("top_n")
            .long("top_n")
            .help("Number of top dependents to fetch")
            .default_value("10"))
        .arg(Arg::with_name("max_pages")
            .long("max_pages")
            .help("Maximum number of pages to fetch")
            .default_value("10"))
        .arg(Arg::with_name("min_stars")
            .long("min_stars")
            .help("Minimum number of stars")
            .default_value("0"))
        .arg(Arg::with_name("packages")
            .long("packages")
            .help("Show package dependents instead of repositories")
            .takes_value(false))
        .arg(Arg::with_name("description")
            .long("description")
            .help("Show repository descriptions")
            .takes_value(false))
        .arg(Arg::with_name("no-cache")
            .long("no-cache")
            .help("Disable caching")
            .takes_value(false))
        .get_matches();

    let owner = matches.value_of("owner").unwrap();
    let repo = matches.value_of("repo").unwrap();
    let top_n = matches.value_of("top_n").unwrap().parse::<usize>().unwrap_or(10);
    let max_pages = matches.value_of("max_pages").unwrap().parse::<usize>().unwrap_or(10);
    let min_stars = matches.value_of("min_stars").unwrap().parse::<f64>().unwrap_or(0.0);
    let dependent_type = if matches.is_present("packages") { "PACKAGE" } else { "REPOSITORY" };
    let show_desc = matches.is_present("description");
    let use_cache = !matches.is_present("no-cache");

    println!("Fetching {} dependents for {}/{}...", dependent_type.to_lowercase(), owner, repo);

    let top_dependents = get_top_dependents(
        owner, 
        repo, 
        top_n, 
        max_pages, 
        min_stars, 
        dependent_type, 
        show_desc, 
        use_cache
    ).await?;

    if !top_dependents.is_empty() {
        println!("\nTop {} {} dependents (min {} stars):", 
            top_dependents.len(), 
            dependent_type.to_lowercase(), 
            min_stars);
        
        for (idx, dep) in top_dependents.iter().enumerate() {
            if show_desc {
                let desc_text = match &dep.description {
                    Some(desc) if !desc.is_empty() => format!("\n   {}", desc),
                    _ => String::new(),
                };
                println!("{}. {} (⭐ {}){}", idx + 1, dep.repo, dep.stars, desc_text);
            } else {
                println!("{}. {} (⭐ {})", idx + 1, dep.repo, dep.stars);
            }
        }
    } else {
        println!("No {} dependents found or access denied.", dependent_type.to_lowercase());
    }

    Ok(())
}

fn get_cache_dir() -> PathBuf {
    let mut cache_path = cache_dir().unwrap_or_else(|| PathBuf::from("./cache"));
    cache_path.push("gh_get_dependent");
    fs::create_dir_all(&cache_path).unwrap_or_else(|_| {
        println!("Warning: Could not create cache directory");
    });
    cache_path
}

fn get_cache_path(url: &str) -> PathBuf {
    let mut hasher = Md5::new();
    hasher.input_str(url);
    let url_hash = hasher.result_str();
    
    let mut path = get_cache_dir();
    path.push(format!("{}.json", url_hash));
    path
}

fn is_cache_valid(cache_path: &PathBuf) -> bool {
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

async fn cached_fetch(client: &Client, url: &str, use_cache: bool) -> Result<String, Box<dyn std::error::Error>> {
    let cache_path = get_cache_path(url);
    
    if use_cache && is_cache_valid(&cache_path) {
        match fs::read_to_string(&cache_path) {
            Ok(cached_data) => {
                match serde_json::from_str::<CachedResponse>(&cached_data) {
                    Ok(cached) => return Ok(cached.content),
                    Err(e) => {
                        println!("Warning: Cache file corrupted, fetching fresh data: {}", e);
                        // Continue to fetch fresh data
                    }
                }
            },
            Err(e) => {
                println!("Warning: Could not read cache file: {}", e);
                // Continue to fetch fresh data
            }
        }
    }
    
    // Fetch fresh data
    let response = client.get(url).send().await?;
    if response.status().is_success() {
        let html = response.text().await?;
        
        if use_cache {
            let cached = CachedResponse {
                timestamp: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                content: html.clone(),
            };
            
            match serde_json::to_string(&cached) {
                Ok(json) => {
                    fs::write(&cache_path, json).unwrap_or_else(|e| {
                        println!("Warning: Could not write to cache file: {}", e);
                    });
                },
                Err(e) => {
                    println!("Warning: Could not serialize cache data: {}", e);
                }
            }
        }
        
        Ok(html)
    } else {
        Err(format!("Failed to fetch {}: {}", url, response.status()).into())
    }
}

fn convert_stars_to_number(stars_text: &str) -> f64 {
    if stars_text == "N/A" {
        return -1.0;
    }
    
    let stars_text = stars_text.trim();
    if stars_text.is_empty() {
        return 0.0;
    }
    
    if stars_text.to_lowercase().contains('k') {
        let num_str = stars_text.to_lowercase().replace('k', "");
        match num_str.parse::<f64>() {
            Ok(num) => return num * 1000.0,
            Err(_) => return 0.0,
        }
    }
    
    stars_text.replace(',', "").parse::<f64>().unwrap_or(0.0)
}

async fn get_max_deps(client: &Client, url: &str, dependent_type: &str, use_cache: bool) -> usize {
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

async fn get_repo_description(client: &Client, repo_path: &str, use_cache: bool) -> Option<String> {
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

fn parse_page(html: &str) -> (Vec<(String, String)>, Option<String>) {
    let document = Html::parse_document(html);
    let mut dependents = Vec::new();
    
    // Selectors
    let rows_selector = Selector::parse("div.Box > div.flex-items-center").unwrap();
    let repo_selector = Selector::parse("span > a.text-bold").unwrap();
    let stars_selector = Selector::parse("div > span:nth-child(1)").unwrap();
    let pagination_selector = Selector::parse("div.paginate-container > div > a").unwrap();
    
    // Extract dependents
    for row in document.select(&rows_selector) {
        if let Some(repo_link) = row.select(&repo_selector).next() {
            if let Some(href) = repo_link.value().attr("href") {
                let dep_repo = href.trim_start_matches('/').to_string();
                
                let mut stars = "0".to_string();
                if let Some(stars_element) = row.select(&stars_selector).next() {
                    let stars_text = stars_element.text().collect::<Vec<_>>().join("");
                    if !stars_text.trim().is_empty() {
                        stars = stars_text.trim().to_string();
                    }
                }
                
                dependents.push((dep_repo, stars));
            }
        }
    }
    
    // Find next page link
    let pagination_buttons: Vec<_> = document.select(&pagination_selector).collect();
    let next_link = if pagination_buttons.len() == 2 {
        pagination_buttons[1].value().attr("href").map(String::from)
    } else if !pagination_buttons.is_empty() {
        let text = pagination_buttons[0].text().collect::<Vec<_>>().join("");
        if text.trim() == "Next" {
            pagination_buttons[0].value().attr("href").map(String::from)
        } else {
            None
        }
    } else {
        None
    };
    
    (dependents, next_link)
}

async fn get_top_dependents(
    owner: &str,
    repo: &str,
    top_n: usize,
    max_pages: usize,
    min_stars: f64,
    dependent_type: &str,
    show_desc: bool,
    use_cache: bool,
) -> Result<Vec<Dependent>, Box<dyn std::error::Error>> {
    let base_url = format!("{}/{}/{}/network/dependents", GITHUB_BASE_URL, owner, repo);
    let mut page_url = format!("{}?dependent_type={}", base_url, dependent_type);
    
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"),
    );
    
    let client = Client::builder()
        .default_headers(headers)
        .build()?;
    
    let mut all_dependents = Vec::new();
    let mut page_count = 0;
    
    // Get the maximum number of dependents
    let max_deps = get_max_deps(&client, &base_url, dependent_type, use_cache).await;
    let pb = if max_deps > 0 {
        println!("Found {} total dependents", max_deps);
        let total = std::cmp::min(max_deps, max_pages * REPOS_PER_PAGE);
        let pb = ProgressBar::new(total as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-"));
        pb
    } else {
        ProgressBar::new_spinner()
    };
    
    while page_count < max_pages {
        page_count += 1;
        
        match cached_fetch(&client, &page_url, use_cache).await {
            Ok(html) => {
                let (deps, next_link) = parse_page(&html);
                if deps.is_empty() {
                    break;
                }
                
                all_dependents.extend(deps);
                pb.set_position(all_dependents.len() as u64);
                
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
    
    // Remove duplicates while keeping highest star count
    let mut unique_deps: HashMap<String, String> = HashMap::new();
    for (repo, stars) in all_dependents {
        let current_stars = convert_stars_to_number(&stars);
        if !unique_deps.contains_key(&repo) || 
           current_stars > convert_stars_to_number(unique_deps.get(&repo).unwrap()) {
            unique_deps.insert(repo, stars);
        }
    }
    
    // Filter by minimum stars and convert to Vec
    let mut filtered_deps: Vec<(String, String)> = unique_deps.into_iter()
        .filter(|(_, stars)| convert_stars_to_number(stars) >= min_stars)
        .collect();
    
    // Sort by star count
    filtered_deps.sort_by(|(_, a_stars), (_, b_stars)| {
        convert_stars_to_number(b_stars).partial_cmp(&convert_stars_to_number(a_stars)).unwrap()
    });
    
    // Take top N
    let top_deps = filtered_deps.into_iter().take(top_n).collect::<Vec<_>>();
    
    // Add descriptions if requested
    let mut result = Vec::new();
    
    if show_desc && !top_deps.is_empty() {
        println!("Fetching repository descriptions...");
        
        for (repo, stars) in top_deps {
            let description = if show_desc {
                get_repo_description(&client, &repo, use_cache).await
            } else {
                None
            };
            
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
    
    Ok(result)
}