use rust_xlsxwriter::Workbook;
use zip::write::FileOptions;
use zip::ZipWriter;
use std::io::{Write, Cursor};
use sha2::{Sha256, Digest};
use crate::ledger::{LedgerRow, QuarantineRow};
use crate::aggregation::ScopeAggregation;
use crate::models::Language;
use serde_json::json;

async fn generate_summary_xlsx(agg: &ScopeAggregation, _lang: &Language) -> Vec<u8> {
    let mut workbook = Workbook::new();
    let res = (|| {
        let worksheet = workbook.add_worksheet().set_name("GHG Inventar 2024")?;
        worksheet.write_string(0, 0, "GHG Inventory Summary 2024")?;
        worksheet.write_string(2, 0, "Scope 1 Total:")?;
        worksheet.write_number(2, 1, agg.scope1_total_tco2e)?;
        worksheet.write_string(3, 0, "Scope 2 Total:")?;
        worksheet.write_number(3, 1, agg.scope2_total_tco2e)?;
        worksheet.write_string(4, 0, "Scope 3 Total:")?;
        worksheet.write_number(4, 1, agg.scope3_data.grand_total_tco2e)?;
        worksheet.write_string(6, 0, "Total Inventory:")?;
        worksheet.write_number(6, 1, agg.total_inventory_tco2e)?;
        workbook.save_to_buffer()
    })();

    match res {
        Ok(buf) => buf.to_vec(),
        Err(_) => Vec::new(),
    }
}

async fn generate_scope_breakdown_xlsx(_agg: &ScopeAggregation) -> Vec<u8> {
    let mut workbook = Workbook::new();
    let res = (|| {
        workbook.add_worksheet().set_name("Scope 1 Detail")?;
        workbook.add_worksheet().set_name("Scope 2 Detail")?;
        workbook.add_worksheet().set_name("Scope 3 Kategorien")?;
        workbook.add_worksheet().set_name("Top 10 Hotspots")?;
        workbook.save_to_buffer()
    })();
    
    match res {
        Ok(buf) => buf.to_vec(),
        Err(_) => Vec::new(),
    }
}

async fn generate_audit_trail_xlsx(_ledger: &[LedgerRow]) -> Vec<u8> {
    let mut workbook = Workbook::new();
    let res = (|| {
        workbook.add_worksheet().set_name("Verarbeitete Zeilen")?;
        workbook.add_worksheet().set_name("Angenommene Einheiten")?;
        workbook.add_worksheet().set_name("Prüfsummen-Kette")?;
        workbook.save_to_buffer()
    })();
    
    match res {
        Ok(buf) => buf.to_vec(),
        Err(_) => Vec::new(),
    }
}

async fn generate_quarantine_xlsx(_quarantine: &[QuarantineRow]) -> Vec<u8> {
    let mut workbook = Workbook::new();
    let res = (|| {
        workbook.add_worksheet().set_name("Quarantäne-Übersicht")?;
        workbook.add_worksheet().set_name("Nach Fehlertyp")?;
        workbook.save_to_buffer()
    })();
    
    match res {
        Ok(buf) => buf.to_vec(),
        Err(_) => Vec::new(),
    }
}

async fn generate_ef_reference_xlsx() -> Vec<u8> {
    let mut workbook = Workbook::new();
    let res = (|| {
        workbook.add_worksheet().set_name("Verwendete Faktoren")?;
        workbook.add_worksheet().set_name("GWP100 Referenztabelle")?;
        workbook.save_to_buffer()
    })();
    
    match res {
        Ok(buf) => buf.to_vec(),
        Err(_) => Vec::new(),
    }
}

fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

pub async fn create_fritz_package(
    _run_id: &str,
    lang: &Language,
    api_key: &str,
    agg: &ScopeAggregation,
    ledger: &[LedgerRow],
    quarantine: &[QuarantineRow],
) -> Result<Vec<u8>, String> {
    let summary_fut = generate_summary_xlsx(agg, lang);
    let breakdown_fut = generate_scope_breakdown_xlsx(agg);
    let audit_fut = generate_audit_trail_xlsx(ledger);
    let quarantine_fut = generate_quarantine_xlsx(quarantine);
    let ef_fut = generate_ef_reference_xlsx();
    let narrative_fut = crate::gemini_client::generate_narrative(agg, lang, api_key);

    let (
        summary_bin, breakdown_bin, audit_bin, quarantine_bin, ef_bin, narrative_res
    ) = tokio::join!(summary_fut, breakdown_fut, audit_fut, quarantine_fut, ef_fut, narrative_fut);

    let narrative_md = narrative_res.unwrap_or_else(|_| "Fallback narrative".to_string());
    let narrative_bytes = narrative_md.into_bytes();
    
    let manifest = json!({
        "01_GHG_Inventar_Zusammenfassung.xlsx": hash_bytes(&summary_bin),
        "02_Scope_Aufschluesselung.xlsx": hash_bytes(&breakdown_bin),
        "03_Audit_Trail_Master.xlsx": hash_bytes(&audit_bin),
        "04_Quarantaene_Log.xlsx": hash_bytes(&quarantine_bin),
        "05_Emissionsfaktoren_Referenz.xlsx": hash_bytes(&ef_bin),
        "06_Narrative_Bericht.md": hash_bytes(&narrative_bytes),
    });
    let manifest_bin = serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?;

    let mut zip_buf = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut zip_buf));
        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);

        zip.start_file("00_Manifest.json", options).map_err(|e| e.to_string())?;
        zip.write_all(&manifest_bin).map_err(|e| e.to_string())?;

        zip.start_file("01_GHG_Inventar_Zusammenfassung.xlsx", options).map_err(|e| e.to_string())?;
        zip.write_all(&summary_bin).map_err(|e| e.to_string())?;

        zip.start_file("02_Scope_Aufschluesselung.xlsx", options).map_err(|e| e.to_string())?;
        zip.write_all(&breakdown_bin).map_err(|e| e.to_string())?;

        zip.start_file("03_Audit_Trail_Master.xlsx", options).map_err(|e| e.to_string())?;
        zip.write_all(&audit_bin).map_err(|e| e.to_string())?;

        zip.start_file("04_Quarantaene_Log.xlsx", options).map_err(|e| e.to_string())?;
        zip.write_all(&quarantine_bin).map_err(|e| e.to_string())?;

        zip.start_file("05_Emissionsfaktoren_Referenz.xlsx", options).map_err(|e| e.to_string())?;
        zip.write_all(&ef_bin).map_err(|e| e.to_string())?;

        zip.start_file("06_Narrative_Bericht.md", options).map_err(|e| e.to_string())?;
        zip.write_all(&narrative_bytes).map_err(|e| e.to_string())?;

        zip.finish().map_err(|e| e.to_string())?;
    }

    Ok(zip_buf)
}
