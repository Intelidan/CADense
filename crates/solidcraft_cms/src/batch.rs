//! Headless Batch Ingestion & CAM Matrix Processing Engine.
//!
//! Automatically iterates customer order / catalog CSV matrices, morphs 2D profiles,
//! preserves punch tooling, folds 3D solid flanges, and exports manufacturing asset packages (.dxf + .step).

use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::data::{BendDirection, CmsFoldParams, CmsProductType, CmsSlotMetadata};
use crate::eval::{execute_relaxed_fold, migrate_rigid_coupler_slots, morph_base_2d_sketch, reposition_bend_lines};
use crate::export::{export_2d_flat_dxf, export_3d_step};
use crate::topology::cms_identify_topology;
use crate::trace::StructuralWireLoop;

#[derive(Error, Debug)]
pub enum BatchError {
    #[error("CSV Parsing Error at line {line}: {msg}")]
    CsvParse { line: usize, msg: String },
    #[error("I/O error during batch CAM execution: {0}")]
    Io(#[from] std::io::Error),
    #[error("Export error during batch processing: {0}")]
    Export(#[from] crate::export::ExportError),
    #[error("Topology error: {0}")]
    Topology(String),
}

/// A parsed row entry from a customer order or catalog CSV file.
#[derive(Clone, Debug, PartialEq)]
pub struct CmsOrderRow {
    pub item_code: String,
    pub category: String,
    pub product_type: CmsProductType,
    pub variation: String,
    pub width: f64,
    pub secondary_width: Option<f64>,
    pub height: f64,
    pub thickness: f64,
    pub angle_deg: f64,
    pub direction: BendDirection,
}

/// Result record for an individually processed batch item.
#[derive(Clone, Debug)]
pub struct BatchItemResult {
    pub item_code: String,
    pub flat_dxf_path: PathBuf,
    pub folded_step_path: PathBuf,
    pub triangle_count: usize,
    pub slots_preserved: usize,
}

/// Summary report of a complete headless batch run.
#[derive(Clone, Debug, Default)]
pub struct BatchProcessingReport {
    pub total_rows: usize,
    pub successful_items: Vec<BatchItemResult>,
    pub failed_rows: Vec<(usize, String)>,
}

/// Parses a CSV string into structured `CmsOrderRow` definitions.
pub fn parse_cms_order_csv(csv_content: &str) -> Result<Vec<CmsOrderRow>, BatchError> {
    let mut rows = Vec::new();
    let lines = csv_content.lines().enumerate();

    // Check optional header row
    let mut first_line = true;

    for (line_idx, raw_line) in lines {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if first_line && parts.first().is_some_and(|p| p.eq_ignore_ascii_case("item_code") || p.eq_ignore_ascii_case("item")) {
            first_line = false;
            continue;
        }
        first_line = false;

        if parts.len() < 7 {
            return Err(BatchError::CsvParse {
                line: line_idx + 1,
                msg: format!("Expected at least 7 columns (Item_Code, Category, Product, Variation, Width, Height, Thickness), got {}", parts.len()),
            });
        }

        let item_code = parts[0].to_string();
        let category = parts[1].to_string();

        let prod_str = parts[2].to_ascii_lowercase();
        let product_type = if prod_str.contains("red") {
            CmsProductType::Reducer
        } else if prod_str.contains("tee") {
            CmsProductType::UnequalTee
        } else if prod_str.contains("bend") || prod_str.contains("elbow") {
            CmsProductType::StandardBendOrRiser
        } else if prod_str.contains("cross") {
            CmsProductType::UnequalCross
        } else if prod_str.contains("in") && prod_str.contains("riser") {
            CmsProductType::InsideRiser
        } else if prod_str.contains("out") && prod_str.contains("riser") {
            CmsProductType::OutsideRiser
        } else {
            CmsProductType::StandardBendOrRiser
        };

        let variation = parts[3].to_string();

        let width = parts[4].parse::<f64>().map_err(|e| BatchError::CsvParse {
            line: line_idx + 1,
            msg: format!("Invalid Width '{}': {}", parts[4], e),
        })?;

        let height = parts[5].parse::<f64>().map_err(|e| BatchError::CsvParse {
            line: line_idx + 1,
            msg: format!("Invalid Height '{}': {}", parts[5], e),
        })?;

        let thickness = parts[6].parse::<f64>().map_err(|e| BatchError::CsvParse {
            line: line_idx + 1,
            msg: format!("Invalid Thickness '{}': {}", parts[6], e),
        })?;

        // Optional secondary width (e.g. Reducer P2 or Tee branch)
        let secondary_width = if parts.len() > 7 && !parts[7].is_empty() {
            parts[7].parse::<f64>().ok()
        } else {
            // Deduce secondary width from item code if possible (e.g. BONN-CT-RED-300-150 -> 150)
            deduce_secondary_width_from_name(&item_code)
        };

        let angle_deg = if parts.len() > 8 && !parts[8].is_empty() {
            parts[8].parse::<f64>().unwrap_or(90.0)
        } else {
            90.0
        };

        let direction = if parts.len() > 9 && parts[9].eq_ignore_ascii_case("down") {
            BendDirection::Down
        } else {
            BendDirection::Up
        };

        rows.push(CmsOrderRow {
            item_code,
            category,
            product_type,
            variation,
            width,
            secondary_width,
            height,
            thickness,
            angle_deg,
            direction,
        });
    }

    Ok(rows)
}

fn deduce_secondary_width_from_name(name: &str) -> Option<f64> {
    let parts: Vec<&str> = name.split(['-', '_', 'x', 'X']).collect();
    if parts.len() >= 2 {
        for win in parts.windows(2) {
            if let (Ok(_w1), Ok(w2)) = (win[0].parse::<f64>(), win[1].parse::<f64>()) {
                return Some(w2);
            }
        }
    }
    None
}

/// Executes headless batch processing: morphs 2D geometry, preserves punch slots, folds flanges,
/// and writes both `.dxf` flat patterns and `.step` 3D solid models to `output_dir`.
pub fn execute_cms_batch_processing(
    csv_content: &str,
    output_dir: &Path,
) -> Result<BatchProcessingReport, BatchError> {
    std::fs::create_dir_all(output_dir)?;

    let rows = parse_cms_order_csv(csv_content)?;
    let mut report = BatchProcessingReport {
        total_rows: rows.len(),
        successful_items: Vec::new(),
        failed_rows: Vec::new(),
    };

    for (idx, row) in rows.into_iter().enumerate() {
        match process_single_order_row(&row, output_dir) {
            Ok(result) => {
                report.successful_items.push(result);
            }
            Err(e) => {
                report.failed_rows.push((idx + 1, e.to_string()));
            }
        }
    }

    Ok(report)
}

/// Processes a single order row into manufacturing assets (.dxf + .step).
fn process_single_order_row(
    row: &CmsOrderRow,
    output_dir: &Path,
) -> Result<BatchItemResult, BatchError> {
    // 1. Synthesize baseline wire loop for the given product type
    let (mut outer_loop, mut slots) = create_baseline_blank(row.product_type);

    // 2. Classify topology and bind ports
    let mut meta = cms_identify_topology(&outer_loop, &[], 50.0, row.thickness)
        .map_err(|e| BatchError::Topology(format!("Failed to identify topology for '{}': {}", row.item_code, e)))?;
    meta.rigid_slots = slots.clone();

    // 3. Set dynamic target dimensions on ports
    if let Some(p1) = meta.port_mut(1) {
        p1.current_width = row.width;
        p1.current_height = row.height;
    }
    if let Some(sec_w) = row.secondary_width {
        if let Some(p2) = meta.port_mut(2) {
            p2.current_width = sec_w;
            p2.current_height = row.height;
        }
    }

    // 4. Morph 2D perimeter sketch
    morph_base_2d_sketch(&mut outer_loop, &meta);

    // 5. Migrate rigid coupler slots (lock scale 1.0)
    migrate_rigid_coupler_slots(&mut slots, &outer_loop);
    meta.rigid_slots = slots.clone();

    // 6. Generate sidewall bend lines tracking the scaled boundary
    let mut bends = generate_sidewall_bends(&outer_loop, row.height, row.angle_deg, row.direction);
    reposition_bend_lines(&mut bends, &outer_loop, &meta);
    meta.active_bends = bends.clone();

    // 7. Execute relaxed fold on primary bend line
    let fold_result = if let Some(primary_bend) = bends.first() {
        execute_relaxed_fold(primary_bend, &outer_loop, row.thickness)
    } else {
        // Fallback flat result if no bends
        crate::eval::RelaxedFoldResult {
            stationary_vertices: outer_loop.vertices.iter().map(|&v| [v.0, v.1, 0.0]).collect(),
            rotated_flange_vertices: Vec::new(),
            fillet_inner_vertices: Vec::new(),
            fillet_outer_vertices: Vec::new(),
            fillet_triangles: Vec::new(),
            fused_solid_triangles: Vec::new(),
        }
    };

    // 8. Export 2D Flat DXF (Layer-segregated CAM)
    let flat_dxf_filename = format!("{}_flat.dxf", row.item_code);
    let flat_dxf_path = output_dir.join(&flat_dxf_filename);
    export_2d_flat_dxf(&outer_loop, &slots, &bends, &flat_dxf_path)?;

    // 9. Export 3D STEP solid model
    let folded_step_filename = format!("{}_folded.step", row.item_code);
    let folded_step_path = output_dir.join(&folded_step_filename);
    export_3d_step(&fold_result, &row.item_code, &folded_step_path)?;

    Ok(BatchItemResult {
        item_code: row.item_code.clone(),
        flat_dxf_path,
        folded_step_path,
        triangle_count: fold_result.fused_solid_triangles.len(),
        slots_preserved: slots.len(),
    })
}

fn create_baseline_blank(product: CmsProductType) -> (StructuralWireLoop, Vec<CmsSlotMetadata>) {
    match product {
        CmsProductType::Reducer => {
            let outer = StructuralWireLoop::new(
                vec![0, 1, 2, 3],
                vec![
                    (0.0, -150.0),
                    (0.0, 150.0),
                    (200.0, 75.0),
                    (200.0, -75.0),
                ],
            );
            let slot = CmsSlotMetadata::new(
                1,
                vec![(15.0, 45.5), (40.0, 45.5), (40.0, 54.5), (15.0, 54.5)],
                (27.5, 50.0),
                0,
                (27.5, 50.0),
            );
            (outer, vec![slot])
        }
        CmsProductType::UnequalTee => {
            let outer = StructuralWireLoop::new(
                vec![0, 1, 2, 3, 4, 5],
                vec![
                    (-200.0, -150.0),
                    (-200.0, 150.0),
                    (-75.0, 250.0),
                    (75.0, 250.0),
                    (200.0, 150.0),
                    (200.0, -150.0),
                ],
            );
            (outer, Vec::new())
        }
        CmsProductType::StandardBendOrRiser => {
            let outer = StructuralWireLoop::new(
                vec![0, 1, 2, 3],
                vec![
                    (0.0, 0.0),
                    (0.0, 200.0),
                    (300.0, 300.0),
                    (200.0, 0.0),
                ],
            );
            (outer, Vec::new())
        }
        _ => {
            // Default 300 x 200 straight blank
            let outer = StructuralWireLoop::new(
                vec![0, 1, 2, 3],
                vec![
                    (0.0, -100.0),
                    (0.0, 100.0),
                    (300.0, 100.0),
                    (300.0, -100.0),
                ],
            );
            (outer, Vec::new())
        }
    }
}

fn generate_sidewall_bends(
    outer_loop: &StructuralWireLoop,
    flange_h: f64,
    angle_deg: f64,
    direction: BendDirection,
) -> Vec<CmsFoldParams> {
    let (min_x, min_y, max_x, max_y) = outer_loop.bounding_box();

    // Two parallel sidewall bend lines inward from the top and bottom perimeter limits
    let top_y = max_y - flange_h.min((max_y - min_y) * 0.4);
    let bot_y = min_y + flange_h.min((max_y - min_y) * 0.4);

    vec![
        CmsFoldParams::new(
            "bend_top",
            1,
            (min_x, top_y),
            (max_x, top_y),
            1.5,
            angle_deg,
            direction,
        ),
        CmsFoldParams::new(
            "bend_bot",
            2,
            (min_x, bot_y),
            (max_x, bot_y),
            1.5,
            angle_deg,
            direction,
        ),
    ]
}
