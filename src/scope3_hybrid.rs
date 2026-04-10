use crate::models::Jurisdiction;
use crate::triage::parse_numeric;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Currency { USD, GBP, EUR, Unknown }

#[derive(Debug, Clone)]
pub struct EeioSector {
    pub code: String,
    pub name: String,
    pub source: String,
    pub kgco2e_per_1000usd: f64,
}

#[derive(Debug, Clone)]
pub struct SpendResult {
    pub tco2e: f64,
    pub currency_detected: Currency,
    pub usd_normalized: f64,
    pub eeio_sector_code: String,
    pub eeio_source: String,
}

pub fn detect_currency_priority(raw_value: &str, raw_header: &str, jur: &Jurisdiction) -> Currency {
    let val_low = raw_value.to_lowercase();
    let head_low = raw_header.to_lowercase();

    let check = |s: &str| {
        if s.contains('$') || s.contains("usd") { Some(Currency::USD) }
        else if s.contains('€') || s.contains("eur") { Some(Currency::EUR) }
        else if s.contains('£') || s.contains("gbp") { Some(Currency::GBP) }
        else { None }
    };

    if let Some(c) = check(&val_low) { return c; }
    if let Some(c) = check(&head_low) { return c; }

    match jur {
        Jurisdiction::US => Currency::USD,
        Jurisdiction::UK => Currency::GBP,
        Jurisdiction::EU => Currency::EUR,
        _ => Currency::Unknown,
    }
}

pub fn select_eeio_sector(cat_id: u8, jur: &Jurisdiction) -> EeioSector {
    match (cat_id, jur) {
        (1, Jurisdiction::US) => EeioSector {
            code: "325".to_string(),
            name: "Chemical Manufacturing".to_string(),
            source: "USEEIO_v2.1".to_string(),
            kgco2e_per_1000usd: 370.0,
        },
        (4, Jurisdiction::UK) => EeioSector {
            code: "SIC_7400".to_string(),
            name: "Freight Transport".to_string(),
            source: "DEFRA_spend_2024".to_string(),
            kgco2e_per_1000usd: 410.0,
        },
        (6, Jurisdiction::EU) => EeioSector {
            code: "H51".to_string(),
            name: "Air Transport".to_string(),
            source: "EXIOBASE_3.8".to_string(),
            kgco2e_per_1000usd: 340.0,
        },
        _ => EeioSector {
            code: "GLOBAL_AVERAGE".to_string(),
            name: "General Sector Average".to_string(),
            source: "MOCK_GLOBAL".to_string(),
            kgco2e_per_1000usd: 350.0,
        },
    }
}

pub fn calculate_spend_based(
    raw_value: &str,
    raw_header: &str,
    cat_id: u8,
    jur: &Jurisdiction,
) -> Result<SpendResult, String> {
    let currency = detect_currency_priority(raw_value, raw_header, jur);
    let value = parse_numeric(raw_value).map_err(|e| format!("Numeric parse error: {:?}", e))?;

    let (usd_normalized, currency_final) = match currency {
        Currency::EUR => (value * 1.08, Currency::EUR),
        Currency::GBP => (value * 1.26, Currency::GBP),
        Currency::USD => (value, Currency::USD),
        Currency::Unknown => (value, Currency::Unknown), // Assume USD if unknown for calculation
    };

    let sector = select_eeio_sector(cat_id, jur);
    
    // Formula: (usd / 1000.0) * kgco2e_per_1000usd / 1000.0 -> tCO2e
    let tco2e = (usd_normalized / 1000.0) * sector.kgco2e_per_1000usd / 1000.0;

    Ok(SpendResult {
        tco2e,
        currency_detected: currency_final,
        usd_normalized,
        eeio_sector_code: sector.code,
        eeio_source: sector.source,
    })
}
