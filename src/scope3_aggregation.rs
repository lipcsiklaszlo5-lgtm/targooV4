use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySummary {
    pub category_id: u8,
    pub total_tco2e: f64,
    pub row_count: u32,
    pub calc_path_mix: (u32, u32), // (ActivityBased count, SpendBased count)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope3Aggregation {
    pub grand_total_tco2e: f64,
    pub category_breakdown: HashMap<u8, CategorySummary>,
    pub data_quality_avg_score: f32,
}

pub fn aggregate_scope3() -> Scope3Aggregation {
    Scope3Aggregation {
        grand_total_tco2e: 0.0,
        category_breakdown: HashMap::new(),
        data_quality_avg_score: 0.0,
    }
}
