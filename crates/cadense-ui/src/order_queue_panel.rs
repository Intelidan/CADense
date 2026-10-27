//! Interactive Order Queue Panel for `crates/cadense-ui`.
//! Allows importing customer order CSVs, inspecting Bonn Metal clearance deductions and
//! rigid slot locks, editing dimensions live, and loading items into the Dual Viewport.

use cadense_order::{CmsFinishType, OrderQueueAction, OrderQueueEngine, OrderQueueItemStatus};
use egui::{Color32, RichText, ScrollArea, Ui};
use std::path::PathBuf;
use uuid::Uuid;

pub struct OrderQueuePanelState {
    pub csv_path_input: String,
    pub export_dir_input: String,
    pub edit_w: f64,
    pub edit_h: f64,
    pub edit_t: f64,
    pub edit_l: f64,
    pub edit_has_l: bool,
    pub edit_finish: CmsFinishType,
    pub edit_with_cover: bool,
    pub synced_item_id: Option<Uuid>,
}

impl Default for OrderQueuePanelState {
    fn default() -> Self {
        Self {
            csv_path_input: "orders/customer_order.csv".to_string(),
            export_dir_input: "output/dsf_batch".to_string(),
            edit_w: 300.0,
            edit_h: 50.0,
            edit_t: 2.0,
            edit_l: 3000.0,
            edit_has_l: true,
            edit_finish: CmsFinishType::Hdg,
            edit_with_cover: false,
            synced_item_id: None,
        }
    }
}

pub struct OrderQueuePanel;

impl OrderQueuePanel {
    pub fn show(ui: &mut Ui, engine: &mut OrderQueueEngine, state: &mut OrderQueuePanelState) -> Option<OrderQueueAction> {
        let mut emitted_action: Option<OrderQueueAction> = None;

        ui.heading("Customer Order CSV Queue (Bonn Metal CMS)");
        ui.add_space(4.0);

        // Top CSV Import Bar
        ui.horizontal(|ui| {
            ui.label("Order CSV:");
            ui.text_edit_singleline(&mut state.csv_path_input);
            if ui.button("Load CSV").clicked() {
                let _ = engine.import_csv_file(&state.csv_path_input);
                state.synced_item_id = None;
            }
            if ui.button("Export All .DSF").clicked() {
                emitted_action = Some(OrderQueueAction::ExportAllDsf { target_dir: PathBuf::from(&state.export_dir_input) });
            }
        });

        let summary = engine.summary();
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!(
                    "Rows: {} | Resolved: {} | Modified: {} | Errors: {} | Total PCS: {}",
                    summary.total_rows, summary.resolved_count, summary.modified_count, summary.error_count, summary.total_piece_qty
                ))
                .strong(),
            );
        });

        ui.separator();

        // Filter Bar
        ui.horizontal(|ui| {
            ui.label("Search SN / Product:");
            ui.text_edit_singleline(&mut engine.filter.search_query);
            ui.checkbox(&mut engine.filter.only_with_cover, "W/Cover Only");
            ui.checkbox(&mut engine.filter.only_errors, "Errors Only");
        });

        ui.separator();

        // Order Queue Table
        let mut clicked_id: Option<Uuid> = None;
        ScrollArea::vertical().id_salt("cadense_order_queue_scroll").max_height(240.0).show(ui, |ui| {
            egui::Grid::new("cadense_order_queue_grid").striped(true).min_col_width(48.0).show(ui, |ui| {
                ui.strong("SN");
                ui.strong("Code");
                ui.strong("Product Name");
                ui.strong("Nominal (W x H)");
                ui.strong("Actual (W x H)");
                ui.strong("T (mm)");
                ui.strong("Slot Rule");
                ui.strong("Finish");
                ui.strong("QTY");
                ui.strong("Status");
                ui.end_row();

                for item in engine.filtered_items() {
                    let is_selected = engine.active_item_id == Some(item.item_id);
                    if ui.selectable_label(is_selected, &item.raw_row.sn).clicked() {
                        clicked_id = Some(item.item_id);
                    }

                    if let Some(spec) = &item.resolved {
                        ui.label(spec.catalog_code.as_str());
                        ui.label(&spec.canonical_variation_name);
                        ui.label(format!("{:.0} x {:.0}", spec.clearance.nominal_width_mm, spec.clearance.nominal_height_mm));
                        ui.label(format!("{:.0} x {:.0}", spec.clearance.actual_width_mm, spec.clearance.actual_height_mm));
                        ui.label(format!("{:.1}", spec.sheet_rule.thickness_mm));
                        ui.label(format!("{} ({}x)", spec.rigid_slot_rule.callout_label, spec.rigid_slot_rule.slots_per_feature));
                        ui.label(spec.sheet_rule.finish.code());
                    } else {
                        ui.label("-");
                        ui.label(&item.raw_row.product_name);
                        ui.label("-");
                        ui.label("-");
                        ui.label("-");
                        ui.label("-");
                        ui.label("-");
                    }

                    ui.label(format!("{}", item.raw_row.qty));
                    let status_color = match item.status {
                        OrderQueueItemStatus::Failed => Color32::from_rgb(220, 60, 60),
                        OrderQueueItemStatus::Modified => Color32::from_rgb(230, 160, 40),
                        OrderQueueItemStatus::ExportedDsf => Color32::from_rgb(50, 190, 90),
                        _ => Color32::from_rgb(80, 160, 240),
                    };
                    ui.colored_label(status_color, item.status.badge_label());
                    ui.end_row();
                }
            });
        });

        if let Some(id) = clicked_id {
            engine.select_item(id);
            emitted_action = Some(OrderQueueAction::OpenInWorkbench(id));
        }

        // Selected Item Clearance & Override Inspector
        if let Some(active_id) = engine.active_item_id {
            if let Some(active_item) = engine.items.iter().find(|i| i.item_id == active_id) {
                if state.synced_item_id != Some(active_id) {
                    if let Some(spec) = &active_item.resolved {
                        state.edit_w = spec.clearance.nominal_width_mm;
                        state.edit_h = spec.clearance.nominal_height_mm;
                        state.edit_t = spec.sheet_rule.thickness_mm;
                        state.edit_l = spec.length_mm.unwrap_or(3000.0);
                        state.edit_has_l = spec.length_mm.is_some();
                        state.edit_finish = spec.sheet_rule.finish;
                        state.edit_with_cover = spec.cover_spec.included;
                        state.synced_item_id = Some(active_id);
                    }
                }

                ui.separator();
                if let Some(spec) = &active_item.resolved {
                    ui.label(
                        RichText::new(format!("Active Item SN {} — {} ({})", spec.sn, spec.canonical_variation_name, spec.dsf_template_path))
                            .strong(),
                    );
                    ui.label(format!("Clearance Rule: {}", spec.clearance.rule_explanation));
                    ui.label(format!("Rigid Slot Lock: {} — {}", spec.rigid_slot_rule.callout_label, spec.rigid_slot_rule.arrangement));

                    ui.horizontal(|ui| {
                        ui.label("Nominal W:");
                        ui.add(egui::DragValue::new(&mut state.edit_w).speed(5.0));
                        ui.label("Nominal H:");
                        ui.add(egui::DragValue::new(&mut state.edit_h).speed(5.0));
                        ui.label("Thickness T:");
                        ui.add(egui::DragValue::new(&mut state.edit_t).speed(0.1));
                        ui.checkbox(&mut state.edit_with_cover, "W/Cover");
                        if ui.button("Apply & Re-Solve").clicked() {
                            emitted_action = Some(OrderQueueAction::UpdateItemDimensions {
                                item_id: active_id,
                                w: state.edit_w,
                                h: state.edit_h,
                                t: state.edit_t,
                                l: if state.edit_has_l { Some(state.edit_l) } else { None },
                                finish: state.edit_finish,
                                with_cover: state.edit_with_cover,
                            });
                        }
                        if ui.button("Open in Dual Viewport").clicked() {
                            emitted_action = Some(OrderQueueAction::OpenInWorkbench(active_id));
                        }
                    });
                } else if let Some(err) = &active_item.error_message {
                    ui.colored_label(Color32::RED, format!("Resolution Error: {}", err));
                }
            }
        }

        emitted_action
    }
}
