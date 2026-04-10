use crate::models::{GhgCategory, ErrorReason};

pub fn calculate_tco2e(converted_value: f64, ef_kgco2e: f64, gwp: f64) -> f64 {
    (converted_value * ef_kgco2e * gwp) / 1000.0
}

pub fn convert_to_canonical(value: f64, unit: &str) -> f64 {
    let multiplier = match unit.to_lowercase().as_str() {
        "therm" => 29.3,
        "mmbtu" => 293.071,
        "mwh" => 1000.0,
        "short_ton" => 907.185,
        "lb" => 0.453592,
        "long_ton" => 1016.05,
        "tonne" | "metric_tonne" => 1000.0,
        "gallon_us" => 3.78541,
        "gallon_uk" => 4.54609,
        "m3" => 1000.0,
        "mile" => 1.60934,
        "ton_mile" => 1.46011,
        _ => 1.0,
    };
    value * multiplier
}

pub fn validate_range(tco2e: f64, category: &GhgCategory, scope3_id: Option<u8>) -> Result<(), ErrorReason> {
    if tco2e < 0.0 {
        return Err(ErrorReason::NegativeValue);
    }
    if tco2e > 10_000_000.0 {
        return Err(ErrorReason::RangeGuardFail);
    }
    match category {
        GhgCategory::Scope1 => {
            if tco2e > 100_000.0 {
                return Err(ErrorReason::RangeGuardFail);
            }
        }
        GhgCategory::Scope2 => {
            if tco2e > 50_000.0 {
                return Err(ErrorReason::RangeGuardFail);
            }
        }
        GhgCategory::Scope3 => {
            if let Some(4) = scope3_id {
                if tco2e > 200_000.0 {
                    return Err(ErrorReason::RangeGuardFail);
                }
            }
        }
    }
    Ok(())
}
