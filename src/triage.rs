use crate::models::{Mapping, GhgCategory, ErrorReason, DictionaryEntry};
use std::time::Duration;

pub fn lookup_dictionary(header: &str, dictionary: &[DictionaryEntry]) -> Option<Mapping> {
    let header = header.to_lowercase();
    let mut best_match: Option<(usize, Mapping)> = None;

    for entry in dictionary {
        for kw in &entry.keywords {
            let kw_low = kw.to_lowercase();
            if header.contains(&kw_low) {
                let kw_len = kw_low.len();
                if best_match.as_ref().map_or(true, |(len, _)| kw_len > *len) {
                    best_match = Some((kw_len, Mapping {
                        canonical_unit: entry.canonical_unit.clone(),
                        ghg_category: entry.ghg_category.clone(),
                        scope3_id: entry.scope3_id,
                    }));
                }
            }
        }
    }

    best_match.map(|(_, m)| m)
}

pub fn parse_numeric(val: &str) -> Result<f64, ErrorReason> {
    let mut cleaned = val.trim().to_string();
    if cleaned.is_empty() || cleaned.to_lowercase() == "n/a" {
        return Err(ErrorReason::NonNumericValue);
    }

    let has_k = cleaned.to_lowercase().ends_with('k');
    if has_k {
        cleaned.pop();
    }

    cleaned = cleaned.chars()
        .filter(|&c| c.is_digit(10) || c == '.' || c == ',' || c == '-')
        .collect();

    if cleaned.is_empty() {
        return Err(ErrorReason::NonNumericValue);
    }

    let first_comma = cleaned.find(',');
    let first_dot = cleaned.find('.');

    match (first_comma, first_dot) {
        (Some(c), Some(d)) => {
            if c > d {
                cleaned = cleaned.replace('.', "").replace(',', ".");
            } else {
                cleaned = cleaned.replace(',', "");
            }
        }
        (Some(_), None) => {
            cleaned = cleaned.replace(',', ".");
        }
        _ => {}
    }

    let mut num: f64 = cleaned.parse().map_err(|_| ErrorReason::NonNumericValue)?;
    if has_k {
        num *= 1000.0;
    }
    Ok(num)
}

pub fn ai_classify(header: &str, value: &str) -> Option<Mapping> {
    let client = reqwest::blocking::Client::new();
    let res = client.post("http://127.0.0.1:9000/classify")
        .timeout(Duration::from_millis(300))
        .json(&serde_json::json!({
            "header": header,
            "value": value
        }))
        .send();

    match res {
        Ok(resp) => {
            if resp.status().is_success() {
                resp.json::<Mapping>().ok()
            } else {
                None
            }
        }
        Err(_) => None,
    }
}
