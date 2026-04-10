use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Jurisdiction { US, UK, EU, CH, Global }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GhgCategory { Scope1, Scope2, Scope3 }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Industry {
   General, Manufacturing, Logistics, Commerce,
   Service, Finance, OilGas, Chemicals, Electronics, Agriculture
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ErrorReason {
   UnknownHeader, NonNumericValue, RangeGuardFail, NegativeValue
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Language { English, German, Hungarian }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Mapping {
   pub canonical_unit: String,
   pub ghg_category: GhgCategory,
   pub scope3_id: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmissionFactor {
   pub factor_value: f64,
   pub gas_type: String,
   pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRequest {
   pub jurisdiction: Jurisdiction,
   pub language: Language,
   pub industry: Industry,
   pub gemini_api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionaryEntry {
    pub keywords: Vec<String>,
    pub ghg_category: GhgCategory,
    pub scope3_id: Option<u8>,
    pub canonical_unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TriageResult {
   Green  { value: f64, unit: String, mapping: Mapping },
   Yellow { value: f64, assumed_unit: String, original_unit: String, mapping: Mapping },
   Red    { error: ErrorReason, raw_header: String, raw_value: String },
}
