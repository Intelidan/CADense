//! `crates/cadense-dsf/src/container.rs`
//!
//! Step 2: Dual-Save Writer (`.solidcraft` Format 41 + `.dsf` Container),
//! In-Memory Byte Serialization (`to_bytes` / `from_bytes` for `.dds` dossiers),
//! and Live SolidCraft v0.25.1 Session Command Emitter.
//!
//! Packaging layout inside `<part>.dsf` (ZIP archive):
//! - `design.solidcraft`: Native SolidCraft v0.25.1 (Format 41) parametric design JSON with timeline
//! - `dsf_meta.json`: Port definitions (`P1..P4`), taxonomy ID, rigid slot rules, and TDS view config
//! - `rigged_sheet.json`: Full `DsfRiggedSheet` state (2D DXF loops, bend lines, kinematic fold tree)

use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::bending::DsfRiggedSheet;
use crate::schema::{DsfMeta, SolidCraftCommand};

/// SolidCraft v0.25.1 native design format version.
pub const SOLIDCRAFT_FORMAT_VERSION_V0_25_1: u32 = 41;

#[derive(Debug)]
pub enum DsfError {
    Io(std::io::Error),
    Zip(zip::result::ZipError),
    Json(serde_json::Error),
    MissingEntry(String),
}

impl std::fmt::Display for DsfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {}", e),
            Self::Zip(e) => write!(f, "ZIP container error: {}", e),
            Self::Json(e) => write!(f, "JSON serialization error: {}", e),
            Self::MissingEntry(name) => {
                write!(f, "Missing required entry in .dsf archive: {}", name)
            }
        }
    }
}

impl std::error::Error for DsfError {}

impl From<std::io::Error> for DsfError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<zip::result::ZipError> for DsfError {
    fn from(e: zip::result::ZipError) -> Self {
        Self::Zip(e)
    }
}

impl From<serde_json::Error> for DsfError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

/// Complete in-memory representation of a `.dsf` (Danish Sheet Fold) package.
#[derive(Clone, Debug)]
pub struct DsfContainer {
    /// Native SolidCraft v0.25.1 (Format 41) JSON bytes (`design.solidcraft`).
    pub solidcraft_design_bytes: Vec<u8>,
    /// CMS product taxonomy, port definitions (`P1..P4`), and rigid coupler slot rules.
    pub meta: DsfMeta,
    /// Optional full Stage 1 / Stage 2 rigged sheet state (2D loops, bend lines, kinematic fold tree).
    pub rigged_sheet: Option<DsfRiggedSheet>,
}

impl DsfContainer {
    pub fn new(solidcraft_design_bytes: Vec<u8>, meta: DsfMeta) -> Self {
        Self { solidcraft_design_bytes, meta, rigged_sheet: None }
    }

    /// Builds a complete `DsfContainer` directly from a `DsfRiggedSheet` and `DsfMeta`,
    /// automatically synthesizing the SolidCraft v0.25.1 (Format 41) design document and timeline.
    pub fn from_rigged_sheet(sheet: DsfRiggedSheet, meta: DsfMeta) -> Result<Self, DsfError> {
        let design_json = Self::build_solidcraft_format41_json(&sheet, &meta)?;
        let solidcraft_design_bytes = serde_json::to_vec_pretty(&design_json)?;
        Ok(Self { solidcraft_design_bytes, meta, rigged_sheet: Some(sheet) })
    }

    /// Synthesizes a native SolidCraft v0.25.1 (Format 41) JSON document containing:
    /// - Format 41 header (`format_version: 41`, `app_version: "0.25.1"`)
    /// - Unit-aware parameters (`W1..W4`, `H1..H4`, `T`, `R`, `K`)
    /// - Chronological feature timeline (`timeline_bundles` from `DsfRiggedSheet`)
    /// - Multi-body 3D B-rep primitives (3D planes, closed-loop sketches, extrudes, and construction-axis revolves)
    pub fn build_solidcraft_format41_json(sheet: &DsfRiggedSheet, meta: &DsfMeta) -> Result<serde_json::Value, DsfError> {
        let mut parameters = serde_json::Map::new();
        parameters.insert(
            "T".to_string(),
            serde_json::json!({
                "expression": format!("{} mm", sheet.material_rule.thickness_mm),
                "value_mm": sheet.material_rule.thickness_mm,
            }),
        );
        parameters.insert(
            "R".to_string(),
            serde_json::json!({
                "expression": format!("{} mm", sheet.material_rule.inner_radius_mm),
                "value_mm": sheet.material_rule.inner_radius_mm,
            }),
        );
        parameters.insert(
            "K".to_string(),
            serde_json::json!({
                "expression": format!("{}", sheet.material_rule.k_factor),
                "value": sheet.material_rule.k_factor,
            }),
        );

        for port in &meta.ports {
            let w_key = if port.width_param.is_empty() { format!("W{}", port.id) } else { port.width_param.clone() };
            let h_key = if port.height_param.is_empty() { format!("H{}", port.id) } else { port.height_param.clone() };

            parameters.insert(
                w_key,
                serde_json::json!({
                    "id": port.id,
                    "label": port.label,
                    "expression": format!("{} mm", port.default_width_mm),
                    "value_mm": port.default_width_mm,
                }),
            );
            parameters.insert(
                h_key,
                serde_json::json!({
                    "id": port.id,
                    "label": port.label,
                    "expression": format!("{} mm", port.default_height_mm),
                    "value_mm": port.default_height_mm,
                }),
            );
        }

        let timeline_features: Vec<serde_json::Value> = sheet
            .timeline_bundles
            .iter()
            .enumerate()
            .map(|(idx, bundle)| {
                serde_json::json!({
                    "index": idx,
                    "feature_group_id": bundle.feature_group_id,
                    "name": bundle.display_name,
                    "suppressed": false,
                    "commands": bundle.commands,
                })
            })
            .collect();

        Ok(serde_json::json!({
            "format": "solidcraft",
            "format_version": SOLIDCRAFT_FORMAT_VERSION_V0_25_1,
            "app_version": "0.25.1",
            "name": sheet.product_name,
            "product_code": sheet.product_code,
            "units": "mm",
            "sheet_metal_rule": {
                "name": sheet.material_rule.name,
                "thickness_mm": sheet.material_rule.thickness_mm,
                "inner_radius_mm": sheet.material_rule.inner_radius_mm,
                "k_factor": sheet.material_rule.k_factor,
            },
            "parameters": parameters,
            "timeline": timeline_features,
            "command_stream": sheet.emit_all_solidcraft_commands(),
        }))
    }

    /// Serializes this `.dsf` container into an in-memory ZIP byte vector (used by `cadense-dds-pdf`
    /// and disk persistence).
    pub fn to_bytes(&self) -> Result<Vec<u8>, DsfError> {
        let mut buf = Cursor::new(Vec::new());
        {
            let mut zip = ZipWriter::new(&mut buf);
            let options: FileOptions<'_, ()> = FileOptions::default().compression_method(CompressionMethod::Deflated).unix_permissions(0o644);

            // 1. Write native SolidCraft Format 41 design file
            zip.start_file("design.solidcraft", options)?;
            zip.write_all(&self.solidcraft_design_bytes)?;

            // 2. Write CMS metadata (`dsf_meta.json`)
            let meta_bytes = serde_json::to_vec_pretty(&self.meta)?;
            zip.start_file("dsf_meta.json", options)?;
            zip.write_all(&meta_bytes)?;

            // 3. Write full rigged sheet state (`rigged_sheet.json`) if present
            if let Some(sheet) = &self.rigged_sheet {
                let sheet_bytes = serde_json::to_vec_pretty(sheet)?;
                zip.start_file("rigged_sheet.json", options)?;
                zip.write_all(&sheet_bytes)?;
            }

            zip.finish()?;
        }
        Ok(buf.into_inner())
    }

    /// Deserializes a `DsfContainer` from an in-memory ZIP byte slice (used by `cadense-dds-pdf`
    /// and disk loading).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DsfError> {
        let reader = Cursor::new(bytes);
        let mut archive = ZipArchive::new(reader)?;

        let mut solidcraft_design_bytes = Vec::new();
        {
            let mut sc_file = archive.by_name("design.solidcraft").map_err(|_| DsfError::MissingEntry("design.solidcraft".to_string()))?;
            sc_file.read_to_end(&mut solidcraft_design_bytes)?;
        }

        let meta: DsfMeta = {
            let mut meta_file = archive.by_name("dsf_meta.json").map_err(|_| DsfError::MissingEntry("dsf_meta.json".to_string()))?;
            let mut meta_bytes = Vec::new();
            meta_file.read_to_end(&mut meta_bytes)?;
            serde_json::from_slice(&meta_bytes)?
        };

        let rigged_sheet: Option<DsfRiggedSheet> = match archive.by_name("rigged_sheet.json") {
            Ok(mut sheet_file) => {
                let mut sheet_bytes = Vec::new();
                sheet_file.read_to_end(&mut sheet_bytes)?;
                let mut sheet: DsfRiggedSheet = serde_json::from_slice(&sheet_bytes)?;
                sheet.rebuild_fold_tree();
                Some(sheet)
            }
            Err(_) => None,
        };

        Ok(Self { solidcraft_design_bytes, meta, rigged_sheet })
    }

    /// Saves ONLY the native `.solidcraft` (Format 41) file so it can be opened directly
    /// in the upstream SolidCraft desktop or web application.
    pub fn save_solidcraft_file(&self, path: impl AsRef<Path>) -> Result<PathBuf, DsfError> {
        let target = path.as_ref().with_extension("solidcraft");
        if let Some(parent) = target.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(&target, &self.solidcraft_design_bytes)?;
        Ok(target)
    }

    /// Saves the `.dsf` container (`design.solidcraft` + `dsf_meta.json` + `rigged_sheet.json`) to disk.
    pub fn save_to_path(&self, path: impl AsRef<Path>) -> Result<PathBuf, DsfError> {
        let target = path.as_ref().with_extension("dsf");
        if let Some(parent) = target.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let bytes = self.to_bytes()?;
        fs::write(&target, bytes)?;
        Ok(target)
    }

    /// Alias for `save_to_path` used by `apps/cadense-studio`.
    /// Legacy compatibility helper: saves the `.dsf` container to `path`.
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<PathBuf, DsfError> {
        self.save_to_path(path)
    }

    /// Dual-Save Writer: Atomically saves BOTH `<base>.solidcraft` (Format 41 for SolidCraft app)
    /// AND `<base>.dsf` (for CADense CMS Studio) side-by-side.
    pub fn save_dual_solidcraft_and_dsf(&self, base_path: impl AsRef<Path>) -> Result<(PathBuf, PathBuf), DsfError> {
        let sc_path = self.save_solidcraft_file(&base_path)?;
        let dsf_path = self.save_to_path(&base_path)?;
        Ok((sc_path, dsf_path))
    }

    /// Loads a `.dsf` container from disk, restoring `design.solidcraft`, `dsf_meta.json`,
    /// and `rigged_sheet.json` (if present).
    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, DsfError> {
        let bytes = fs::read(path)?;
        Self::from_bytes(&bytes)
    }

    /// Legacy compatibility helper: loads the `.dsf` container from `path`.
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, DsfError> {
        Self::load_from_path(path)
    }
}

/// Live Session Command Bridge for driving SolidCraft's 3D viewport on every bend operation.
pub struct SolidCraftLiveSessionBridge;

impl SolidCraftLiveSessionBridge {
    /// Emits the complete chronological list of `SolidCraftCommand`s needed to rebuild
    /// the 3D folded sheet metal model inside SolidCraft's active session.
    pub fn build_live_viewport_commands(sheet: &DsfRiggedSheet) -> Vec<SolidCraftCommand> {
        let mut cmds = Vec::with_capacity(sheet.timeline_bundles.len() * 3 + 2);

        // 1. Sync active sheet metal thickness & K-factor rules in the session
        cmds.push(SolidCraftCommand {
            command: "sheet.manage_rules".to_string(),
            args: serde_json::json!({
                "name": sheet.material_rule.name,
                "thickness_mm": sheet.material_rule.thickness_mm,
                "bend_radius_mm": sheet.material_rule.inner_radius_mm,
                "k_factor": sheet.material_rule.k_factor,
                "relief_type": "none",
                "relief_width_mm": 0.0,
                "relief_depth_mm": 0.0,
            }),
        });

        // 2. Append all flat face extrudes and cylindrical bend-radius revolves
        cmds.extend(sheet.emit_all_solidcraft_commands());

        cmds
    }

    /// Exports the live command sequence as a SolidCraft CLI / MCP compatible script JSON string.
    pub fn to_solidcraft_script_json(sheet: &DsfRiggedSheet) -> Result<String, DsfError> {
        let cmds = Self::build_live_viewport_commands(sheet);
        Ok(serde_json::to_string_pretty(&cmds)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bending::{BendDirection, Loop2D, SheetMaterialRule};
    use crate::catalog::bonn_metal_catalog;

    #[test]
    fn test_dual_save_and_dsf_roundtrip_with_rigged_sheet() {
        let outer = Loop2D::rectangular_blank(400.0, 300.0);
        let mut sheet = DsfRiggedSheet::from_dxf_loops(outer, Vec::new(), SheetMaterialRule::bonn_metal_1_5mm());
        sheet.add_bend([-100.0, -150.0], [-100.0, 150.0], 90.0, BendDirection::Up);
        sheet.add_bend([100.0, -150.0], [100.0, 150.0], 90.0, BendDirection::Up);

        let catalog = bonn_metal_catalog();
        let meta = catalog.into_iter().next().expect("catalog has presets");

        let container = DsfContainer::from_rigged_sheet(sheet.clone(), meta).expect("build container");

        // Verify in-memory `to_bytes` / `from_bytes` roundtrip (used by `cadense-dds-pdf`)
        let raw_bytes = container.to_bytes().expect("to_bytes");
        let from_mem = DsfContainer::from_bytes(&raw_bytes).expect("from_bytes");
        assert!(from_mem.rigged_sheet.is_some());

        let temp_dir = std::env::temp_dir().join("cadense_step2_test");
        let base_path = temp_dir.join("ct_str_test");

        let (sc_path, dsf_path) = container.save_dual_solidcraft_and_dsf(&base_path).expect("dual save");

        assert!(sc_path.exists(), ".solidcraft file must exist");
        assert!(dsf_path.exists(), ".dsf file must exist");

        let loaded = DsfContainer::load_from_path(&dsf_path).expect("load .dsf");
        let loaded_sheet = loaded.rigged_sheet.expect("rigged_sheet preserved");
        assert_eq!(loaded_sheet.bends.len(), 2);
        assert_eq!(loaded_sheet.face_nodes.len(), 3);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
