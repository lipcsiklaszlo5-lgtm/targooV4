use rusqlite::{Connection, Result};

pub fn init_db(conn: &Connection) -> Result<()> {
    // Create esg_ledger table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS esg_ledger (
            id INTEGER PRIMARY KEY,
            run_id TEXT,
            source_file TEXT,
            raw_row_index INTEGER,
            raw_header TEXT,
            ghg_scope TEXT,
            ghg_category TEXT,
            raw_value REAL,
            raw_unit TEXT,
            converted_value REAL,
            converted_unit TEXT,
            assumed_unit TEXT,
            emission_factor REAL,
            ef_source TEXT,
            tco2e REAL,
            confidence REAL,
            sha256_hash TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            
            -- Scope 3 Extension fields
            scope3_category_id INTEGER,
            calc_path TEXT,
            supplier_name TEXT,
            spend_currency TEXT,
            spend_usd_normalized REAL,
            fx_rate_used REAL,
            eeio_sector_code TEXT,
            eeio_source TEXT,
            data_quality_tier TEXT,
            ghg_protocol_dq_score INTEGER
        )",
        [],
    )?;

    // Create quarantine_log table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS quarantine_log (
            id INTEGER PRIMARY KEY,
            run_id TEXT,
            source_file TEXT,
            raw_row_index INTEGER,
            raw_header TEXT,
            raw_value TEXT,
            error_reason TEXT,
            suggested_fix TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Create WORM Triggers
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS prevent_ledger_update 
         BEFORE UPDATE ON esg_ledger 
         BEGIN 
            SELECT RAISE(ABORT, 'WORM violation: Updates prohibited on esg_ledger'); 
         END;",
        [],
    )?;

    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS prevent_ledger_delete 
         BEFORE DELETE ON esg_ledger 
         BEGIN 
            SELECT RAISE(ABORT, 'WORM violation: Deletions prohibited on esg_ledger'); 
         END;",
        [],
    )?;

    Ok(())
}
