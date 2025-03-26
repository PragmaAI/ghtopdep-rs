use clap::ArgMatches;
use crate::error::AppError;

pub const DEFAULT_FORMAT: &str = "table";
pub const DEFAULT_MIN_STARS: f64 = 0.0;

pub struct Config {
    pub owner: String,
    pub repo: String,
    pub top_n: usize,
    pub max_pages: usize,
    pub min_stars: f64,
    pub is_package: bool,
    pub show_desc: bool,
    pub use_cache: bool,
    pub output_format: String,
}

impl Config {
    pub fn from_matches(matches: &ArgMatches) -> Result<Self, AppError> {
        let repo_url = matches.get_one::<String>("repo_url").unwrap();
        let (owner, repo) = parse_repo_url(repo_url)?;

        let top_n = matches.get_one::<String>("top_n").unwrap().parse::<usize>().unwrap_or(10);
        let max_pages = matches.get_one::<String>("max_pages").unwrap().parse::<usize>().unwrap_or(100);
        let min_stars = matches.get_one::<String>("min_stars").unwrap().parse::<f64>().unwrap_or(DEFAULT_MIN_STARS);
        let is_package = matches.get_flag("packages");
        let show_desc = matches.get_flag("description");
        let use_cache = !matches.get_flag("no-cache");

        let output_format = if matches.get_flag("table") {
            "table".to_string()
        } else {
            matches.get_one::<String>("format").unwrap().clone()
        };

        Ok(Config {
            owner,
            repo,
            top_n,
            max_pages,
            min_stars,
            is_package,
            show_desc,
            use_cache,
            output_format,
        })
    }

    pub fn dependent_type(&self) -> &'static str {
        if self.is_package { "PACKAGE" } else { "REPOSITORY" }
    }
}

fn parse_repo_url(repo_url: &str) -> Result<(String, String), AppError> {
    if repo_url.contains("github.com") {
        let parts: Vec<&str> = repo_url.trim_end_matches('/').split('/').collect();
        if parts.len() >= 5 {
            Ok((parts[parts.len() - 2].to_string(), parts[parts.len() - 1].to_string()))
        } else {
            Err(AppError::Other("Invalid GitHub URL format. Expected: https://github.com/owner/repo".to_string()))
        }
    } else if repo_url.contains('/') {
        let parts: Vec<&str> = repo_url.split('/').collect();
        if parts.len() == 2 {
            Ok((parts[0].to_string(), parts[1].to_string()))
        } else {
            Err(AppError::Other("Invalid format. Expected: owner/repo or https://github.com/owner/repo".to_string()))
        }
    } else {
        Err(AppError::Other("Invalid format. Expected: owner/repo or https://github.com/owner/repo".to_string()))
    }
} 