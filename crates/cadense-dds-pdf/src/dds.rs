use cadense_dsf::{DsfContainer, DsfError};
use cadense_order::OrderQueueEngine;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::Path;
use thiserror::Error;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

pub const DDS_MANIFEST_ENTRY: &str = "dds_manifest.json";
pub const DDS_DSF_PREFIX: &str = "components/";

#[derive(Debug, Error)]
pub enum DdsError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ZIP archive error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("DSF container error: {0}")]
    Dsf(#[from] DsfError),
    #[error("Missing manifest entry '{0}' in .dds container")]
    MissingEntry(&'static str),
    #[error("Referenced DSF file '{0}' not found in archive")]
    ComponentNotFound(String),
}

/// Project-level metadata recorded in the `.dds` archive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdsProjectMeta {
    pub dds_version: String,
    pub project_name: String,
    pub customer_name: String,
    pub order_reference: String,
    pub consultant: String,
    pub contractor: String,
    pub date_created: String,
    pub default_material_finish: String,
}

impl Default for DdsProjectMeta {
    fn default() -> Self {
        Self {
            dds_version: "1.0".to_string(),
            project_name: "CMS SUBMITTAL DOSSIER".to_string(),
            customer_name: "STANDARD CUSTOMER".to_string(),
            order_reference: format!("ORD-{}", Utc::now().format("%Y%m%d")),
            consultant: "DAR AL HANDASAH / KHATIB & ALAMI".to_string(),
            contractor: "MAIN MEP CONTRACTOR".to_string(),
            date_created: Utc::now().format("%d-%b-%Y").to_string(),
            default_material_finish: "HOT DIP GALVANIZED (HDG)".to_string(),
        }
    }
}

/// Manifest listing all items and mapped customer orders inside the `.dds`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdsManifest {
    pub meta: DdsProjectMeta,
    pub component_entries: Vec<String>,
    pub item_schedule: Vec<DdsItemRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdsItemRow {
    pub sn: u32,
    pub product_code: String,
    pub dsf_filename: String,
    pub description: String,
    pub dimensions: BTreeMap<String, f64>,
    pub qty: u32,
    pub finish: String,
    pub thickness_mm: f64,
}

/// The Danish Data Sheet (`.dds`) self-contained collection container.
#[derive(Debug, Clone)]
pub struct DdsArchive {
    pub manifest: DdsManifest,
    pub components: BTreeMap<String, Vec<u8>>,
}

impl DdsArchive {
    pub fn new(meta: DdsProjectMeta) -> Self {
        Self { manifest: DdsManifest { meta, component_entries: Vec::new(), item_schedule: Vec::new() }, components: BTreeMap::new() }
    }

    /// Embeds a `.dsf` container into the `.dds` archive.
    pub fn add_dsf(&mut self, filename: &str, dsf: &DsfContainer) -> Result<(), DdsError> {
        let bytes = dsf.to_bytes()?;
        let path = format!("{}{}", DDS_DSF_PREFIX, filename);
        if !self.manifest.component_entries.contains(&path) {
            self.manifest.component_entries.push(path.clone());
        }
        self.components.insert(path, bytes);
        Ok(())
    }

    /// Extracts a `.dsf` container from the `.dds` archive.
    pub fn get_dsf(&self, filename: &str) -> Result<DsfContainer, DdsError> {
        let path = if filename.starts_with(DDS_DSF_PREFIX) { filename.to_string() } else { format!("{}{}", DDS_DSF_PREFIX, filename) };

        let bytes = self.components.get(&path).ok_or_else(|| DdsError::ComponentNotFound(path.clone()))?;
        Ok(DsfContainer::from_bytes(bytes)?)
    }

    /// Populates items from an evaluated `OrderQueueEngine`.
    pub fn populate_from_order_queue(&mut self, queue: &OrderQueueEngine) {
        self.manifest.item_schedule.clear();
        for item in &queue.items {
            let mut dims = BTreeMap::new();

            let (product_code, thickness_mm, finish_str, description) = if let Some(res) = &item.resolved {
                for (&port_idx, &(w, h)) in &res.actual_ports_mm {
                    dims.insert(format!("W{}", port_idx), w);
                    dims.insert(format!("H{}", port_idx), h);
                    if port_idx == 1 {
                        dims.insert("W".to_string(), w);
                        dims.insert("H".to_string(), h);
                    }
                }
                if let Some(l) = res.length_mm.or(item.raw_row.l) {
                    dims.insert("L".to_string(), l);
                }
                let t = res.sheet_rule.thickness_mm;
                dims.insert("T".to_string(), t);
                (res.catalog_code.as_str().to_string(), t, res.sheet_rule.finish.code().to_string(), res.canonical_variation_name.clone())
            } else {
                let raw = &item.raw_row;
                if let Some(w) = raw.w.or(raw.w1) {
                    dims.insert("W".to_string(), w);
                    dims.insert("W1".to_string(), w);
                }
                if let Some(w2) = raw.w2 {
                    dims.insert("W2".to_string(), w2);
                }
                if let Some(w3) = raw.w3 {
                    dims.insert("W3".to_string(), w3);
                }
                if let Some(w4) = raw.w4 {
                    dims.insert("W4".to_string(), w4);
                }
                if let Some(h) = raw.h.or(raw.h1) {
                    dims.insert("H".to_string(), h);
                    dims.insert("H1".to_string(), h);
                }
                if let Some(l) = raw.l {
                    dims.insert("L".to_string(), l);
                }
                let t = raw.t.unwrap_or(1.5);
                dims.insert("T".to_string(), t);
                let code = if raw.product_name.is_empty() { "CT-STR".to_string() } else { raw.product_name.clone() };
                let fin = raw.finish.clone().unwrap_or_else(|| "HDG".to_string());
                (code, t, fin, raw.product_name.clone())
            };

            let sn: u32 = item.raw_row.sn.trim().parse().unwrap_or(item.raw_row.row_number as u32);

            self.manifest.item_schedule.push(DdsItemRow {
                sn,
                dsf_filename: format!("{}.dsf", product_code.to_lowercase()),
                product_code,
                description,
                dimensions: dims,
                qty: item.raw_row.qty,
                finish: finish_str,
                thickness_mm,
            });
        }
    }

    /// Saves the `.dds` archive to a file.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), DdsError> {
        let bytes = self.to_bytes()?;
        let mut f = File::create(path)?;
        f.write_all(&bytes)?;
        Ok(())
    }

    /// Loads a `.dds` archive from a file.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, DdsError> {
        let mut f = File::open(path)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        Self::from_bytes(&buf)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, DdsError> {
        let mut buf = Vec::new();
        {
            let cursor = Cursor::new(&mut buf);
            let mut zip = ZipWriter::new(cursor);
            let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated).unix_permissions(0o644);

            let manifest_bytes = serde_json::to_vec_pretty(&self.manifest)?;
            zip.start_file(DDS_MANIFEST_ENTRY, options)?;
            zip.write_all(&manifest_bytes)?;

            for (path, bytes) in &self.components {
                zip.start_file(path, options)?;
                zip.write_all(bytes)?;
            }
            zip.finish()?;
        }
        Ok(buf)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DdsError> {
        let cursor = Cursor::new(bytes);
        let mut archive = ZipArchive::new(cursor)?;

        let manifest: DdsManifest = {
            let mut mf = archive.by_name(DDS_MANIFEST_ENTRY).map_err(|_| DdsError::MissingEntry(DDS_MANIFEST_ENTRY))?;
            let mut buf = Vec::new();
            mf.read_to_end(&mut buf)?;
            serde_json::from_slice(&buf)?
        };

        let mut components = BTreeMap::new();
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();
            if name.starts_with(DDS_DSF_PREFIX) {
                let mut data = Vec::new();
                file.read_to_end(&mut data)?;
                components.insert(name, data);
            }
        }

        Ok(Self { manifest, components })
    }
}
