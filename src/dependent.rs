use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependent {
    pub repo: String,
    pub stars: String,
    pub description: Option<String>,
}

pub fn convert_stars_to_number(stars_text: &str) -> f64 {
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