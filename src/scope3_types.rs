use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchMethod { Exact, Fuzzy, Inferred }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalcPath { ActivityBased, SpendBased }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataQualityTier { Primary, Secondary, Estimated }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope3Extension {
    pub category_id: u8,
    pub category_name: String,
    pub category_match_method: MatchMethod,
    pub category_confidence: f32,
    pub calc_path: CalcPath,
    // Spend-based mezők
    pub spend_usd_normalized: Option<f64>,
    pub eeio_sector_code: Option<String>,
    pub eeio_source: Option<String>,
    // Activity-based mezők
    pub physical_quantity: Option<f64>,
    pub physical_unit: Option<String>,
    // Data Quality
    pub data_quality_tier: DataQualityTier,
    pub ghg_protocol_dq_score: u8,
}
