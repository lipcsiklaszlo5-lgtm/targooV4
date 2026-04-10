use axum::{
    extract::{State, Multipart, Json},
    response::{IntoResponse, Response},
    http::{StatusCode, header},
    body::Body,
};
use crate::{SharedState};
use crate::models::{RunRequest, GhgCategory, Mapping};
use crate::ledger::{LedgerRow, QuarantineRow, insert_ledger_row, insert_quarantine_row};
use crate::ingest;
use crate::triage;
use crate::physics;
use crate::scope3_types::{Scope3Extension, CalcPath, MatchMethod, DataQualityTier};
use crate::scope3_classifier;
use crate::scope3_hybrid;
use crate::scope3_range;
use crate::aggregation;
use crate::output_factory;
use crate::db;
use serde_json::json;
use chrono::Utc;
use uuid::Uuid;

pub async fn upload_handler(State(state): State<SharedState>, mut multipart: Multipart) -> impl IntoResponse {
    let mut lock = state.lock().await;
    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = field.file_name().unwrap_or("unknown").to_string();
        let data = field.bytes().await.unwrap_or_default();
        let path = format!("/tmp/{}", file_name);
        if let Ok(_) = std::fs::write(&path, &data) {
            println!("Staged file: {}", path);
            lock.staged_files.push(path);
        }
    }
    StatusCode::OK
}

pub async fn run_handler(State(state): State<SharedState>, Json(payload): Json<RunRequest>) -> impl IntoResponse {
    let state_clone = state.clone();
    tokio::spawn(async move {
        let run_id = Uuid::new_v4().to_string();
        
        let (staged_files, dictionary) = {
            let mut lock = state_clone.lock().await;
            lock.status = "processing".to_string();
            lock.current_step = 1;
            lock.ledger.clear();
            lock.quarantine.clear();
            (lock.staged_files.clone(), lock.dictionary.clone())
        };

        let mut local_ledger = Vec::new();
        let mut local_quarantine = Vec::new();
        let mut prev_hash = "0".to_string();

        let conn_res = rusqlite::Connection::open_in_memory();
        let conn = match conn_res {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to open DB: {}", e);
                let mut lock = state_clone.lock().await;
                lock.status = "error".to_string();
                return;
            }
        };
        let _ = db::init_db(&conn);

        println!("Starting processing run_id: {}", run_id);
        for file_path in staged_files {
            println!("Processing file: {}", file_path);
            if let Ok(rows) = ingest::parse_file(&file_path) {
                println!("Parsed {} lines", rows.len());
                for raw_line in rows {
                    // Try each cell in the line to find a matching activity
                    let mut found_match = false;
                    
                    // First, try to find an activity/mapping among all values
                    let mut best_mapping: Option<(usize, Mapping, String, String)> = None; // (val_idx, mapping, header, value)

                    for (i, val) in raw_line.values.iter().enumerate() {
                        let header = raw_line.headers.get(i).cloned().unwrap_or_default();
                        
                        // Try triage on header or value
                        let mut mapping = triage::lookup_dictionary(&header, &dictionary);
                        if mapping.is_none() {
                            mapping = triage::lookup_dictionary(val, &dictionary);
                        }

                        if let Some(m) = mapping {
                            // If we find a mapping, we also need a numeric value in the SAME line
                            // Look for the first numeric value that isn't the activity itself
                            for (j, val_inner) in raw_line.values.iter().enumerate() {
                                if let Ok(num) = triage::parse_numeric(val_inner) {
                                    best_mapping = Some((j, m.clone(), header.clone(), val_inner.clone()));
                                    break;
                                }
                            }
                            if best_mapping.is_some() { break; }
                        }
                    }

                    if let Some((_, m, raw_h, raw_v)) = best_mapping {
                        if let Ok(val) = triage::parse_numeric(&raw_v) {
                            let mut tco2e = 0.0;
                            let mut s3_ext: Option<Scope3Extension> = None;
                            let conv_val = physics::convert_to_canonical(val, &m.canonical_unit);
                            let ef = 0.5;

                            if m.ghg_category == GhgCategory::Scope3 {
                                let s3_class = scope3_classifier::classify_scope3(&raw_h, &raw_v, &raw_line.source_file);
                                let (cat_id, _match_method, confidence) = match s3_class {
                                    scope3_classifier::Scope3Classification::Matched { category_id, match_method, confidence } => (category_id, match_method, confidence),
                                    scope3_classifier::Scope3Classification::Inferred { category_id, reason: _, confidence } => (category_id, "Inferred".to_string(), confidence),
                                    scope3_classifier::Scope3Classification::Unknown { error: _ } => (m.scope3_id.unwrap_or(1), "Dictionary".to_string(), 0.7),
                                };

                                if raw_v.contains('$') || raw_v.contains('€') || raw_v.contains('£') {
                                    if let Ok(spend_res) = scope3_hybrid::calculate_spend_based(&raw_v, &raw_h, cat_id, &payload.jurisdiction) {
                                        tco2e = spend_res.tco2e;
                                        s3_ext = Some(Scope3Extension {
                                            category_id: cat_id,
                                            category_name: format!("Category {}", cat_id),
                                            category_match_method: MatchMethod::Inferred,
                                            category_confidence: confidence,
                                            calc_path: CalcPath::SpendBased,
                                            spend_usd_normalized: Some(spend_res.usd_normalized),
                                            eeio_sector_code: Some(spend_res.eeio_sector_code),
                                            eeio_source: Some(spend_res.eeio_source),
                                            physical_quantity: None,
                                            physical_unit: None,
                                            data_quality_tier: DataQualityTier::Estimated,
                                            ghg_protocol_dq_score: 4,
                                        });
                                    }
                                } else {
                                    tco2e = physics::calculate_tco2e(conv_val, ef, 1.0);
                                    s3_ext = Some(Scope3Extension {
                                        category_id: cat_id,
                                        category_name: format!("Category {}", cat_id),
                                        category_match_method: MatchMethod::Exact,
                                        category_confidence: confidence,
                                        calc_path: CalcPath::ActivityBased,
                                        spend_usd_normalized: None,
                                        eeio_sector_code: None,
                                        eeio_source: None,
                                        physical_quantity: Some(conv_val),
                                        physical_unit: Some(m.canonical_unit.clone()),
                                        data_quality_tier: DataQualityTier::Secondary,
                                        ghg_protocol_dq_score: 3,
                                    });
                                }

                                if let Some(ext) = &s3_ext {
                                    if let Err(err_msg) = scope3_range::range_guard_check(tco2e, cat_id, &ext.calc_path) {
                                        local_quarantine.push(QuarantineRow {
                                            run_id: run_id.clone(),
                                            source_file: raw_line.source_file.clone(),
                                            raw_row_index: raw_line.row_index,
                                            raw_header: raw_h.clone(),
                                            raw_value: raw_v.clone(),
                                            error_reason: err_msg,
                                            suggested_fix: None,
                                            created_at: Utc::now(),
                                        });
                                        continue;
                                    }
                                }
                            } else {
                                tco2e = physics::calculate_tco2e(conv_val, ef, 1.0);
                                if let Err(e) = physics::validate_range(tco2e, &m.ghg_category, None) {
                                    local_quarantine.push(QuarantineRow {
                                        run_id: run_id.clone(),
                                        source_file: raw_line.source_file.clone(),
                                        raw_row_index: raw_line.row_index,
                                        raw_header: raw_h.clone(),
                                        raw_value: raw_v.clone(),
                                        error_reason: format!("{:?}", e),
                                        suggested_fix: None,
                                        created_at: Utc::now(),
                                    });
                                    continue;
                                }
                            }

                            let mut l_row = LedgerRow {
                                row_id: None,
                                run_id: run_id.clone(),
                                source_file: raw_line.source_file.clone(),
                                raw_row_index: raw_line.row_index,
                                raw_header: raw_h,
                                ghg_scope: format!("{:?}", m.ghg_category),
                                ghg_category: "Category".to_string(),
                                raw_value: val,
                                raw_unit: m.canonical_unit.clone(),
                                converted_value: conv_val,
                                converted_unit: m.canonical_unit.clone(),
                                assumed_unit: None,
                                emission_factor: ef,
                                ef_source: "MOCK".to_string(),
                                tco2e,
                                confidence: 1.0,
                                sha256_hash: String::new(),
                                created_at: Utc::now(),
                                scope3_ext: s3_ext,
                            };

                            if let Ok(h) = insert_ledger_row(&conn, &mut l_row, &prev_hash) {
                                prev_hash = h;
                                local_ledger.push(l_row);
                                found_match = true;
                            }
                        }
                    }

                    if !found_match {
                        local_quarantine.push(QuarantineRow {
                            run_id: run_id.clone(),
                            source_file: raw_line.source_file.clone(),
                            raw_row_index: raw_line.row_index,
                            raw_header: "Line".to_string(),
                            raw_value: raw_line.values.join("|"),
                            error_reason: "Triage failed for entire line".to_string(),
                            suggested_fix: None,
                            created_at: Utc::now(),
                        });
                    }
                }
            }
        }

        println!("Starting aggregation for {} rows", local_ledger.len());
        {
            let mut lock = state_clone.lock().await;
            lock.current_step = 4;
        }
        let agg = aggregation::aggregate_ledger(&local_ledger);

        println!("Generating Fritz Package...");
        {
            let mut lock = state_clone.lock().await;
            lock.current_step = 5;
        }
        let api_key = payload.gemini_api_key.clone().or_else(|| std::env::var("GEMINI_API_KEY").ok()).unwrap_or_else(|| "MISSING_KEY".to_string());
        let zip_res = output_factory::create_fritz_package(&run_id, &payload.language, &api_key, &agg, &local_ledger, &local_quarantine).await;

        println!("Finalizing processing...");
        {
            let mut lock = state_clone.lock().await;
            lock.ledger = local_ledger;
            lock.quarantine = local_quarantine;
            if let Ok(zip) = zip_res {
                println!("ZIP generated, success!");
                lock.zip_package = Some(zip);
                lock.status = "finished".to_string();
            } else {
                eprintln!("ZIP generation failed: {:?}", zip_res.err());
                lock.status = "error".to_string();
            }
            lock.current_step = 6;
        }
    });

    StatusCode::ACCEPTED
}

pub async fn status_handler(State(state): State<SharedState>) -> impl IntoResponse {
    let lock = state.lock().await;
    Json(json!({
        "status": lock.status,
        "current_step": lock.current_step,
        "ledger_count": lock.ledger.len(),
        "quarantine_count": lock.quarantine.len(),
    }))
}

pub async fn download_handler(State(state): State<SharedState>) -> impl IntoResponse {
    let lock = state.lock().await;
    if lock.status == "finished" {
        if let Some(zip) = &lock.zip_package {
            let res = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/zip")
                .header(header::CONTENT_DISPOSITION, "attachment; filename=Fritz_Package.zip")
                .body(Body::from(zip.clone()));
            
            match res {
                Ok(r) => return r.into_response(),
                Err(e) => {
                    eprintln!("Response building error: {}", e);
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            }
        }
    }
    StatusCode::NOT_FOUND.into_response()
}
