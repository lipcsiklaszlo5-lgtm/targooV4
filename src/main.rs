pub mod models;
pub mod triage;
pub mod physics;
pub mod scope3_classifier;
pub mod scope3_hybrid;
pub mod scope3_types;
pub mod scope3_range;
pub mod scope3_jurisdiction;
pub mod scope3_aggregation;
pub mod db;
pub mod ingest;
pub mod ledger;
pub mod aggregation;
pub mod i18n;
pub mod gemini_client;
pub mod output_factory;
pub mod api;

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use crate::models::DictionaryEntry;

pub struct AppState {
    pub status: String,
    pub current_step: u32,
    pub ledger: Vec<crate::ledger::LedgerRow>,
    pub quarantine: Vec<crate::ledger::QuarantineRow>,
    pub staged_files: Vec<String>,
    pub zip_package: Option<Vec<u8>>,
    pub dictionary: Vec<DictionaryEntry>,
}

pub type SharedState = Arc<Mutex<AppState>>;

#[tokio::main]
async fn main() {
    // Load dictionary
    let dictionary = match std::fs::read_to_string("data/dictionary.json") {
        Ok(content) => {
            serde_json::from_str::<Vec<DictionaryEntry>>(&content).unwrap_or_else(|e| {
                eprintln!("Warning: Failed to parse dictionary.json: {}", e);
                Vec::new()
            })
        }
        Err(e) => {
            eprintln!("Warning: Failed to read data/dictionary.json: {}", e);
            Vec::new()
        }
    };

    let state = Arc::new(Mutex::new(AppState {
        status: "idle".to_string(),
        current_step: 0,
        ledger: Vec::new(),
        quarantine: Vec::new(),
        staged_files: Vec::new(),
        zip_package: None,
        dictionary,
    }));

    let app = Router::new()
        .route("/upload", post(api::upload_handler))
        .route("/run", post(api::run_handler))
        .route("/status", get(api::status_handler))
        .route("/download", get(api::download_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener_res = tokio::net::TcpListener::bind("0.0.0.0:8080").await;
    match listener_res {
        Ok(listener) => {
            println!("Targoo V2 Server running on 0.0.0.0:8080");
            if let Err(e) = axum::serve(listener, app).await {
                eprintln!("Server error: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to bind to port 8080: {}", e);
        }
    }
}
