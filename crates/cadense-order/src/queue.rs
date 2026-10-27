//! Interactive Order Queue Engine (`OrderQueueEngine`).
//! Manages batch CSV ingestion, item inspection, interactive dimension/finish overrides,
//! and `.dsf` ("Danish Sheet Fold") container export.

use crate::csv_parser::{CustomerOrderCsvParser, OrderCsvDiagnostic, RawOrderRow};
use crate::resolver::{ResolvedOrderSpec, TaxClearanceResolver};
use crate::taxonomy::{CmsFinishType, ProductVariationRegistry};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderQueueItemStatus {
    Pending,
    Resolved,
    InReview,
    Modified,
    ExportedDsf,
    Failed,
}

impl OrderQueueItemStatus {
    pub fn badge_label(&self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::Resolved => "RESOLVED",
            Self::InReview => "IN REVIEW",
            Self::Modified => "MODIFIED",
            Self::ExportedDsf => "DSF SAVED",
            Self::Failed => "ERROR",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderQueueItem {
    pub item_id: Uuid,
    pub raw_row: RawOrderRow,
    pub resolved: Option<ResolvedOrderSpec>,
    pub status: OrderQueueItemStatus,
    pub error_message: Option<String>,
    pub exported_dsf_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrderQueueFilter {
    pub search_query: String,
    pub finish_filter: Option<CmsFinishType>,
    pub only_with_cover: bool,
    pub only_errors: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrderQueueSummary {
    pub total_rows: usize,
    pub resolved_count: usize,
    pub modified_count: usize,
    pub exported_count: usize,
    pub error_count: usize,
    pub total_piece_qty: u32,
}

/// UI-to-Engine actions emitted by the interactive Order Queue Panel.
#[derive(Debug, Clone)]
pub enum OrderQueueAction {
    /// Load the selected queue item into the 3D Folded + 2D Unfolded Dual Viewport.
    OpenInWorkbench(Uuid),
    /// Override nominal dimensions `(W, H, T, L, Finish, WithCover)` on an item and re-run the resolver.
    UpdateItemDimensions { item_id: Uuid, w: f64, h: f64, t: f64, l: Option<f64>, finish: CmsFinishType, with_cover: bool },
    /// Export the selected item as a `.dsf` container.
    ExportSingleDsf { item_id: Uuid, target_dir: PathBuf },
    /// Batch-export all valid queue items as `.dsf` files for Phase 4 `.dds` PDF generation.
    ExportAllDsf { target_dir: PathBuf },
}

pub struct OrderQueueEngine {
    pub registry: ProductVariationRegistry,
    pub items: Vec<OrderQueueItem>,
    pub active_item_id: Option<Uuid>,
    pub filter: OrderQueueFilter,
    pub diagnostics: Vec<OrderCsvDiagnostic>,
    pub source_csv_name: Option<String>,
}

impl Default for OrderQueueEngine {
    fn default() -> Self {
        Self::new(ProductVariationRegistry::default())
    }
}

impl OrderQueueEngine {
    pub fn new(registry: ProductVariationRegistry) -> Self {
        Self {
            registry,
            items: Vec::new(),
            active_item_id: None,
            filter: OrderQueueFilter::default(),
            diagnostics: Vec::new(),
            source_csv_name: None,
        }
    }

    /// Loads and resolves a customer order CSV file from disk.
    pub fn import_csv_file<P: AsRef<Path>>(&mut self, path: P) -> Result<OrderQueueSummary, String> {
        let path_ref = path.as_ref();
        let report = CustomerOrderCsvParser::parse_file(path_ref).map_err(|e| e.to_string())?;
        self.source_csv_name = path_ref.file_name().map(|n| n.to_string_lossy().to_string());
        Ok(self.populate_from_parse_report(report.rows, report.diagnostics))
    }

    /// Loads and resolves a customer order CSV string directly.
    pub fn import_csv_str(&mut self, csv_text: &str, source_label: impl Into<String>) -> Result<OrderQueueSummary, String> {
        let report = CustomerOrderCsvParser::parse_str(csv_text).map_err(|e| e.to_string())?;
        self.source_csv_name = Some(source_label.into());
        Ok(self.populate_from_parse_report(report.rows, report.diagnostics))
    }

    fn populate_from_parse_report(&mut self, rows: Vec<RawOrderRow>, diagnostics: Vec<OrderCsvDiagnostic>) -> OrderQueueSummary {
        self.items.clear();
        self.diagnostics = diagnostics;

        for row in rows {
            let item_id = Uuid::new_v4();
            match TaxClearanceResolver::resolve_row(&row, &self.registry) {
                Ok(spec) => {
                    self.items.push(OrderQueueItem {
                        item_id,
                        raw_row: row,
                        resolved: Some(spec),
                        status: OrderQueueItemStatus::Resolved,
                        error_message: None,
                        exported_dsf_path: None,
                    });
                }
                Err(err) => {
                    self.items.push(OrderQueueItem {
                        item_id,
                        raw_row: row,
                        resolved: None,
                        status: OrderQueueItemStatus::Failed,
                        error_message: Some(err.to_string()),
                        exported_dsf_path: None,
                    });
                }
            }
        }

        self.active_item_id = self.items.first().map(|i| i.item_id);
        self.summary()
    }

    /// Selects an item for inspection in the Dual Viewport workbench.
    pub fn select_item(&mut self, item_id: Uuid) -> Option<&OrderQueueItem> {
        self.active_item_id = Some(item_id);
        if let Some(item) = self.items.iter_mut().find(|i| i.item_id == item_id) {
            if item.status == OrderQueueItemStatus::Resolved {
                item.status = OrderQueueItemStatus::InReview;
            }
        }
        self.items.iter().find(|i| i.item_id == item_id)
    }

    /// Applies interactive user refinements from the Order Queue Inspector and re-evaluates rules.
    pub fn update_item_parameters(
        &mut self,
        item_id: Uuid,
        w: f64,
        h: f64,
        t: f64,
        l: Option<f64>,
        finish: CmsFinishType,
        with_cover: bool,
    ) -> Result<&ResolvedOrderSpec, String> {
        let item = self.items.iter_mut().find(|i| i.item_id == item_id).ok_or_else(|| "Order Queue item not found".to_string())?;

        item.raw_row.w = Some(w);
        item.raw_row.w1 = Some(w);
        item.raw_row.h = Some(h);
        item.raw_row.h1 = Some(h);
        item.raw_row.t = Some(t);
        item.raw_row.l = l;
        item.raw_row.finish = Some(finish.code().to_string());
        item.raw_row.explicit_with_cover = Some(with_cover);

        match TaxClearanceResolver::resolve_row(&item.raw_row, &self.registry) {
            Ok(new_spec) => {
                item.resolved = Some(new_spec);
                item.status = OrderQueueItemStatus::Modified;
                item.error_message = None;
                Ok(item.resolved.as_ref().unwrap())
            }
            Err(err) => {
                item.status = OrderQueueItemStatus::Failed;
                item.error_message = Some(err.to_string());
                Err(err.to_string())
            }
        }
    }

    /// Returns items matching the active search and finish filter.
    pub fn filtered_items(&self) -> Vec<&OrderQueueItem> {
        let query = self.filter.search_query.trim().to_uppercase();
        self.items
            .iter()
            .filter(|item| {
                if self.filter.only_errors && item.status != OrderQueueItemStatus::Failed {
                    return false;
                }
                if let Some(spec) = &item.resolved {
                    if self.filter.only_with_cover && !spec.cover_spec.included {
                        return false;
                    }
                    if let Some(fin) = self.filter.finish_filter {
                        if spec.sheet_rule.finish != fin {
                            return false;
                        }
                    }
                    if !query.is_empty() {
                        let matches_sn = item.raw_row.sn.to_uppercase().contains(&query);
                        let matches_name = item.raw_row.product_name.to_uppercase().contains(&query);
                        let matches_code = spec.catalog_code.as_str().contains(&query);
                        if !matches_sn && !matches_name && !matches_code {
                            return false;
                        }
                    }
                } else if !query.is_empty() {
                    let matches_sn = item.raw_row.sn.to_uppercase().contains(&query);
                    let matches_name = item.raw_row.product_name.to_uppercase().contains(&query);
                    if !matches_sn && !matches_name {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    /// Computes live queue statistics for the Order Queue header bar.
    pub fn summary(&self) -> OrderQueueSummary {
        let mut summary = OrderQueueSummary { total_rows: self.items.len(), ..Default::default() };
        for item in &self.items {
            summary.total_piece_qty += item.raw_row.qty;
            match item.status {
                OrderQueueItemStatus::Resolved | OrderQueueItemStatus::InReview => {
                    summary.resolved_count += 1;
                }
                OrderQueueItemStatus::Modified => {
                    summary.resolved_count += 1;
                    summary.modified_count += 1;
                }
                OrderQueueItemStatus::ExportedDsf => {
                    summary.resolved_count += 1;
                    summary.exported_count += 1;
                }
                OrderQueueItemStatus::Failed => {
                    summary.error_count += 1;
                }
                OrderQueueItemStatus::Pending => {}
            }
        }
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::taxonomy::BonnCatalogCode;

    const SAMPLE_BONN_ORDER_CSV: &str = "\
SN,Product Name,W,W1,W2,W3,H,T,L,QTY,Finish
01,CABLE TRAY STRAIGHT W/COVER,600,,,,75,2.0,3000,45,HDG
10,CABLE TRAY UNEQUAL TEE,,900,300,900,50,2.0,,12,HDG
33,CABLE TRAY STRAIGHT COUPLER,0,,,,75,2.0,180,200,HDG
34,CABLE TRAY STRAIGHT COUPLER,0,,,,50,2.0,180,200,HDG
41,CABLE TRUNKING COUPLER,50,,,,50,1.2,,100,GI
44,CABLE TRUNKING COUPLER,150,,,,150,1.2,,80,GI
51,CABLE TRUNKING EQUAL TEE W/COVER,75,,,,75,1.2,,15,GI
54,CABLE TRUNKING EQUAL TEE W/COVER,150,,,,150,1.2,,10,GI
59,CABLE TRUNKING END CAP,100,,,,100,1.2,,50,GI
70,UNISTRUT SLOTTED CHANNEL,41,,,,41,2.0,3000,250,GI
";

    #[test]
    fn test_bonn_metal_csv_batch_and_clearance_rules() {
        let mut engine = OrderQueueEngine::default();
        let summary = engine.import_csv_str(SAMPLE_BONN_ORDER_CSV, "bonn_submittal_order.csv").expect("CSV should parse cleanly");

        assert_eq!(summary.total_rows, 10);
        assert_eq!(summary.error_count, 0);
        assert_eq!(summary.resolved_count, 10);

        // Verify SN 01: CT-STR W/COVER -> 15mm exploded cover, 11mm return flange, OBROUND 30X7
        let sn01 = engine.items[0].resolved.as_ref().unwrap();
        assert_eq!(sn01.catalog_code, BonnCatalogCode::CtStr);
        assert!(sn01.cover_spec.included);
        assert_eq!(sn01.cover_spec.return_lip_height_mm, 15.0);
        assert_eq!(sn01.sheet_rule.return_flange_mm, Some(11.0));
        assert_eq!(sn01.rigid_slot_rule.callout_label, "OBROUND 30X7");

        // Verify SN 33 & SN 34: CT-CPL height clearance (H-5mm) and Obround 50X7 vs 30X7 switch
        let sn33 = engine.items[2].resolved.as_ref().unwrap();
        assert_eq!(sn33.clearance.actual_height_mm, 70.0);
        assert_eq!(sn33.rigid_slot_rule.callout_label, "OBROUND 50X7");

        let sn34 = engine.items[3].resolved.as_ref().unwrap();
        assert_eq!(sn34.clearance.actual_height_mm, 45.0);
        assert_eq!(sn34.rigid_slot_rule.callout_label, "OBROUND 30X7");

        // Verify SN 41 & SN 44: TRK-CPL height clearance (H-5mm), L=50 vs L=70, 2 slots vs 4 slots (2x2)
        let sn41 = engine.items[4].resolved.as_ref().unwrap();
        assert_eq!(sn41.clearance.actual_height_mm, 45.0);
        assert_eq!(sn41.length_mm, Some(50.0));
        assert_eq!(sn41.rigid_slot_rule.slots_per_feature, 2);

        let sn44 = engine.items[5].resolved.as_ref().unwrap();
        assert_eq!(sn44.clearance.actual_height_mm, 145.0);
        assert_eq!(sn44.length_mm, Some(70.0));
        assert_eq!(sn44.rigid_slot_rule.slots_per_feature, 4);

        // Verify SN 51 & SN 54: TRK-ETEE discrete bolting tab slot rule (1 slot at H=75 vs 2 slots at H=150)
        let sn51 = engine.items[6].resolved.as_ref().unwrap();
        assert_eq!(sn51.rigid_slot_rule.slots_per_feature, 1);
        let sn54 = engine.items[7].resolved.as_ref().unwrap();
        assert_eq!(sn54.rigid_slot_rule.slots_per_feature, 2);

        // Verify SN 59: TRK-END U-cap clearance (Actual_W = W - 5mm = 95, Actual_H = H - 5mm = 95)
        let sn59 = engine.items[8].resolved.as_ref().unwrap();
        assert_eq!(sn59.clearance.actual_width_mm, 95.0);
        assert_eq!(sn59.clearance.actual_height_mm, 95.0);

        // Verify SN 70: UNI-41 -> OBROUND 29X12
        let sn70 = engine.items[9].resolved.as_ref().unwrap();
        assert_eq!(sn70.catalog_code, BonnCatalogCode::Uni41);
        assert_eq!(sn70.rigid_slot_rule.callout_label, "OBROUND 29X12");
    }
}
