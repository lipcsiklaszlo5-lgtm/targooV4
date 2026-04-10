use strsim::levenshtein;

#[derive(Debug, Clone)]
pub enum Scope3Classification {
    Matched { category_id: u8, match_method: String, confidence: f32 },
    Inferred { category_id: u8, reason: String, confidence: f32 },
    Unknown { error: String },
}

const SCOPE3_KEYWORD_MAP: &[(&[&str], u8)] = &[
    (&["purchase", "supplier", "procurement", "material", "beschaffung"], 1),
    (&["capex", "capital", "equipment", "machinery", "investition"], 2),
    (&["freight", "transport", "shipping", "logistics", "spedition"], 4),
    (&["flight", "travel", "hotel", "business_trip", "dienstreise"], 6),
    (&["investment", "portfolio", "financial", "financed_emissions"], 15),
];

pub fn classify_scope3(raw_header: &str, _raw_value: &str, file_name: &str) -> Scope3Classification {
    let header_norm = raw_header.to_lowercase();
    let file_norm = file_name.to_lowercase();
    let value_norm = _raw_value.to_lowercase();

    // LEVEL 1: Exact Match
    for (keywords, id) in SCOPE3_KEYWORD_MAP {
        for &kw in *keywords {
            if header_norm.contains(kw) || file_norm.contains(kw) {
                return Scope3Classification::Matched {
                    category_id: *id,
                    match_method: "Exact".to_string(),
                    confidence: 1.0,
                };
            }
        }
    }

    // LEVEL 2: Fuzzy Match (Levenshtein <= 2)
    for (keywords, id) in SCOPE3_KEYWORD_MAP {
        for &kw in *keywords {
            for word in header_norm.split_whitespace() {
                if levenshtein(word, kw) <= 2 {
                    return Scope3Classification::Matched {
                        category_id: *id,
                        match_method: "Fuzzy".to_string(),
                        confidence: 0.85,
                    };
                }
            }
        }
    }

    // LEVEL 3: Heuristic (Currency detection)
    let currencies = ["$", "€", "£", "usd", "eur", "gbp"];
    if currencies.iter().any(|&c| value_norm.contains(c)) {
        return Scope3Classification::Inferred {
            category_id: 1,
            reason: "Currency detected, assuming Purchased Goods".to_string(),
            confidence: 0.5,
        };
    }

    // FALLBACK
    Scope3Classification::Unknown {
        error: "No matching Scope 3 category found".to_string(),
    }
}
