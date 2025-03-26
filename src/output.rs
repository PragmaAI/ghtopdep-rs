use prettytable::{Table, row};
use serde_json;

use crate::config::Config;
use crate::dependent::Dependent;
use crate::error::AppError;

pub fn display_results(
    config: &Config,
    top_dependents: &[Dependent],
    total_repos_count: usize,
    more_than_zero_count: usize,
    max_deps: usize,
    elapsed_secs: f64,
) -> Result<(), AppError> {
    match config.output_format.as_str() {
        "json" => {
            let result = serde_json::json!({
                "dependents": top_dependents,
                "stats": {
                    "total_repositories": total_repos_count,
                    "repositories_with_stars": more_than_zero_count,
                    "elapsed_seconds": elapsed_secs
                }
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        },
        "table" => {
            let mut table = Table::new();
            table.add_row(row!["url", "stars"]);
            
            for dep in top_dependents.iter() {
                let url = format!("https://github.com/{}", dep.repo);
                table.add_row(row![url, dep.stars]);
            }
            
            table.printstd();
            
            println!("found {} repositories{}", 
                total_repos_count,
                if total_repos_count < max_deps { " others repositories are private" } else { "" }
            );
            println!("found {} repositories with more than zero star", more_than_zero_count);
            println!("Completed in {:.2} seconds", elapsed_secs);
        },
        _ => {
            if !top_dependents.is_empty() {
                println!("\nTop {} {} dependents (min {} stars):", 
                    top_dependents.len(), 
                    config.dependent_type().to_lowercase(), 
                    config.min_stars);
                
                for (idx, dep) in top_dependents.iter().enumerate() {
                    if config.show_desc {
                        let desc_text = match &dep.description {
                            Some(desc) if !desc.is_empty() => format!("\n   {}", desc),
                            _ => String::new(),
                        };
                        println!("{}. {} (⭐ {}){}", idx + 1, dep.repo, dep.stars, desc_text);
                    } else {
                        println!("{}. {} (⭐ {})", idx + 1, dep.repo, dep.stars);
                    }
                }
                
                println!("\nFound {} total repositories", total_repos_count);
                println!("Found {} repositories with stars", more_than_zero_count);
                println!("Completed in {:.2} seconds", elapsed_secs);
            } else {
                println!("No {} dependents found or access denied.", 
                    config.dependent_type().to_lowercase());
            }
        }
    }

    Ok(())
} 