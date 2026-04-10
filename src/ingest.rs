use std::path::Path;
use calamine::{Reader, Xlsx, open_workbook, Data};
use csv::ReaderBuilder;

#[derive(Debug, Clone)]
pub struct RawDataRow {
    pub source_file: String,
    pub row_index: u32,
    pub headers: Vec<String>,
    pub values: Vec<String>,
}

pub fn parse_file(file_path: &str) -> Result<Vec<RawDataRow>, String> {
    let path = Path::new(file_path);
    let extension = path.extension()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "No file extension found".to_string())?
        .to_lowercase();

    match extension.as_str() {
        "csv" => parse_csv(file_path),
        "xlsx" => parse_xlsx(file_path),
        _ => Err(format!("Unsupported file extension: {}", extension)),
    }
}

fn parse_csv(file_path: &str) -> Result<Vec<RawDataRow>, String> {
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_path(file_path)
        .map_err(|e| format!("CSV open error: {}", e))?;

    let headers: Vec<String> = rdr.headers()
        .map_err(|e| format!("CSV header error: {}", e))?
        .iter()
        .map(|s| s.to_string())
        .collect();

    let mut rows = Vec::new();
    let file_name = Path::new(file_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(file_path)
        .to_string();

    for (row_idx, result) in rdr.records().enumerate() {
        let record = result.map_err(|e| format!("CSV record error: {}", e))?;
        let values: Vec<String> = record.iter().map(|s| s.to_string()).collect();
        rows.push(RawDataRow {
            source_file: file_name.clone(),
            row_index: (row_idx + 1) as u32,
            headers: headers.clone(),
            values,
        });
    }

    Ok(rows)
}

fn parse_xlsx(file_path: &str) -> Result<Vec<RawDataRow>, String> {
    let mut workbook: Xlsx<_> = open_workbook(file_path).map_err(|e| format!("XLSX open error: {}", e))?;
    let sheet_name = workbook.sheet_names()
        .first()
        .cloned()
        .ok_or_else(|| "No sheets found in XLSX".to_string())?;
    
    let range = workbook.worksheet_range(&sheet_name)
        .map_err(|e| format!("XLSX range error: {}", e))?;

    let mut rows = Vec::new();
    let file_name = Path::new(file_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(file_path)
        .to_string();

    let mut headers = Vec::<String>::new();
    for (row_idx, row) in range.rows().enumerate() {
        if row_idx == 0 {
            for cell in row {
                headers.push(cell.to_string());
            }
            continue;
        }

        let values: Vec<String> = row.iter().map(|cell| match cell {
            Data::Empty => String::new(),
            Data::String(s) => s.clone(),
            Data::Float(f) => f.to_string(),
            Data::Int(i) => i.to_string(),
            Data::Bool(b) => b.to_string(),
            Data::Error(e) => format!("{:?}", e),
            Data::DateTime(d) => d.to_string(),
            _ => cell.to_string(),
        }).collect();

        rows.push(RawDataRow {
            source_file: file_name.clone(),
            row_index: row_idx as u32,
            headers: headers.clone(),
            values,
        });
    }

    Ok(rows)
}
