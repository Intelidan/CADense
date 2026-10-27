//! `cadense-ui` — Phase 2 Port Assignment Inspector & Live 2D Unfolded Flat Pattern Window
//!
//! Designed to dock cleanly inside SolidCraft v0.23.1 (`crates/ui-egui`) while keeping
//! the main 3D WGPU viewport in its Folded 3D state.

use cadense_dsf::{
    bonn_metal_catalog, DsfMeta, PortDimensionOverrides, PortEqualityMode,
    SolidCraftCommand,
};
use egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlatLayer {
    Outline,
    Cut,
    BendUp,
    BendDown,
    Perforation,
}

impl FlatLayer {
    pub fn color(&self) -> Color32 {
        match self {
            FlatLayer::Outline => Color32::from_rgb(230, 238, 248),
            FlatLayer::Cut => Color32::from_rgb(255, 110, 110),
            FlatLayer::BendUp => Color32::from_rgb(80, 200, 120),
            FlatLayer::BendDown => Color32::from_rgb(255, 195, 70),
            FlatLayer::Perforation => Color32::from_rgb(120, 180, 255),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FlatSegment2D {
    pub start_mm: [f64; 2],
    pub end_mm: [f64; 2],
    pub layer: FlatLayer,
}

pub struct CadenseDsfStudio {
    pub catalog: Vec<DsfMeta>,
    pub selected_index: usize,
    pub active_meta: DsfMeta,
    pub overrides: PortDimensionOverrides,
    pub show_flat_pattern_window: bool,
    pub flat_pan_px: Vec2,
    pub flat_zoom: f32,
    pub flat_segments: Vec<FlatSegment2D>,
    pub pending_engine_commands: Vec<SolidCraftCommand>,
    pub status_banner: String,
}

impl Default for CadenseDsfStudio {
    fn default() -> Self {
        let catalog = bonn_metal_catalog();
        let active_meta = catalog[2].clone();
        let mut studio = Self {
            catalog,
            selected_index: 2,
            active_meta,
            overrides: PortDimensionOverrides::default(),
            show_flat_pattern_window: true,
            flat_pan_px: Vec2::ZERO,
            flat_zoom: 0.35,
            flat_segments: Vec::new(),
            pending_engine_commands: Vec::new(),
            status_banner: "Ready — 3D Folded + Live 2D Unfolded Flat Preview Active".to_string(),
        };
        studio.rebuild_preview_and_commands();
        studio
    }
}

impl CadenseDsfStudio {
    pub fn rebuild_preview_and_commands(&mut self) {
        self.pending_engine_commands = self.active_meta.build_solidcraft_commands(&self.overrides);
        let resolved = self.active_meta.resolve_parameters(&self.overrides);

        let w1 = resolved
            .get("W1")
            .or_else(|| resolved.get("W"))
            .copied()
            .unwrap_or(300.0);
        let w2 = resolved.get("W2").copied().unwrap_or(w1);
        let h = resolved
            .get("H")
            .or_else(|| resolved.get("H1"))
            .copied()
            .unwrap_or(50.0);
        let slot_l = resolved.get("Slot_L").copied().unwrap_or(20.0);
        let slot_w = resolved.get("Slot_W").copied().unwrap_or(8.0);

        let mut segs = Vec::new();
        let total_w = w1 + 2.0 * h;
        let total_l = (w2 + 200.0).max(300.0);

        let hw = total_w * 0.5;
        let hl = total_l * 0.5;
        segs.push(FlatSegment2D {
            start_mm: [-hw, -hl],
            end_mm: [hw, -hl],
            layer: FlatLayer::Outline,
        });
        segs.push(FlatSegment2D {
            start_mm: [hw, -hl],
            end_mm: [hw, hl],
            layer: FlatLayer::Outline,
        });
        segs.push(FlatSegment2D {
            start_mm: [hw, hl],
            end_mm: [-hw, hl],
            layer: FlatLayer::Outline,
        });
        segs.push(FlatSegment2D {
            start_mm: [-hw, hl],
            end_mm: [-hw, -hl],
            layer: FlatLayer::Outline,
        });

        let bx = w1 * 0.5;
        segs.push(FlatSegment2D {
            start_mm: [-bx, -hl],
            end_mm: [-bx, hl],
            layer: FlatLayer::BendUp,
        });
        segs.push(FlatSegment2D {
            start_mm: [bx, -hl],
            end_mm: [bx, hl],
            layer: FlatLayer::BendUp,
        });

        for side in [-1.0_f64, 1.0_f64] {
            let cx = side * (bx + h * 0.5);
            for idx in [-1.5_f64, -0.5, 0.5, 1.5] {
                let cy = idx * 50.0;
                let sl = slot_l * 0.5;
                let sw = slot_w * 0.5;
                segs.push(FlatSegment2D {
                    start_mm: [cx - sw, cy - sl],
                    end_mm: [cx + sw, cy - sl],
                    layer: FlatLayer::Cut,
                });
                segs.push(FlatSegment2D {
                    start_mm: [cx + sw, cy - sl],
                    end_mm: [cx + sw, cy + sl],
                    layer: FlatLayer::Cut,
                });
                segs.push(FlatSegment2D {
                    start_mm: [cx + sw, cy + sl],
                    end_mm: [cx - sw, cy + sl],
                    layer: FlatLayer::Cut,
                });
                segs.push(FlatSegment2D {
                    start_mm: [cx - sw, cy + sl],
                    end_mm: [cx - sw, cy - sl],
                    layer: FlatLayer::Cut,
                });
            }
        }

        self.flat_segments = segs;
        self.status_banner = format!(
            "Updated {} ({}) — {} SolidCraft engine commands queued",
            self.active_meta.product_code,
            self.active_meta.product_title,
            self.pending_engine_commands.len()
        );
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        egui::Panel::right("cadense_dsf_port_inspector")
            .resizable(true)
            .default_size(360.0)
            .show(ui, |ui| {
                self.render_inspector_content(ui);
            });

        if self.show_flat_pattern_window {
            self.render_flat_pattern_window(ui.ctx());
        }
    }

    pub fn ui_window(&mut self, ctx: &egui::Context) {
        let mut show_win = true;
        egui::Window::new("📐 CADense DSF Port Studio")
            .id(egui::Id::new("cadense_dsf_port_inspector_window"))
            .default_size([360.0, 560.0])
            .resizable(true)
            .open(&mut show_win)
            .show(ctx, |ui| {
                self.render_inspector_content(ui);
            });

        if self.show_flat_pattern_window {
            self.render_flat_pattern_window(ctx);
        }
    }

    pub fn render_inspector_content(&mut self, ui: &mut egui::Ui) {
        ui.heading("📐 CADense DSF Port Studio");
        ui.label("Phase 2 — Danish Sheet Fold (.dsf) & Port Assignment");
        ui.separator();

        let mut template_changed = false;
        egui::ComboBox::from_label("CMS Product Family")
            .selected_text(format!(
                "{} — {}",
                self.active_meta.product_code, self.active_meta.product_title
            ))
            .show_ui(ui, |ui| {
                for (idx, item) in self.catalog.iter().enumerate() {
                    if ui
                        .selectable_value(
                            &mut self.selected_index,
                            idx,
                            format!("{} — {}", item.product_code, item.product_title),
                        )
                        .clicked()
                    {
                        template_changed = true;
                    }
                }
            });

        if template_changed {
            self.active_meta = self.catalog[self.selected_index].clone();
            self.overrides = PortDimensionOverrides::default();
            self.rebuild_preview_and_commands();
        }

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label("Port Equality Mode:");
            let mut mode = self.active_meta.equality_mode;
            if ui
                .selectable_value(&mut mode, PortEqualityMode::AllEqual, "All Equal")
                .changed()
                || ui
                    .selectable_value(
                        &mut mode,
                        PortEqualityMode::SymmetricOpposite,
                        "Symm Tee",
                    )
                    .changed()
                || ui
                    .selectable_value(&mut mode, PortEqualityMode::Unequal, "Unequal")
                    .changed()
            {
                self.active_meta.equality_mode = mode;
                self.rebuild_preview_and_commands();
            }
        });

        ui.separator();
        ui.strong("🔌 Assigned Open Ports (Parametric Width & Height)");

        let resolved = self.active_meta.resolve_parameters(&self.overrides);
        let mut ports_edited = false;

        for port in &self.active_meta.ports {
            ui.group(|ui| {
                ui.label(format!(
                    "Port {} ({}) -> Faces: {}",
                    port.id,
                    port.label,
                    port.anchor_faces.join(", ")
                ));
                let mut w_val = resolved
                    .get(&port.width_param)
                    .copied()
                    .unwrap_or(port.default_width_mm);
                let mut h_val = resolved
                    .get(&port.height_param)
                    .copied()
                    .unwrap_or(port.default_height_mm);

                ui.horizontal(|ui| {
                    ui.label(format!("{} ({}):", port.label, port.width_param));
                    if ui
                        .add(
                            egui::DragValue::new(&mut w_val)
                                .speed(5.0)
                                .range(40.0..=1200.0)
                                .suffix(" mm"),
                        )
                        .changed()
                    {
                        self.overrides
                            .values_mm
                            .insert(port.width_param.clone(), w_val);
                        if self.active_meta.equality_mode == PortEqualityMode::AllEqual {
                            self.overrides.values_mm.insert("W".to_string(), w_val);
                        }
                        ports_edited = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label(format!("HEIGHT ({}):", port.height_param));
                    if ui
                        .add(
                            egui::DragValue::new(&mut h_val)
                                .speed(5.0)
                                .range(25.0..=300.0)
                                .suffix(" mm"),
                        )
                        .changed()
                    {
                        self.overrides
                            .values_mm
                            .insert(port.height_param.clone(), h_val);
                        if self.active_meta.equality_mode == PortEqualityMode::AllEqual {
                            self.overrides.values_mm.insert("H".to_string(), h_val);
                        }
                        ports_edited = true;
                    }
                });
            });
        }

        if ports_edited {
            self.rebuild_preview_and_commands();
        }

        ui.separator();
        ui.strong("🔒 Locked Rigid Features (Unscaled Holes/Slots)");
        for rf in &self.active_meta.rigid_features {
            let count = resolved
                .get(&rf.count_param_name)
                .copied()
                .unwrap_or(1.0) as u32;
            ui.label(format!(
                "• {}: Locked {:.0}x{:.0} mm | Active Count: {}",
                rf.callout_label, rf.locked_length_mm, rf.locked_width_mm, count
            ));
        }

        if !self.active_meta.derived_formulas.is_empty() {
            ui.separator();
            ui.strong("📏 Derived Clearance Formulas");
            for df in &self.active_meta.derived_formulas {
                let actual = resolved.get(&df.target_param).copied().unwrap_or(0.0);
                ui.label(format!(
                    "• {} = {} {:+.0} mm => {:.1} mm",
                    df.target_param, df.source_param, df.offset_mm, actual
                ));
            }
        }

        ui.separator();
        ui.checkbox(
            &mut self.show_flat_pattern_window,
            "Show Simultaneous 2D Unfolded Flat Pattern Window",
        );
        ui.small(&self.status_banner);
    }

    fn render_flat_pattern_window(&mut self, ctx: &egui::Context) {
        egui::Window::new(format!(
            "2D Unfolded Flat Pattern — {} ({})",
            self.active_meta.product_code,
            self.active_meta.default_finish.note_text()
        ))
        .default_size([520.0, 380.0])
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Layers:");
                ui.colored_label(FlatLayer::Outline.color(), "■ OUTLINE");
                ui.colored_label(FlatLayer::BendUp.color(), "■ BEND_UP");
                ui.colored_label(FlatLayer::Cut.color(), "■ LOCKED OBROUND SLOTS");
                if ui.button("Reset View").clicked() {
                    self.flat_pan_px = Vec2::ZERO;
                    self.flat_zoom = 0.35;
                }
            });
            ui.separator();

            let (response, painter) =
                ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
            if response.dragged() {
                self.flat_pan_px += response.drag_delta();
            }

            let rect: Rect = response.rect;
            painter.rect_filled(rect, 4.0, Color32::from_rgb(18, 22, 28));

            let center: Pos2 = rect.center() + self.flat_pan_px;
            let to_screen = |pt: [f64; 2]| -> Pos2 {
                Pos2::new(
                    center.x + (pt[0] as f32) * self.flat_zoom,
                    center.y - (pt[1] as f32) * self.flat_zoom,
                )
            };

            for seg in &self.flat_segments {
                let p1 = to_screen(seg.start_mm);
                let p2 = to_screen(seg.end_mm);
                let width = match seg.layer {
                    FlatLayer::Outline => 2.0_f32,
                    FlatLayer::BendUp | FlatLayer::BendDown => 1.5_f32,
                    FlatLayer::Cut | FlatLayer::Perforation => 1.5_f32,
                };
                painter.line_segment([p1, p2], Stroke::new(width, seg.layer.color()));
            }
        });
    }
    
}
pub mod logo;
pub mod order_queue_panel;
pub mod solidcraft_hook;
pub mod theme;
pub mod version;

pub use logo::{
    CadenseLogoStyle, CadenseLogoWidget, CADENSE_BADGE_BG, CADENSE_COBALT_BLUE,
    CADENSE_SILVER_WHITE, CONCEPT_23_BADGE_SVG, CONCEPT_23_HORIZONTAL_LOCKUP_SVG,
    CONCEPT_23_METALLIC_3D_SVG,
};
pub use order_queue_panel::{OrderQueuePanel, OrderQueuePanelState};
pub use solidcraft_hook::CadenseSolidCraftExtension;
pub use theme::{CadenseTheme, ThemeColors};
pub use version::{CadenseAppVersion, CURRENT_VERSION};


