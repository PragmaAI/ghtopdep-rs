use clap::{Arg, ArgAction, Command};
use crate::config::Config;

#[test]
fn test_parse_repo_url_github_format() {
    let matches = Command::new("test")
        .arg(Arg::new("repo_url").required(true).index(1).default_value("https://github.com/owner/repo"))
        .arg(Arg::new("top_n").long("rows").default_value("10"))
        .arg(Arg::new("max_pages").long("max_pages").default_value("100"))
        .arg(Arg::new("min_stars").long("minstar").default_value("0"))
        .arg(Arg::new("packages").long("packages").action(ArgAction::SetTrue))
        .arg(Arg::new("description").long("description").action(ArgAction::SetTrue))
        .arg(Arg::new("no-cache").long("no-cache").action(ArgAction::SetTrue))
        .arg(Arg::new("format").long("format").default_value("table"))
        .arg(Arg::new("table").long("table").action(ArgAction::SetTrue))
        .get_matches_from(vec!["test", "https://github.com/owner/repo"]);
    
    let config = Config::from_matches(&matches).unwrap();
    assert_eq!(config.owner, "owner");
    assert_eq!(config.repo, "repo");
}

#[test]
fn test_parse_repo_url_short_format() {
    let matches = Command::new("test")
        .arg(Arg::new("repo_url").required(true).index(1).default_value("owner/repo"))
        .arg(Arg::new("top_n").long("rows").default_value("10"))
        .arg(Arg::new("max_pages").long("max_pages").default_value("100"))
        .arg(Arg::new("min_stars").long("minstar").default_value("0"))
        .arg(Arg::new("packages").long("packages").action(ArgAction::SetTrue))
        .arg(Arg::new("description").long("description").action(ArgAction::SetTrue))
        .arg(Arg::new("no-cache").long("no-cache").action(ArgAction::SetTrue))
        .arg(Arg::new("format").long("format").default_value("table"))
        .arg(Arg::new("table").long("table").action(ArgAction::SetTrue))
        .get_matches_from(vec!["test", "owner/repo"]);
    
    let config = Config::from_matches(&matches).unwrap();
    assert_eq!(config.owner, "owner");
    assert_eq!(config.repo, "repo");
}

#[test]
fn test_dependent_type() {
    let matches = Command::new("test")
        .arg(Arg::new("repo_url").required(true).index(1).default_value("owner/repo"))
        .arg(Arg::new("top_n").long("rows").default_value("10"))
        .arg(Arg::new("max_pages").long("max_pages").default_value("100"))
        .arg(Arg::new("min_stars").long("minstar").default_value("0"))
        .arg(Arg::new("packages").long("packages").action(ArgAction::SetTrue))
        .arg(Arg::new("description").long("description").action(ArgAction::SetTrue))
        .arg(Arg::new("no-cache").long("no-cache").action(ArgAction::SetTrue))
        .arg(Arg::new("format").long("format").default_value("table"))
        .arg(Arg::new("table").long("table").action(ArgAction::SetTrue))
        .get_matches_from(vec!["test", "owner/repo", "--packages"]);
    
    let config = Config::from_matches(&matches).unwrap();
    assert_eq!(config.dependent_type(), "PACKAGE");
    
    let matches = Command::new("test")
        .arg(Arg::new("repo_url").required(true).index(1).default_value("owner/repo"))
        .arg(Arg::new("top_n").long("rows").default_value("10"))
        .arg(Arg::new("max_pages").long("max_pages").default_value("100"))
        .arg(Arg::new("min_stars").long("minstar").default_value("0"))
        .arg(Arg::new("packages").long("packages").action(ArgAction::SetTrue))
        .arg(Arg::new("description").long("description").action(ArgAction::SetTrue))
        .arg(Arg::new("no-cache").long("no-cache").action(ArgAction::SetTrue))
        .arg(Arg::new("format").long("format").default_value("table"))
        .arg(Arg::new("table").long("table").action(ArgAction::SetTrue))
        .get_matches_from(vec!["test", "owner/repo"]);
    
    let config = Config::from_matches(&matches).unwrap();
    assert_eq!(config.dependent_type(), "REPOSITORY");
}

#[test]
fn test_invalid_repo_url() {
    let matches = Command::new("test")
        .arg(Arg::new("repo_url").required(true).index(1).default_value("invalid-format"))
        .arg(Arg::new("top_n").long("rows").default_value("10"))
        .arg(Arg::new("max_pages").long("max_pages").default_value("100"))
        .arg(Arg::new("min_stars").long("minstar").default_value("0"))
        .arg(Arg::new("packages").long("packages").action(ArgAction::SetTrue))
        .arg(Arg::new("description").long("description").action(ArgAction::SetTrue))
        .arg(Arg::new("no-cache").long("no-cache").action(ArgAction::SetTrue))
        .arg(Arg::new("format").long("format").default_value("table"))
        .arg(Arg::new("table").long("table").action(ArgAction::SetTrue))
        .get_matches_from(vec!["test", "invalid-format"]);
    
    let result = Config::from_matches(&matches);
    assert!(result.is_err());
} 