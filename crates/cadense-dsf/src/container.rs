use crate::schema::DsfMeta;
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::Path;
use thiserror::Error;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

pub const SOLIDCRAFT_ENTRY_NAME: &str = "design.solidcraft";
pub const DSF_META_ENTRY_NAME: &str = "dsf_meta.json";

#[derive(Debug, Error)]
pub enum DsfError {
    #[error("I/O error while processing .dsf container: {0}")]
    Io(#[from] std::io::Error),
    #[error("ZIP archive error in .dsf container: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("JSON serialization error in dsf_meta.json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Missing required entry '{0}' inside .dsf container")]
    MissingEntry(&'static str),
}

#[derive(Debug, Clone)]
pub struct DsfContainer {
    pub solidcraft_bytes: Vec<u8>,
    pub meta: DsfMeta,
}

impl DsfContainer {
    pub fn new(solidcraft_bytes: Vec<u8>, meta: DsfMeta) -> Self {
        Self {
            solidcraft_bytes,
            meta,
        }
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), DsfError> {
        let bytes = self.to_bytes()?;
        let mut file = File::create(path)?;
        file.write_all(&bytes)?;
        Ok(())
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, DsfError> {
        let mut file = File::open(path)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        Self::from_bytes(&buf)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, DsfError> {
        let mut buf = Vec::new();
        {
            let cursor = Cursor::new(&mut buf);
            let mut zip = ZipWriter::new(cursor);
            let options = SimpleFileOptions::default()
                .compression_method(CompressionMethod::Deflated)
                .unix_permissions(0o644);

            zip.start_file(SOLIDCRAFT_ENTRY_NAME, options)?;
            zip.write_all(&self.solidcraft_bytes)?;

            let meta_json = serde_json::to_vec_pretty(&self.meta)?;
            zip.start_file(DSF_META_ENTRY_NAME, options)?;
            zip.write_all(&meta_json)?;

            zip.finish()?;
        }
        Ok(buf)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DsfError> {
        let cursor = Cursor::new(bytes);
        let mut archive = ZipArchive::new(cursor)?;

        let mut solidcraft_bytes = Vec::new();
        {
            let mut sc_file = archive
                .by_name(SOLIDCRAFT_ENTRY_NAME)
                .map_err(|_| DsfError::MissingEntry(SOLIDCRAFT_ENTRY_NAME))?;
            sc_file.read_to_end(&mut solidcraft_bytes)?;
        }

        let meta: DsfMeta = {
            let mut meta_file = archive
                .by_name(DSF_META_ENTRY_NAME)
                .map_err(|_| DsfError::MissingEntry(DSF_META_ENTRY_NAME))?;
            let mut meta_bytes = Vec::new();
            meta_file.read_to_end(&mut meta_bytes)?;
            serde_json::from_slice(&meta_bytes)?
        };

        Ok(Self {
            solidcraft_bytes,
            meta,
        })
    }
}
