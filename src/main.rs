mod cache;
mod client;
mod config;
mod dependent;
mod error;
mod github;
mod output;
#[cfg(test)]
mod tests;
use clap::{Arg, ArgAction, Command};
use std::time::Instant;

use config::Config;
use github::get_top_dependents;
use output::display_results;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("ghtodep-rs")
        .version("1.0")
        .about("Get top dependent repositories for a GitHub project")
        .arg(
            Arg::new("repo_url")
                .help("GitHub repository URL or owner/repo format")
                .required(true)
                .index(1)
        )
        .arg(
            Arg::new("top_n")
                .long("rows")
                .help("Number of top dependents to fetch")
                .default_value("10")
        )
        .arg(
            Arg::new("max_pages")
                .long("max_pages")
                .help("Maximum number of pages to fetch")
                .default_value("100")
        )
        .arg(
            Arg::new("min_stars")
                .long("minstar")
                .help("Minimum number of stars")
                .default_value("0")
        )
        .arg(
            Arg::new("packages")
                .long("packages")
                .help("Show package dependents instead of repositories")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("description")
                .long("description")
                .help("Show repository descriptions")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("no-cache")
                .long("no-cache")
                .help("Disable caching")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("format")
                .long("format")
                .help("Output format (text, json, table)")
                .default_value(config::DEFAULT_FORMAT)
        )
        .arg(
            Arg::new("table")
                .long("table")
                .help("Use table output format (shorthand for --format table)")
                .action(ArgAction::SetTrue)
        )
        .get_matches();

    let config = Config::from_matches(&matches)?;
    println!("Fetching {} dependents for {}/{}...", 
        if config.is_package { "package" } else { "repository" }, 
        config.owner, 
        config.repo);

    let start_time = Instant::now();

    let (top_dependents, total_repos_count, more_than_zero_count, max_deps) = 
        get_top_dependents(&config).await?;

    let elapsed = start_time.elapsed();
    
    display_results(
        &config,
        &top_dependents,
        total_repos_count,
        more_than_zero_count,
        max_deps,
        elapsed.as_secs_f64(),
    )?;

    Ok(())
}