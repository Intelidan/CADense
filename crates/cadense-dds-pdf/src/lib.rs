//! `cadense-dds-pdf` — Danish Data Sheet (`.dds`) Archive & Vector PDF Technical Data Sheet Generator
//!
//! Provides:
//! - `.dds` project archive container packaging collections of `.dsf` files, customer order tables, and project metadata.
//! - 3D B-rep Hidden-Line Removal (HLR) isometric vector projection engine.
//! - Red dimension annotation system with extension lines, arrowheads, and angled text callouts.
//! - Sectional view insets (11mm tray return flange, 10mm/15mm cover lips, 41x41 Unistrut channel).
//! - Submittal title block & PDF generator supporting 1-Page-Per-Row and Grouped Submittal modes.

pub mod dds;
pub mod hlr;
pub mod dimensions;
pub mod insets;
pub mod pdf_writer;
pub mod generator;

pub use dds::{DdsArchive, DdsError, DdsManifest, DdsProjectMeta};
pub use hlr::{HlrEngine, IsoCamera, Line2D, LineKind, Mesh3D, Triangle3D, Vertex3D};
pub use dimensions::{DimensionAnnotation, DimensionStyle, DimensionSystem};
pub use insets::{SectionalInset, InsetKind};
pub use pdf_writer::PdfStreamWriter;
pub use generator::{BonnMetalSubmittalGenerator, PageMode, SubmittalPageSpec};

#[cfg(test)]
mod tests {
    use super::*;
    use cadense_order::OrderQueueEngine;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_generate_bonn_metal_submittal_pdf() {
        let mut queue = OrderQueueEngine::default();
        // Try local or workspace relative paths
        let csv_paths = [
            "../../orders/customer_orders/sample_bonn_order.csv",
            "orders/customer_orders/sample_bonn_order.csv",
        ];
        for p in csv_paths {
            if let Ok(content) = fs::read_to_string(p) {
                let _ = queue.import_csv_str(&content, "sample_bonn_order.csv");
                break;
            }
        }

        let mut meta = DdsProjectMeta::default();
        meta.customer_name = "BONN METAL CONS. IND. LLC".to_string();
        meta.project_name = "CMS CABLE TRAY & TRUNKING SUBMITTAL".to_string();
        meta.order_reference = "PO-2026-CMS-0042".to_string();

        let mut archive = DdsArchive::new(meta);
        archive.populate_from_order_queue(&queue);

        let pdf_bytes = BonnMetalSubmittalGenerator::generate_pdf(&archive, PageMode::OnePagePerRow);
        assert!(!pdf_bytes.is_empty());
        assert!(pdf_bytes.starts_with(b"%PDF-1.4"));

        let out_paths = [
            "../../orders/customer_orders/BONN_METAL_TDS_SUBMITTAL.pdf",
            "orders/customer_orders/BONN_METAL_TDS_SUBMITTAL.pdf",
        ];
        for p in out_paths {
            if let Some(parent) = Path::new(p).parent() {
                if parent.exists() {
                    let _ = fs::write(p, &pdf_bytes);
                }
            }
        }
    }
}