use crate::scope3_types::CalcPath;

pub fn range_guard_check(tco2e: f64, cat_id: u8, calc_path: &CalcPath) -> Result<(), String> {
    if tco2e < 0.0 {
        return Err("Negative value not allowed".to_string());
    }
    if tco2e > 10_000_000.0 {
        return Err("Exceeds absolute max (10M tCO2e)".to_string());
    }

    match cat_id {
        1 => {
            if matches!(calc_path, CalcPath::ActivityBased) && tco2e > 50_000.0 {
                return Err("Cat 1 Activity exceeds 50,000 tCO2e".to_string());
            }
            if matches!(calc_path, CalcPath::SpendBased) && tco2e > 500_000.0 {
                return Err("Cat 1 Spend exceeds 500,000 tCO2e".to_string());
            }
        },
        4 => {
            if matches!(calc_path, CalcPath::ActivityBased) && tco2e > 200_000.0 {
                return Err("Cat 4 Activity exceeds 200,000 tCO2e".to_string());
            }
            if matches!(calc_path, CalcPath::SpendBased) && tco2e > 500_000.0 {
                return Err("Cat 4 Spend exceeds 500,000 tCO2e".to_string());
            }
        },
        _ => {}
    }

    Ok(())
}
