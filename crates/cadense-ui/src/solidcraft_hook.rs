//! Additive UI Hook connecting SolidCraft v0.23.1 (`crates/ui`) to the CADense CMS Extension.
//!
//! Renders:
//! 1. The `CMS / TruFold` Ribbon Toolbar right alongside `Sheet Metal` in SolidCraft's top bar.
//! 2. The **Product Taxonomy & `.dsf` Wrapper Dialog** (where a `.solidcraft` sheet-metal model
//!    created from an inserted DXF is assigned a Product Category, Product Name, Variation,
//!    and independent Port 1..4 `W_i x H_i` bindings, and saved as a `.dsf` container).
//! 3. Floating toggles for the **2D Unfolded Flat Pattern Preview** and **Customer Order Queue**.

use cadense_dsf::trapezoidal_scaler::{MultiPortParametricSolution, TrapezoidalMultiPortScaler};
use cadense_order::{CmsFinishType, OrderQueueAction, OrderQueueEngine, ProductVariationRegistry};
use egui::{Color32, Context, RichText, Ui, Window};
use std::collections::BTreeMap;

use crate::order_queue_panel::{OrderQueuePanel, OrderQueuePanelState};

pub struct CadenseSolidCraftExtension {
    pub registry: ProductVariationRegistry,
    pub selected_variation_idx: usize,
    pub finish: CmsFinishType,
    pub with_cover: bool,
    pub independent_mode: bool,
    pub port_dims: BTreeMap<usize, (f64, f64)>,
    pub transition_length_mm: f64,
    pub last_solution: Option<MultiPortParametricSolution>,
    pub show_port_inspector: bool,
    pub show_2d_flat_pattern: bool,
    pub show_order_queue: bool,
    pub dsf_export_path: String,
    pub status_banner: String,
    pub order_engine: OrderQueueEngine,
    pub order_panel_state: OrderQueuePanelState,
    pub coupler_flaps: bool,
    pub coupler_hole_pattern: String,
    pub coupler_slot_count: u32,
    pub coupler_hole_offset_mm: f64,
}

impl Default for CadenseSolidCraftExtension {
    fn default() -> Self {
        let registry = ProductVariationRegistry::default();
        let mut port_dims = BTreeMap::new();
        port_dims.insert(1, (300.0, 50.0));
        port_dims.insert(2, (300.0, 50.0));
        port_dims.insert(3, (300.0, 50.0));

        let last_solution = Some(TrapezoidalMultiPortScaler::solve(&port_dims, 150.0));

        Self {
            registry,
            selected_variation_idx: 29, // Default: CABLE TRAY UNEQUAL TEE
            finish: CmsFinishType::Hdg,
            with_cover: false,
            independent_mode: true,
            port_dims,
            transition_length_mm: 150.0,
            last_solution,
            show_port_inspector: true,
            show_2d_flat_pattern: true,
            show_order_queue: false,
            dsf_export_path: "assets/templates/bonn_metal/ct_utee.dsf".to_string(),
            status_banner: "Active .solidcraft model linked. Assign Product Name & Ports to save as .dsf.".to_string(),
            order_engine: OrderQueueEngine::default(),
            order_panel_state: OrderQueuePanelState::default(),
            coupler_flaps: true,
            coupler_hole_pattern: "OBROUND 20X8".to_string(),
            coupler_slot_count: 2,
            coupler_hole_offset_mm: 25.0,
        }
    }
}

impl CadenseSolidCraftExtension {
    /// Call this inside SolidCraft's top Ribbon bar when the `CMS / TruFold` tab is active.
    pub fn show_ribbon_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Group 1: Product Taxonomy Assignment (65 variations from product_variations.csv)
            ui.vertical(|ui| {
                ui.label(RichText::new("1. Assign CMS Product Name").strong());
                let entry_options: Vec<(usize, String)> = self
                    .registry
                    .entries()
                    .iter()
                    .enumerate()
                    .map(|(idx, entry)| {
                        (
                            idx,
                            format!(
                                "{:02} | [{}] {} ({} Ports)",
                                entry.variation_id,
                                entry.catalog_code.as_str(),
                                entry.variation_name,
                                entry.port_count
                            ),
                        )
                    })
                    .collect();

                let current_name = self
                    .registry
                    .entries()
                    .get(self.selected_variation_idx)
                    .map(|e| format!("[{}] {}", e.catalog_code.as_str(), e.variation_name))
                    .unwrap_or_else(|| "Select Product...".to_string());

                let mut changed_selection = false;
                egui::ComboBox::from_id_salt("cadense_ribbon_product_combo").selected_text(current_name).width(290.0).show_ui(ui, |ui| {
                    for (idx, label) in entry_options {
                        if ui.selectable_value(&mut self.selected_variation_idx, idx, label).changed() {
                            changed_selection = true;
                        }
                    }
                });
                if changed_selection {
                    self.sync_ports_to_variation();
                }
            });

            ui.separator();

            // Group 2: Port Inspector & 2D Unfolded Flat Pattern Toggles
            ui.vertical(|ui| {
                ui.label(RichText::new("2. Ports & Dual Viewport").strong());
                ui.horizontal(|ui| {
                    ui.toggle_value(&mut self.show_port_inspector, "Port & Slanted Wall Inspector");
                    ui.toggle_value(&mut self.show_2d_flat_pattern, "2D Unfolded Preview");
                });
            });

            ui.separator();

            // Group 3: Wrap .solidcraft -> .dsf Container
            ui.vertical(|ui| {
                ui.label(RichText::new("3. Package .DSF Template").strong());
                if ui.button("Save Active .solidcraft as .DSF").clicked() {
                    if let Some(entry) = self.registry.entries().get(self.selected_variation_idx) {
                        self.dsf_export_path = entry.catalog_code.dsf_template_path();
                        self.status_banner =
                            format!("Wrapped active .solidcraft + '{}' metadata into {}", entry.variation_name, self.dsf_export_path);
                    }
                }
            });

            ui.separator();

            // Group 4: Phase 3 Customer Order Queue
            ui.vertical(|ui| {
                ui.label(RichText::new("4. Batch Automation").strong());
                ui.toggle_value(&mut self.show_order_queue, "Customer Order CSV Queue");
            });
        });
    }

    /// Call this once per frame in SolidCraft's main `egui::Context` loop to render floating CMS windows.
    pub fn show_floating_windows(&mut self, ctx: &Context) {
        if self.show_port_inspector {
            self.render_port_inspector_window(ctx);
        }
        if self.show_order_queue {
            let mut open = self.show_order_queue;
            Window::new("CADense — Customer Order CSV Queue").open(&mut open).default_width(780.0).show(ctx, |ui| {
                if let Some(action) = OrderQueuePanel::show(ui, &mut self.order_engine, &mut self.order_panel_state) {
                    self.handle_queue_action(action);
                }
            });
            self.show_order_queue = open;
        }
    }

    fn render_port_inspector_window(&mut self, ctx: &Context) {
        let mut open = self.show_port_inspector;
        Window::new("CADense — Port Assignment & Slanted Wall Inspector").open(&mut open).default_width(460.0).show(ctx, |ui| {
            let entry = &self.registry.entries()[self.selected_variation_idx];
            ui.label(RichText::new(format!("Product: {} ({})", entry.variation_name, entry.catalog_code.as_str())).strong());
            ui.label(format!(
                "Category: {} | Ports: {} | Template: {}",
                entry.category.as_str(),
                entry.port_count,
                entry.catalog_code.dsf_template_path()
            ));

            ui.add_space(4.0);
            ui.checkbox(&mut self.independent_mode, "Independent Unequal Port Mode (W_i and H_i independent per port)");
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.coupler_flaps, "Coupler Flaps / Side Bolting Tabs (ON/OFF)");
                ui.separator();
                ui.label("Pattern:");
                egui::ComboBox::from_id_salt("coupler_hole_combo")
                    .selected_text(&self.coupler_hole_pattern)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.coupler_hole_pattern, "OBROUND 20X8".to_string(), "OBROUND 20X8 (Standard)");
                        ui.selectable_value(&mut self.coupler_hole_pattern, "OBROUND 30X7".to_string(), "OBROUND 30X7 (Coupler)");
                        ui.selectable_value(&mut self.coupler_hole_pattern, "OBROUND 50X7".to_string(), "OBROUND 50X7 (Heavy)");
                        ui.selectable_value(&mut self.coupler_hole_pattern, "OBROUND 29X12".to_string(), "OBROUND 29X12 (Unistrut)");
                        ui.selectable_value(&mut self.coupler_hole_pattern, "ROUND 10MM".to_string(), "ROUND 10MM (Fastener)");
                    });
            });
            if self.coupler_flaps {
                ui.horizontal(|ui| {
                    ui.label("Slot Offset from Face:");
                    ui.add(egui::DragValue::new(&mut self.coupler_hole_offset_mm).speed(1.0).range(10.0..=60.0).suffix(" mm"));
                    ui.label("Slots / Flange:");
                    ui.add(egui::DragValue::new(&mut self.coupler_slot_count).speed(1.0).range(1..=4));
                });
            }

            ui.separator();
            let mut changed = false;
            let port_ids: Vec<usize> = self.port_dims.keys().copied().collect();

            for pid in port_ids {
                if let Some((w, h)) = self.port_dims.get_mut(&pid) {
                    ui.horizontal(|ui| {
                        ui.strong(format!("Port {}:", pid));
                        ui.label(format!("W{} (mm):", pid));
                        if ui.add(egui::DragValue::new(w).speed(5.0).range(25.0..=1200.0)).changed() {
                            changed = true;
                        }
                        ui.label(format!("H{} (mm):", pid));
                        if ui.add(egui::DragValue::new(h).speed(5.0).range(25.0..=300.0)).changed() {
                            changed = true;
                        }
                        let slots = if *h >= 100.0 { 2 } else { 1 };
                        ui.colored_label(Color32::from_rgb(90, 200, 120), format!("Slots: {}x", slots));
                    });
                }
            }

            if changed {
                if !self.independent_mode {
                    if let Some(&(w1, h1)) = self.port_dims.get(&1) {
                        for (_, dim) in self.port_dims.iter_mut() {
                            *dim = (w1, h1);
                        }
                    }
                }
                self.last_solution = Some(TrapezoidalMultiPortScaler::solve(&self.port_dims, self.transition_length_mm));
            }

            if let Some(sol) = &self.last_solution {
                ui.separator();
                ui.label(RichText::new("Trapezoidal / Slanted Transition Walls:").strong());
                for wall in &sol.transition_walls {
                    ui.label(format!(
                        "• {} (P{} -> P{}): H {:.0}->{:.0}mm (Slope {:+.1}°), Plan Taper {:.1}°, Bend Len {:.1}mm",
                        wall.wall_id,
                        wall.from_port_id,
                        wall.to_port_id,
                        wall.start_height_mm,
                        wall.end_height_mm,
                        wall.top_edge_slope_angle_deg,
                        wall.plan_taper_angle_deg,
                        wall.bend_line_length_mm
                    ));
                }
            }

            ui.separator();
            ui.horizontal(|ui| {
                ui.label(".DSF Target:");
                ui.text_edit_singleline(&mut self.dsf_export_path);
            });
            ui.colored_label(Color32::from_rgb(120, 190, 255), &self.status_banner);
        });
        self.show_port_inspector = open;
    }

    fn sync_ports_to_variation(&mut self) {
        if let Some(entry) = self.registry.entries().get(self.selected_variation_idx) {
            self.port_dims.clear();
            for p in 1..=entry.port_count {
                self.port_dims.insert(p, (300.0, 50.0));
            }
            self.with_cover = entry.default_with_cover;
            self.finish = entry.default_finish;
            self.dsf_export_path = entry.catalog_code.dsf_template_path();
            self.last_solution = Some(TrapezoidalMultiPortScaler::solve(&self.port_dims, self.transition_length_mm));
        }
    }

    fn handle_queue_action(&mut self, action: OrderQueueAction) {
        match action {
            OrderQueueAction::OpenInWorkbench(item_id) => {
                if let Some(item) = self.order_engine.select_item(item_id) {
                    if let Some(spec) = &item.resolved {
                        self.port_dims = spec.actual_ports_mm.clone();
                        self.last_solution = Some(TrapezoidalMultiPortScaler::solve(&self.port_dims, self.transition_length_mm));
                        self.show_port_inspector = true;
                        self.status_banner = format!("Loaded Order SN {} ({}) into Workbench.", spec.sn, spec.canonical_variation_name);
                    }
                }
            }
            OrderQueueAction::UpdateItemDimensions { item_id, w, h, t, l, finish, with_cover } => {
                if let Ok(spec) = self.order_engine.update_item_parameters(item_id, w, h, t, l, finish, with_cover) {
                    self.port_dims = spec.actual_ports_mm.clone();
                    self.last_solution = Some(TrapezoidalMultiPortScaler::solve(&self.port_dims, self.transition_length_mm));
                }
            }
            _ => {}
        }
    }
}
