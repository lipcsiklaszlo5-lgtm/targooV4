use sha2::{Sha256, Digest};
use rusqlite::{Connection, params};
use chrono::{DateTime, Utc};
use crate::scope3_types::Scope3Extension;

#[derive(Debug, Clone)]
pub struct LedgerRow {
    pub row_id: Option<i64>,
    pub run_id: String,
    pub source_file: String,
    pub raw_row_index: u32,
    pub raw_header: String,
    pub ghg_scope: String,
    pub ghg_category: String,
    pub raw_value: f64,
    pub raw_unit: String,
    pub converted_value: f64,
    pub converted_unit: String,
    pub assumed_unit: Option<String>,
    pub emission_factor: f64,
    pub ef_source: String,
    pub tco2e: f64,
    pub confidence: f32,
    pub sha256_hash: String,
    pub created_at: DateTime<Utc>,
    pub scope3_ext: Option<Scope3Extension>,
}

#[derive(Debug, Clone)]
pub struct QuarantineRow {
    pub run_id: String,
    pub source_file: String,
    pub raw_row_index: u32,
    pub raw_header: String,
    pub raw_value: String,
    pub error_reason: String,
    pub suggested_fix: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub fn calculate_row_hash(
    run_id: &str,
    row_idx: u32,
    header: &str,
    raw_val: f64,
    tco2e: f64,
    prev_hash: &str,
) -> String {
    let input = format!("{}{}{}{}{:.8}{}", run_id, row_idx, header, raw_val, tco2e, prev_hash);
    let mut hasher = Sha256::new();
    hasher.update(input);
    hex::encode(hasher.finalize())
}

pub fn insert_ledger_row(
    conn: &Connection,
    row: &mut LedgerRow,
    prev_hash: &str,
) -> Result<String, String> {
    let new_hash = calculate_row_hash(
        &row.run_id,
        row.raw_row_index,
        &row.raw_header,
        row.raw_value,
        row.tco2e,
        prev_hash,
    );
    row.sha256_hash = new_hash.clone();

    let sql = "INSERT INTO esg_ledger (
        run_id, source_file, raw_row_index, raw_header, ghg_scope, ghg_category,
        raw_value, raw_unit, converted_value, converted_unit, assumed_unit,
        emission_factor, ef_source, tco2e, confidence, sha256_hash, created_at,
        scope3_category_id, calc_path, supplier_name, spend_currency,
        spend_usd_normalized, fx_rate_used, eeio_sector_code, eeio_source,
        data_quality_tier, ghg_protocol_dq_score
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27)";

    let created_at_str = row.created_at.to_rfc3339();

    let (
        s3_id, calc_path, spend_usd, eeio_code, eeio_src, dq_tier, dq_score
    ) = if let Some(ext) = &row.scope3_ext {
        (
            Some(ext.category_id as i64),
            Some(format!("{:?}", ext.calc_path)),
            ext.spend_usd_normalized,
            ext.eeio_sector_code.clone(),
            ext.eeio_source.clone(),
            Some(format!("{:?}", ext.data_quality_tier)),
            Some(ext.ghg_protocol_dq_score as i64),
        )
    } else {
        (None, None, None, None, None, None, None)
    };

    // Note: supplier_name, spend_currency, fx_rate_used are currently Null as they are not in Scope3Extension
    conn.execute(
        sql,
        params![
            row.run_id, row.source_file, row.raw_row_index, row.raw_header,
            row.ghg_scope, row.ghg_category, row.raw_value, row.raw_unit,
            row.converted_value, row.converted_unit, row.assumed_unit,
            row.emission_factor, row.ef_source, row.tco2e, row.confidence,
            row.sha256_hash, created_at_str,
            s3_id, calc_path, rusqlite::types::Null, rusqlite::types::Null,
            spend_usd, rusqlite::types::Null, eeio_code, eeio_src,
            dq_tier, dq_score
        ],
    ).map_err(|e| format!("Database error: {}", e))?;

    Ok(new_hash)
}

pub fn insert_quarantine_row(conn: &Connection, row: &QuarantineRow) -> Result<(), String> {
    let sql = "INSERT INTO quarantine_log (
        run_id, source_file, raw_row_index, raw_header, raw_value,
        error_reason, suggested_fix, created_at
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)";

    let created_at_str = row.created_at.to_rfc3339();

    conn.execute(
        sql,
        params![
            row.run_id, row.source_file, row.raw_row_index, row.raw_header,
            row.raw_value, row.error_reason, row.suggested_fix, created_at_str
        ],
    ).map_err(|e| format!("Database error: {}", e))?;

    Ok(())
}
