//! Customer Order CSV Batch Parser for Bonn Metal Cons. Ind. Submittal Sheets.
//! Supports flexible column names (`SN`, `Product Name`, `W`, `W1`..`W4`, `H`, `H1`..`H4`,
//! `T`, `L`, `QTY`, `Finish`) and inline dimension strings.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CsvParseError {
    #[error("Failed to read customer order CSV file: {0}")]
    Io(#[from] std::io::Error),
    #[error("CSV format error: {0}")]
    Csv(#[from] csv::Error),
}

/// Raw parsed row from a customer order CSV file prior to clearance and slot rule resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawOrderRow {
    pub row_number: usize,
    pub sn: String,
    pub product_name: String,
    pub w: Option<f64>,
    pub w1: Option<f64>,
    pub w2: Option<f64>,
    pub w3: Option<f64>,
    pub w4: Option<f64>,
    pub h: Option<f64>,
    pub h1: Option<f64>,
    pub h2: Option<f64>,
    pub h3: Option<f64>,
    pub h4: Option<f64>,
    pub t: Option<f64>,
    pub l: Option<f64>,
    pub qty: u32,
    pub finish: Option<String>,
    pub explicit_with_cover: Option<bool>,
    pub client_notes: Option<String>,
}

impl RawOrderRow {
    /// Returns the primary nominal width `W` (or `W1` if `W` is omitted).
    pub fn primary_width(&self) -> f64 {
        self.w.or(self.w1).unwrap_or(300.0)
    }

    /// Returns the primary nominal height `H` (or `H1` if `H` is omitted).
    pub fn primary_height(&self) -> f64 {
        self.h.or(self.h1).unwrap_or(50.0)
    }
}

/// Non-fatal diagnostic or validation warning emitted during CSV parsing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCsvDiagnostic {
    pub row_number: usize,
    pub sn: String,
    pub message: String,
    pub is_error: bool,
}

/// Complete report returned by `CustomerOrderCsvParser`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrderCsvParseReport {
    pub rows: Vec<RawOrderRow>,
    pub diagnostics: Vec<OrderCsvDiagnostic>,
}

pub struct CustomerOrderCsvParser;

impl CustomerOrderCsvParser {
    /// Parses a customer order CSV file from disk.
    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<OrderCsvParseReport, CsvParseError> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Self::parse_str(&content)
    }

    /// Parses customer order CSV text into `RawOrderRow` items and diagnostics.
    pub fn parse_str(csv_text: &str) -> Result<OrderCsvParseReport, CsvParseError> {
        let mut rdr = csv::ReaderBuilder::new().flexible(true).trim(csv::Trim::All).from_reader(csv_text.as_bytes());

        let headers = rdr.headers()?.clone();
        let mut col_map: HashMap<String, usize> = HashMap::new();
        for (idx, h) in headers.iter().enumerate() {
            col_map.insert(Self::normalize_header(h), idx);
        }

        let mut report = OrderCsvParseReport::default();

        for (zero_idx, record_res) in rdr.records().enumerate() {
            let row_number = zero_idx + 2; // 1-based including header row
            let record = match record_res {
                Ok(r) => r,
                Err(err) => {
                    report.diagnostics.push(OrderCsvDiagnostic {
                        row_number,
                        sn: format!("ROW-{}", row_number),
                        message: format!("Malformed CSV row: {}", err),
                        is_error: true,
                    });
                    continue;
                }
            };

            if record.iter().all(|field| field.trim().is_empty()) {
                continue;
            }

            let sn = Self::get_field(&record, &col_map, &["SN", "SNO", "SERIALNO", "ITEM", "ID"]).unwrap_or_else(|| format!("{:02}", zero_idx + 1));

            let product_name =
                match Self::get_field(&record, &col_map, &["PRODUCTNAME", "PRODUCT", "DESCRIPTION", "ITEMDESCRIPTION", "VARIATION", "TITLE"]) {
                    Some(name) if !name.is_empty() => name,
                    _ => {
                        report.diagnostics.push(OrderCsvDiagnostic {
                            row_number,
                            sn: sn.clone(),
                            message: "Missing Product Name column value.".to_string(),
                            is_error: true,
                        });
                        continue;
                    }
                };

            let w = Self::get_f64(&record, &col_map, &["W", "WIDTH", "WIDTHMM"]);
            let w1 = Self::get_f64(&record, &col_map, &["W1", "WIDTH1", "PORT1W"]);
            let w2 = Self::get_f64(&record, &col_map, &["W2", "WIDTH2", "PORT2W"]);
            let w3 = Self::get_f64(&record, &col_map, &["W3", "WIDTH3", "PORT3W"]);
            let w4 = Self::get_f64(&record, &col_map, &["W4", "WIDTH4", "PORT4W"]);

            let h = Self::get_f64(&record, &col_map, &["H", "HEIGHT", "HEIGHTMM"]);
            let h1 = Self::get_f64(&record, &col_map, &["H1", "HEIGHT1", "PORT1H"]);
            let h2 = Self::get_f64(&record, &col_map, &["H2", "HEIGHT2", "PORT2H"]);
            let h3 = Self::get_f64(&record, &col_map, &["H3", "HEIGHT3", "PORT3H"]);
            let h4 = Self::get_f64(&record, &col_map, &["H4", "HEIGHT4", "PORT4H"]);

            let t = Self::get_f64(&record, &col_map, &["T", "THK", "THICKNESS", "THICKNESSMM"]);
            let l = Self::get_f64(&record, &col_map, &["L", "LEN", "LENGTH", "LENGTHMM"]);

            let qty = Self::get_field(&record, &col_map, &["QTY", "QUANTITY", "PCS", "COUNT"])
                .and_then(|v| v.replace(',', "").parse::<u32>().ok())
                .unwrap_or(1);

            let finish = Self::get_field(&record, &col_map, &["FINISH", "MATERIALFINISH", "COATING", "MATERIAL"]);

            let explicit_with_cover = Self::get_field(&record, &col_map, &["COVER", "WITHCOVER", "WCOVER"]).map(|v| {
                let u = v.to_uppercase();
                u == "YES" || u == "TRUE" || u == "1" || u == "W/COVER"
            });

            let client_notes = Self::get_field(&record, &col_map, &["NOTES", "NOTE", "REMARKS"]);

            if w.is_none() && w1.is_none() && !product_name.to_uppercase().contains("COUPLER") {
                report.diagnostics.push(OrderCsvDiagnostic {
                    row_number,
                    sn: sn.clone(),
                    message: "No explicit W or W1 provided; defaulting to template baseline width.".to_string(),
                    is_error: false,
                });
            }

            report.rows.push(RawOrderRow {
                row_number,
                sn,
                product_name,
                w,
                w1,
                w2,
                w3,
                w4,
                h,
                h1,
                h2,
                h3,
                h4,
                t,
                l,
                qty,
                finish,
                explicit_with_cover,
                client_notes,
            });
        }

        Ok(report)
    }

    fn normalize_header(raw: &str) -> String {
        raw.trim().to_uppercase().chars().filter(|c| c.is_ascii_alphanumeric()).collect()
    }

    fn get_field(record: &csv::StringRecord, col_map: &HashMap<String, usize>, aliases: &[&str]) -> Option<String> {
        for &alias in aliases {
            if let Some(&idx) = col_map.get(alias) {
                if let Some(val) = record.get(idx) {
                    let trimmed = val.trim();
                    if !trimmed.is_empty() && trimmed != "-" && !trimmed.eq_ignore_ascii_case("N/A") {
                        return Some(trimmed.to_string());
                    }
                }
            }
        }
        None
    }

    fn get_f64(record: &csv::StringRecord, col_map: &HashMap<String, usize>, aliases: &[&str]) -> Option<f64> {
        Self::get_field(record, col_map, aliases).and_then(|s| {
            let cleaned: String = s.chars().take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-').collect();
            cleaned.parse::<f64>().ok()
        })
    }
}
