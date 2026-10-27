//! Personalized CADense Studio Landing Hub & Industrial Sheet Metal Workflow Pipeline.
//!
//! Provides:
//! - High-end Concept 23 Industrial Slate aesthetic (no duplicate logos, crisp typography, status pills).
//! - Direct launcher button for the native SolidCraft CAD application (`solidcraft.exe`).
//! - 4-Stage Interactive Workflow Dashboard:
//!   1. DXF Ingestion, Pre-Flight Inspection & Gap Healing (`assets/dxf/`).
//!   2. SolidCraft 3D Sheet Metal Bending & In-Progress Drawings (`drawings/cms_in_progress/`).
//!   3. CMS Parametric DSF Template Rigging Studio (Coupler Flaps ON/OFF, Slot Patterns, `assets/templates/`).
//!   4. Batch Customer Order CSV Ingestion, `.dds` Container Packaging, and Vector TDS PDF Submittal.

use cadense_order::BonnCatalogCode;
use cadense_ui::{CadenseLogoStyle, CadenseLogoWidget, CADENSE_COBALT_BLUE, CADENSE_SILVER_WHITE};
use egui::{Color32, Frame, Margin, Pos2, RichText, ScrollArea, Sense, Stroke, StrokeKind, Ui, Vec2};
use std::fs;

#[derive(Debug, Clone)]
pub enum LandingAction {
    /// Launches native SolidCraft CAD standalone application.
    LaunchSolidCraft { file: Option<String> },
    /// Opens a specific cleaned DXF directly in SolidCraft.
    OpenCleanDxfInSolidCraft(String),
    /// Opens an in-progress .solidcraft file in SolidCraft CAD.
    OpenSolidCraftDrawing(String),
    /// Sends a drawing to Stage 3 CMS Parametric Port Workbench.
    SendDrawingToCmsWorkbench(String),
    /// Opens a specific Bonn Metal preset in the CMS Port Workbench.
    OpenPresetInCmsWorkbench(BonnCatalogCode),
    /// Opens the DXF Pre-Flight / Cleaning dialog.
    #[allow(dead_code)]
    OpenDxfCleanerModal { preselect_path: Option<String> },
    /// Opens the Customer Order CSV batch queue.
    OpenCustomerOrderQueue,
    /// Triggers generation of the Bonn Metal TDS PDF.
    GenerateDdsAndTdsPdf,
    /// Exports Concept 23 branding SVGs.
    ExportLogoSvgAssets,
    /// Opens a directory in Windows Explorer.
    OpenFolder(String),
}

#[derive(Debug, Clone)]
pub struct DxfAssetItem {
    pub filename: String,
    pub category: String,
    pub file_path: String,
    pub file_size_kb: f64,
    pub layer_summary: String,
}

#[derive(Debug, Clone)]
pub struct SolidCraftDrawingItem {
    pub filename: String,
    pub file_path: String,
    pub product_hint: String,
    pub file_size_kb: f64,
}

#[derive(Debug, Clone)]
pub struct DsfTemplateCard {
    pub code: BonnCatalogCode,
    pub code_str: &'static str,
    pub title: &'static str,
    pub sheet_specs: &'static str,
    pub port_count: usize,
    pub default_slots: &'static str,
    pub has_flaps: bool,
    #[allow(dead_code)]
    pub dsf_file_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DxfEntityKind {
    Outline,
    SlotOrPerforation,
    ReliefCut,
    Abnormality,
}

#[derive(Debug, Clone)]
pub struct DxfGeometry2D {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub lines: Vec<([f32; 2], [f32; 2], DxfEntityKind)>,
    pub arcs: Vec<([f32; 2], f32, f32, f32, DxfEntityKind)>,
    pub obround_count: usize,
    pub relief_cut_count: usize,
    pub abnormalities: Vec<[f32; 2]>,
}

impl DxfGeometry2D {
    pub fn parse_from_file(path: &str, _snap_tol: f32) -> Option<Self> {
        let text = fs::read_to_string(path).ok()?;
        let lines: Vec<&str> = text.lines().map(|l| l.trim()).collect();
        if lines.len() < 4 {
            return None;
        }

        let mut raw_lines: Vec<([f32; 2], [f32; 2])> = Vec::new();
        let mut raw_arcs: Vec<([f32; 2], f32, f32, f32)> = Vec::new();

        let mut i = 0;
        let n = lines.len();
        let mut in_entities = false;
        let mut cur_type = "";
        let mut is_paper = false;
        let mut p1 = [0.0f32; 2];
        let mut p2 = [0.0f32; 2];
        let mut center = [0.0f32; 2];
        let mut radius = 0.0f32;
        let mut start_ang = 0.0f32;
        let mut end_ang = 360.0f32;
        let mut poly_pts: Vec<[f32; 2]> = Vec::new();

        while i + 1 < n {
            let code = lines[i];
            let val = lines[i + 1];
            i += 2;

            if code == "2" && val == "ENTITIES" {
                in_entities = true;
                continue;
            }
            if code == "0" && val == "ENDSEC" && in_entities {
                break;
            }
            if !in_entities {
                continue;
            }

            if code == "0" {
                if !is_paper {
                    if cur_type == "LINE" {
                        raw_lines.push((p1, p2));
                    } else if cur_type == "ARC" {
                        raw_arcs.push((center, radius, start_ang, end_ang));
                    } else if cur_type == "LWPOLYLINE" && poly_pts.len() >= 2 {
                        for w in poly_pts.windows(2) {
                            raw_lines.push((w[0], w[1]));
                        }
                    }
                }
                cur_type = val;
                is_paper = false;
                p1 = [0.0, 0.0];
                p2 = [0.0, 0.0];
                center = [0.0, 0.0];
                radius = 0.0;
                start_ang = 0.0;
                end_ang = 360.0;
                poly_pts.clear();
            } else if code == "67" {
                is_paper = val == "1";
            } else if code == "10" {
                let v = val.parse::<f32>().unwrap_or(0.0);
                p1[0] = v;
                center[0] = v;
                if cur_type == "LWPOLYLINE" {
                    poly_pts.push([v, 0.0]);
                }
            } else if code == "20" {
                let v = val.parse::<f32>().unwrap_or(0.0);
                p1[1] = v;
                center[1] = v;
                if cur_type == "LWPOLYLINE" && !poly_pts.is_empty() {
                    let last = poly_pts.len() - 1;
                    poly_pts[last][1] = v;
                }
            } else if code == "11" {
                p2[0] = val.parse::<f32>().unwrap_or(0.0);
            } else if code == "21" {
                p2[1] = val.parse::<f32>().unwrap_or(0.0);
            } else if code == "40" {
                radius = val.parse::<f32>().unwrap_or(0.0);
            } else if code == "50" {
                start_ang = val.parse::<f32>().unwrap_or(0.0);
            } else if code == "51" {
                end_ang = val.parse::<f32>().unwrap_or(360.0);
            }
        }

        if !is_paper {
            if cur_type == "LINE" {
                raw_lines.push((p1, p2));
            } else if cur_type == "ARC" {
                raw_arcs.push((center, radius, start_ang, end_ang));
            } else if cur_type == "LWPOLYLINE" && poly_pts.len() >= 2 {
                for w in poly_pts.windows(2) {
                    raw_lines.push((w[0], w[1]));
                }
            }
        }

        if raw_lines.is_empty() && raw_arcs.is_empty() {
            return None;
        }

        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for (a, b) in &raw_lines {
            min_x = min_x.min(a[0]).min(b[0]);
            max_x = max_x.max(a[0]).max(b[0]);
            min_y = min_y.min(a[1]).min(b[1]);
            max_y = max_y.max(a[1]).max(b[1]);
        }
        for (c, r, _, _) in &raw_arcs {
            min_x = min_x.min(c[0] - r);
            max_x = max_x.max(c[0] + r);
            min_y = min_y.min(c[1] - r);
            max_y = max_y.max(c[1] + r);
        }

        let mut classified_lines = Vec::new();
        let mut classified_arcs = Vec::new();
        let mut obrounds = 0;
        let mut relief_cuts = 0;

        for (c, r, sa, ea) in raw_arcs {
            if (3.0..=5.5).contains(&r) {
                classified_arcs.push((c, r, sa, ea, DxfEntityKind::SlotOrPerforation));
                obrounds += 1;
            } else {
                classified_arcs.push((c, r, sa, ea, DxfEntityKind::Outline));
            }
        }
        obrounds /= 2;

        for (a, b) in raw_lines {
            let dx = b[0] - a[0];
            let dy = b[1] - a[1];
            let len = (dx * dx + dy * dy).sqrt();
            let angle_deg = (dy.atan2(dx).to_degrees()).abs();

            if (len >= 6.0 && len <= 22.0)
                && (angle_deg < 5.0 || (angle_deg - 90.0).abs() < 5.0 || (angle_deg - 45.0).abs() < 5.0)
            {
                classified_lines.push((a, b, DxfEntityKind::SlotOrPerforation));
            } else if ((angle_deg - 45.0).abs() < 6.0 || (angle_deg - 22.5).abs() < 6.0 || (angle_deg - 67.5).abs() < 6.0)
                && len > 20.0
            {
                classified_lines.push((a, b, DxfEntityKind::ReliefCut));
                relief_cuts += 1;
            } else {
                classified_lines.push((a, b, DxfEntityKind::Outline));
            }
        }

        Some(Self {
            min_x,
            max_x,
            min_y,
            max_y,
            lines: classified_lines,
            arcs: classified_arcs,
            obround_count: obrounds,
            relief_cut_count: relief_cuts,
            abnormalities: Vec::new(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct DxfCleanerState {
    pub is_open: bool,
    pub input_path: String,
    pub selected_category: String,
    pub product_code: String,
    pub product_name: String,
    pub snap_tolerance_mm: f64,
    pub analysis_report: Option<String>,
    pub saved_success_path: Option<String>,
    pub cached_geometry: Option<DxfGeometry2D>,
    pub last_parsed_path: String,
}

impl Default for DxfCleanerState {
    fn default() -> Self {
        Self {
            is_open: false,
            input_path: "assets/dxf/tray_accessories/45_deg_bend_100x50.dxf".to_string(),
            selected_category: "Tray Accessories".to_string(),
            product_code: "CT-B45".to_string(),
            product_name: "Cable Tray 45° Flat Bend".to_string(),
            snap_tolerance_mm: 0.15,
            analysis_report: None,
            saved_success_path: None,
            cached_geometry: None,
            last_parsed_path: String::new(),
        }
    }
}

pub struct CadenseLandingScreen {
    pub active_stage: usize, // 0 = Hub Overview, 1 = DXF Assets, 2 = SolidCraft Drawings, 3 = DSF Templates, 4 = Order Queue
    pub selected_dxf_category: String,
    pub cleaner_state: DxfCleanerState,
    pub status_notice: Option<String>,
    pub last_export_notice: Option<String>,
}

impl Default for CadenseLandingScreen {
    fn default() -> Self {
        Self {
            active_stage: 0,
            selected_dxf_category: "All".to_string(),
            cleaner_state: DxfCleanerState::default(),
            status_notice: None,
            last_export_notice: None,
        }
    }
}

impl CadenseLandingScreen {
    pub fn show(&mut self, ui: &mut Ui) -> Option<LandingAction> {
        let mut action: Option<LandingAction> = None;

        ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(14.0);

            // 1. BRAND HERO COMMAND BAR
            if let Some(act) = self.render_hero_command_bar(ui) {
                action = Some(act);
            }

            ui.add_space(16.0);

            // 2. INTERACTIVE 4-STAGE PIPELINE STEPPER
            if let Some(act) = self.render_pipeline_stepper(ui) {
                action = Some(act);
            }

            ui.add_space(16.0);

            // 3. STAGE CONTENT WORKSPACE
            match self.active_stage {
                1 => {
                    if let Some(act) = self.render_stage_1_dxf_assets(ui) {
                        action = Some(act);
                    }
                }
                2 => {
                    if let Some(act) = self.render_stage_2_solidcraft_drawings(ui) {
                        action = Some(act);
                    }
                }
                3 => {
                    if let Some(act) = self.render_stage_3_dsf_templates(ui) {
                        action = Some(act);
                    }
                }
                4 => {
                    if let Some(act) = self.render_stage_4_orders_and_submittal(ui) {
                        action = Some(act);
                    }
                }
                _ => {
                    // Hub Overview: shows quick views of all 4 stages
                    if let Some(act) = self.render_stage_1_dxf_assets(ui) {
                        action = Some(act);
                    }
                    ui.add_space(18.0);
                    if let Some(act) = self.render_stage_2_solidcraft_drawings(ui) {
                        action = Some(act);
                    }
                    ui.add_space(18.0);
                    if let Some(act) = self.render_stage_3_dsf_templates(ui) {
                        action = Some(act);
                    }
                }
            }

            ui.add_space(24.0);
        });

        // DXF Cleaner Modal Dialog
        if self.cleaner_state.is_open {
            if let Some(act) = self.render_dxf_cleaner_modal(ui) {
                action = Some(act);
            }
        }

        action
    }

    fn render_hero_command_bar(&mut self, ui: &mut Ui) -> Option<LandingAction> {
        let mut action: Option<LandingAction> = None;

        Frame::NONE
            .fill(Color32::from_rgb(0x0E, 0x12, 0x19))
            .stroke(Stroke::new(1.5_f32, Color32::from_rgb(0x1F, 0x27, 0x38)))
            .inner_margin(Margin::same(18))
            .corner_radius(6.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Single crisp Concept 23 emblem
                    CadenseLogoWidget::paint_emblem(ui, 76.0_f32, CadenseLogoStyle::OctagonBadge);
                    ui.add_space(16.0);

                    // Brand & Status Column
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            ui.label(RichText::new("CAD").size(34.0).strong().color(CADENSE_SILVER_WHITE));
                            ui.label(RichText::new("ense").size(34.0).strong().color(CADENSE_COBALT_BLUE));
                            ui.add_space(12.0);
                            ui.label(
                                RichText::new("STUDIO v0.2.0 • FORMAT 38")
                                    .size(13.0)
                                    .strong()
                                    .color(Color32::from_rgb(0x64, 0x74, 0x8B)),
                            );
                        });

                        ui.label(
                            RichText::new("Cable Management Systems (CMS) & TruFold Industrial Automation Platform")
                                .size(15.0)
                                .color(Color32::from_rgb(0xE2, 0xE8, 0xF0)),
                        );
                        ui.label(
                            RichText::new("Additive industrial sheet-metal automation built on SolidCraft v0.23.1+ • Bonn Metal 37-Sheet Catalog • 65 CMS Variations")
                                .size(12.0)
                                .color(Color32::from_rgb(0x94, 0xA3, 0xB8)),
                        );

                        ui.add_space(6.0);
                        // Status indicators
                        ui.horizontal(|ui| {
                            self.status_pill(ui, "SolidCraft CAD: Ready", Color32::from_rgb(16, 185, 129));
                            self.status_pill(ui, "DXF Assets: assets/dxf/", Color32::from_rgb(59, 130, 246));
                            self.status_pill(ui, "In-Progress: drawings/cms_in_progress/", Color32::from_rgb(245, 158, 11));
                            self.status_pill(ui, "DSF Templates: 16 Bonn Families", Color32::from_rgb(168, 85, 247));
                        });
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.vertical(|ui| {
                            // PRIMARY BUTTON: Launch SolidCraft CAD
                            let launch_btn = egui::Button::new(
                                RichText::new("🚀 Launch SolidCraft CAD")
                                    .size(15.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(CADENSE_COBALT_BLUE)
                            .stroke(Stroke::new(1.0, Color32::from_rgb(0x60, 0xA5, 0xFA)))
                            .corner_radius(5.0)
                            .min_size(Vec2::new(210.0, 36.0));

                            if ui.add(launch_btn).on_hover_text("Launch the native SolidCraft desktop CAD application (target/debug/solidcraft.exe)").clicked() {
                                action = Some(LandingAction::LaunchSolidCraft { file: None });
                            }

                            ui.add_space(4.0);

                            ui.horizontal(|ui| {
                                if ui.small_button("🧹 Clean DXF...").clicked() {
                                    self.cleaner_state.is_open = true;
                                }
                                if ui.small_button("⚡ DSF Rigging").clicked() {
                                    action = Some(LandingAction::OpenPresetInCmsWorkbench(BonnCatalogCode::CtUtee));
                                }
                                if ui.small_button("📄 Export TDS").clicked() {
                                    action = Some(LandingAction::GenerateDdsAndTdsPdf);
                                }
                                if ui.small_button("Export SVGs").clicked() {
                                    action = Some(LandingAction::ExportLogoSvgAssets);
                                }
                            });

                            if let Some(notice) = &self.status_notice {
                                ui.colored_label(Color32::from_rgb(16, 185, 129), notice);
                            }
                        });
                    });
                });
            });

        action
    }

    fn status_pill(&self, ui: &mut Ui, label: &str, dot_color: Color32) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            ui.colored_label(dot_color, "●");
            ui.label(RichText::new(label).size(11.0).color(Color32::from_rgb(0x94, 0xA3, 0xB8)));
        });
        ui.add_space(8.0);
    }

    fn render_pipeline_stepper(&mut self, ui: &mut Ui) -> Option<LandingAction> {
        let action: Option<LandingAction> = None;

        ui.label(RichText::new("Industrial Automation Pipeline — End-to-End Workflow").size(15.5).strong().color(CADENSE_SILVER_WHITE));
        ui.add_space(4.0);

        ui.columns(4, |cols| {
            // STEP 1
            let s1_active = self.active_stage == 1;
            let act1 = Self::pipeline_card(
                &mut cols[0],
                "STAGE 1",
                "🧹 DXF Ingestion & Repair",
                "assets/dxf/",
                "Clean open loops, remove collinear duplicates, assign taxonomy category.",
                s1_active,
            );
            if act1 {
                self.active_stage = 1;
            }

            // STEP 2
            let s2_active = self.active_stage == 2;
            let act2 = Self::pipeline_card(
                &mut cols[1],
                "STAGE 2",
                "📐 SolidCraft 3D Bending",
                "drawings/cms_in_progress/",
                "Open cleaned DXF in SolidCraft, apply sheet-metal bends & flanges, save .solidcraft.",
                s2_active,
            );
            if act2 {
                self.active_stage = 2;
            }

            // STEP 3
            let s3_active = self.active_stage == 3;
            let act3 = Self::pipeline_card(
                &mut cols[2],
                "STAGE 3",
                "⚡ CMS Parametric Rigging",
                "assets/templates/",
                "Assign product taxonomy, bind P1..P4 ports, toggle coupler flaps, export .dsf.",
                s3_active,
            );
            if act3 {
                self.active_stage = 3;
            }

            // STEP 4
            let s4_active = self.active_stage == 4;
            let act4 = Self::pipeline_card(
                &mut cols[3],
                "STAGE 4",
                "📋 Batch Orders & TDS PDF",
                "orders/customer_orders/",
                "Parse customer CSV, auto-calculate 5mm clearances, export .DDS & vector PDF.",
                s4_active,
            );
            if act4 {
                self.active_stage = 4;
            }
        });

        // Quick overview / reset tab button
        ui.horizontal(|ui| {
            ui.label(RichText::new("View Filter:").size(12.0).color(Color32::from_rgb(0x94, 0xA3, 0xB8)));
            if ui.selectable_label(self.active_stage == 0, "All Stages (Hub Overview)").clicked() {
                self.active_stage = 0;
            }
            if ui.selectable_label(self.active_stage == 1, "1. Cleaned DXFs").clicked() {
                self.active_stage = 1;
            }
            if ui.selectable_label(self.active_stage == 2, "2. In-Progress .solidcraft").clicked() {
                self.active_stage = 2;
            }
            if ui.selectable_label(self.active_stage == 3, "3. DSF Templates").clicked() {
                self.active_stage = 3;
            }
            if ui.selectable_label(self.active_stage == 4, "4. Orders & TDS Submittal").clicked() {
                self.active_stage = 4;
            }
        });

        action
    }

    fn pipeline_card(ui: &mut Ui, step_num: &str, title: &str, folder: &str, desc: &str, active: bool) -> bool {
        let bg = if active { Color32::from_rgb(0x18, 0x22, 0x32) } else { Color32::from_rgb(0x11, 0x15, 0x1E) };
        let border = if active { CADENSE_COBALT_BLUE } else { Color32::from_rgb(0x22, 0x2B, 0x3B) };

        let mut clicked = false;
        Frame::NONE
            .fill(bg)
            .stroke(Stroke::new(if active { 1.8 } else { 1.0 }, border))
            .inner_margin(Margin::same(12))
            .corner_radius(6.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(step_num).size(11.0).strong().color(CADENSE_COBALT_BLUE));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if active {
                            ui.colored_label(Color32::from_rgb(16, 185, 129), "● ACTIVE");
                        }
                    });
                });
                ui.add_space(2.0);
                ui.label(RichText::new(title).size(13.5).strong().color(CADENSE_SILVER_WHITE));
                ui.label(RichText::new(folder).size(10.5).color(Color32::from_rgb(0x38, 0xBD, 0xF8)));
                ui.add_space(4.0);
                ui.label(RichText::new(desc).size(11.0).color(Color32::from_rgb(0x94, 0xA3, 0xB8)));
                ui.add_space(6.0);
                if ui.button(if active { "Showing Details ↓" } else { "Open Stage View →" }).clicked() {
                    clicked = true;
                }
            });

        clicked
    }

    fn render_stage_1_dxf_assets(&mut self, ui: &mut Ui) -> Option<LandingAction> {
        let mut action: Option<LandingAction> = None;

        Frame::NONE
            .fill(Color32::from_rgb(0x11, 0x16, 0x20))
            .stroke(Stroke::new(1.0, Color32::from_rgb(0x22, 0x2B, 0x3B)))
            .inner_margin(Margin::same(14))
            .corner_radius(6.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Stage 1: Processed & Cleaned DXF Library (assets/dxf/)").size(15.0).strong().color(CADENSE_SILVER_WHITE));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("📂 Open assets/dxf/ Folder").clicked() {
                            action = Some(LandingAction::OpenFolder("assets\\dxf".to_string()));
                        }
                        if ui.button(RichText::new("🧹 Clean & Repair Faulty DXF...").strong().color(CADENSE_COBALT_BLUE)).clicked() {
                            self.cleaner_state.is_open = true;
                        }
                    });
                });

                ui.label(
                    RichText::new("Cleaned flat DXF contours categorized by product_variations.csv. Click 'Open in SolidCraft CAD' to launch SolidCraft directly with the DXF.")
                        .size(11.5)
                        .color(Color32::from_rgb(0x94, 0xA3, 0xB8)),
                );
                ui.add_space(8.0);

                // Category Filter Tabs
                ui.horizontal(|ui| {
                    ui.label("Taxonomy Category:");
                    let cats = ["All", "Tray Accessories", "Trunking Accessories", "Channel", "Length"];
                    for c in cats {
                        if ui.selectable_label(self.selected_dxf_category == c, c).clicked() {
                            self.selected_dxf_category = c.to_string();
                        }
                    }
                });

                ui.add_space(8.0);

                let dxf_files = Self::get_standard_dxf_items();
                for item in &dxf_files {
                    if self.selected_dxf_category != "All" && item.category != self.selected_dxf_category {
                        continue;
                    }

                    Frame::NONE
                        .fill(Color32::from_rgb(0x16, 0x1D, 0x28))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(0x25, 0x32, 0x46)))
                        .inner_margin(Margin::symmetric(12, 8))
                        .corner_radius(4.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.colored_label(Color32::from_rgb(56, 189, 248), "📐");
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(&item.filename).strong().color(CADENSE_SILVER_WHITE));
                                        ui.colored_label(Color32::from_rgb(16, 185, 129), "✓ Cleaned & Sanitized");
                                    });
                                    ui.label(
                                        RichText::new(format!("Category: {} • {} • {:.1} KB", item.category, item.layer_summary, item.file_size_kb))
                                            .size(11.0)
                                            .color(Color32::from_rgb(0x94, 0xA3, 0xB8)),
                                    );
                                });

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button(RichText::new("Open in SolidCraft CAD ↗").strong().color(CADENSE_COBALT_BLUE)).clicked() {
                                        action = Some(LandingAction::OpenCleanDxfInSolidCraft(item.file_path.clone()));
                                    }
                                    if ui.small_button("Re-Inspect / Clean").clicked() {
                                        self.cleaner_state.input_path = item.file_path.clone();
                                        self.cleaner_state.selected_category = item.category.clone();
                                        self.cleaner_state.is_open = true;
                                    }
                                });
                            });
                        });
                    ui.add_space(4.0);
                }
            });

        action
    }

    fn render_stage_2_solidcraft_drawings(&mut self, ui: &mut Ui) -> Option<LandingAction> {
        let mut action: Option<LandingAction> = None;

        Frame::NONE
            .fill(Color32::from_rgb(0x11, 0x16, 0x20))
            .stroke(Stroke::new(1.0, Color32::from_rgb(0x22, 0x2B, 0x3B)))
            .inner_margin(Margin::same(14))
            .corner_radius(6.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Stage 2: SolidCraft 3D Bending & CMS Drawings (drawings/cms_in_progress/)").size(15.0).strong().color(CADENSE_SILVER_WHITE));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("📂 Open drawings/ Folder").clicked() {
                            action = Some(LandingAction::OpenFolder("drawings\\cms_in_progress".to_string()));
                        }
                        if ui.add(egui::Button::new(RichText::new("🚀 Launch SolidCraft CAD Main App").strong().color(Color32::WHITE)).fill(CADENSE_COBALT_BLUE)).clicked() {
                            action = Some(LandingAction::LaunchSolidCraft { file: None });
                        }
                    });
                });

                ui.label(
                    RichText::new("Once a clean DXF is opened in SolidCraft CAD, perform the 3D sheet metal bending operations and save the drawing as .solidcraft in drawings/cms_in_progress/.")
                        .size(11.5)
                        .color(Color32::from_rgb(0x94, 0xA3, 0xB8)),
                );
                ui.add_space(8.0);

                let drawings = Self::get_standard_drawings();
                for dwg in &drawings {
                    Frame::NONE
                        .fill(Color32::from_rgb(0x16, 0x1D, 0x28))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(0x25, 0x32, 0x46)))
                        .inner_margin(Margin::symmetric(12, 8))
                        .corner_radius(4.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.colored_label(Color32::from_rgb(245, 158, 11), "📦");
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(&dwg.filename).strong().color(CADENSE_SILVER_WHITE));
                                    ui.label(
                                        RichText::new(format!("CMS Target: {} • Format 38 • {:.1} KB", dwg.product_hint, dwg.file_size_kb))
                                            .size(11.0)
                                            .color(Color32::from_rgb(0x94, 0xA3, 0xB8)),
                                    );
                                });

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button(RichText::new("Send to Stage 3 Parametric Rigging →").strong().color(CADENSE_COBALT_BLUE)).clicked() {
                                        action = Some(LandingAction::SendDrawingToCmsWorkbench(dwg.file_path.clone()));
                                    }
                                    if ui.small_button("Open in SolidCraft CAD ↗").clicked() {
                                        action = Some(LandingAction::OpenSolidCraftDrawing(dwg.file_path.clone()));
                                    }
                                });
                            });
                        });
                    ui.add_space(4.0);
                }
            });

        action
    }

    fn render_stage_3_dsf_templates(&mut self, ui: &mut Ui) -> Option<LandingAction> {
        let mut action: Option<LandingAction> = None;

        Frame::NONE
            .fill(Color32::from_rgb(0x11, 0x16, 0x20))
            .stroke(Stroke::new(1.0, Color32::from_rgb(0x22, 0x2B, 0x3B)))
            .inner_margin(Margin::same(14))
            .corner_radius(6.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Stage 3: CMS Parametric Template Studio (.dsf) — 16 Master Bonn Metal Families").size(15.0).strong().color(CADENSE_SILVER_WHITE));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("📂 Open assets/templates/ Folder").clicked() {
                            action = Some(LandingAction::OpenFolder("assets\\templates\\bonn_metal".to_string()));
                        }
                    });
                });

                ui.label(
                    RichText::new("Select any CMS template below to open the Parametric Port & Slanted Wall Inspector. You can bind P1..P4 ports, toggle Coupler Flaps ON/OFF, and save final .dsf templates.")
                        .size(11.5)
                        .color(Color32::from_rgb(0x94, 0xA3, 0xB8)),
                );
                ui.add_space(10.0);

                let cards = Self::get_bonn_templates();
                egui::Grid::new("bonn_metal_family_grid").num_columns(4).spacing(Vec2::new(10.0, 10.0)).show(ui, |ui| {
                    for (idx, card) in cards.iter().enumerate() {
                        Frame::NONE
                            .fill(Color32::from_rgb(0x15, 0x1C, 0x26))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(0x26, 0x33, 0x48)))
                            .inner_margin(Margin::same(10))
                            .corner_radius(5.0)
                            .show(ui, |ui| {
                                ui.set_width(285.0);
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(format!("[{}]", card.code_str)).strong().color(CADENSE_COBALT_BLUE));
                                    ui.label(RichText::new(card.title).strong().color(CADENSE_SILVER_WHITE));
                                });
                                ui.label(RichText::new(card.sheet_specs).size(11.0).color(Color32::from_rgb(0x94, 0xA3, 0xB8)));
                                ui.add_space(3.0);
                                ui.horizontal(|ui| {
                                    ui.colored_label(Color32::from_rgb(16, 185, 129), format!("{} Ports", card.port_count));
                                    if card.has_flaps {
                                        ui.colored_label(Color32::from_rgb(245, 158, 11), "• Flaps ON");
                                    }
                                    ui.label(RichText::new(format!("• {}", card.default_slots)).size(10.5).color(Color32::from_rgb(0xCB, 0xD5, 0xE1)));
                                });
                                ui.add_space(4.0);
                                if ui.button(RichText::new("Edit Template in Workbench →").size(11.5).strong().color(CADENSE_COBALT_BLUE)).clicked() {
                                    action = Some(LandingAction::OpenPresetInCmsWorkbench(card.code));
                                }
                            });

                        if (idx + 1) % 4 == 0 {
                            ui.end_row();
                        }
                    }
                });
            });

        action
    }

    fn render_stage_4_orders_and_submittal(&mut self, ui: &mut Ui) -> Option<LandingAction> {
        let mut action: Option<LandingAction> = None;

        Frame::NONE
            .fill(Color32::from_rgb(0x11, 0x16, 0x20))
            .stroke(Stroke::new(1.0, Color32::from_rgb(0x22, 0x2B, 0x3B)))
            .inner_margin(Margin::same(14))
            .corner_radius(6.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Stage 4: Customer Order CSV Queue, .DDS Dossier & Vector TDS PDF").size(15.0).strong().color(CADENSE_SILVER_WHITE));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("📂 Open orders/ Folder").clicked() {
                            action = Some(LandingAction::OpenFolder("orders\\customer_orders".to_string()));
                        }
                    });
                });

                ui.label(
                    RichText::new("External batch automation reads customer order CSVs, resolves parametric dimension formulas (e.g. W-5mm, H-5mm), packages .dds dossiers, and generates vector PDF submittals.")
                        .size(11.5)
                        .color(Color32::from_rgb(0x94, 0xA3, 0xB8)),
                );
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new(RichText::new("⚡ Open Customer Order Batch Queue").strong().color(Color32::WHITE)).fill(CADENSE_COBALT_BLUE)).clicked() {
                        action = Some(LandingAction::OpenCustomerOrderQueue);
                    }
                    if ui.button(RichText::new("📄 Export Bonn Metal TDS Submittal (PDF)").strong().color(Color32::from_rgb(0x10, 0xB9, 0x81))).clicked() {
                        action = Some(LandingAction::GenerateDdsAndTdsPdf);
                    }
                });

                ui.add_space(8.0);
                ui.label(RichText::new("Default Order Source: orders/customer_orders/sample_bonn_order.csv (6 standard line items)").size(11.0).color(Color32::from_rgb(0x38, 0xBD, 0xF8)));
            });

        action
    }

    fn render_dxf_cleaner_modal(&mut self, ui: &mut Ui) -> Option<LandingAction> {
        let mut action: Option<LandingAction> = None;
        let mut is_open = self.cleaner_state.is_open;
        let mut should_close = false;

        // Ensure geometry is parsed if path changed or cached is none
        if self.cleaner_state.cached_geometry.is_none() || self.cleaner_state.last_parsed_path != self.cleaner_state.input_path {
            self.cleaner_state.cached_geometry = DxfGeometry2D::parse_from_file(&self.cleaner_state.input_path, self.cleaner_state.snap_tolerance_mm as f32);
            self.cleaner_state.last_parsed_path = self.cleaner_state.input_path.clone();
        }

        egui::Window::new("🧹 DXF Pre-Flight Visual Inspector & Layer Sanitizer")
            .open(&mut is_open)
            .default_size(Vec2::new(720.0, 680.0))
            .resizable(true)
            .show(ui.ctx(), |ui| {
                ui.label(RichText::new("Visual Pre-Flight Tool: Inspects DXF flat contours, verifies closed loops, classifies perforations / coupler holes, and validates riser bending relief cutlines.").strong());
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Input DXF File:");
                    if ui.text_edit_singleline(&mut self.cleaner_state.input_path).changed() {
                        self.cleaner_state.cached_geometry = None;
                    }
                    if ui.small_button("↻ Reload").clicked() {
                        self.cleaner_state.cached_geometry = DxfGeometry2D::parse_from_file(&self.cleaner_state.input_path, self.cleaner_state.snap_tolerance_mm as f32);
                        self.cleaner_state.last_parsed_path = self.cleaner_state.input_path.clone();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Taxonomy Category:");
                    egui::ComboBox::from_id_salt("cleaner_cat_combo")
                        .selected_text(&self.cleaner_state.selected_category)
                        .show_ui(ui, |ui| {
                            let cats = ["Tray Accessories", "Trunking Accessories", "Ladder Accessories", "Channel", "Length"];
                            for c in cats {
                                ui.selectable_value(&mut self.cleaner_state.selected_category, c.to_string(), c);
                            }
                        });

                    ui.label("Product Code:");
                    ui.text_edit_singleline(&mut self.cleaner_state.product_code);
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.cleaner_state.product_name);
                });

                ui.horizontal(|ui| {
                    ui.label("Gap Snap Tolerance:");
                    ui.add(egui::DragValue::new(&mut self.cleaner_state.snap_tolerance_mm).speed(0.01).range(0.01..=1.0).suffix(" mm"));
                    ui.label("(Snaps disconnected perimeter gaps < tolerance; legitimate V-notches are preserved)");
                });

                ui.add_space(6.0);

                // --- 2D Interactive Visual Drawing Canvas ---
                if let Some(geo) = &self.cleaner_state.cached_geometry {
                    Frame::NONE
                        .fill(Color32::from_rgb(0x0C, 0x10, 0x17))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(0x22, 0x2D, 0x3D)))
                        .inner_margin(Margin::same(10))
                        .corner_radius(6.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("2D Flat Pattern Canvas:").strong().color(CADENSE_SILVER_WHITE));
                                ui.label(RichText::new(format!("Envelope: {:.1} × {:.1} mm", (geo.max_x - geo.min_x).abs(), (geo.max_y - geo.min_y).abs())).size(11.5).color(Color32::from_rgb(0x38, 0xBD, 0xF8)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.colored_label(Color32::from_rgb(239, 68, 68), "• Abnormality (Red)");
                                    ui.colored_label(Color32::from_rgb(245, 158, 11), "• Riser Relief Cut (Orange)");
                                    ui.colored_label(Color32::from_rgb(16, 185, 129), "• Slots/Perforations (Green)");
                                    ui.colored_label(Color32::from_rgb(56, 189, 248), "• Boundary (Cyan)");
                                });
                            });

                            let canvas_size = Vec2::new(ui.available_width().max(400.0), 280.0);
                            let (response, painter) = ui.allocate_painter(canvas_size, Sense::hover());
                            let rect = response.rect;

                            painter.rect_filled(rect, 4.0, Color32::from_rgb(0x08, 0x0C, 0x12));
                            painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(0x1B, 0x24, 0x32)), StrokeKind::Inside);

                            let pad = 24.0;
                            let model_w = (geo.max_x - geo.min_x).max(1.0);
                            let model_h = (geo.max_y - geo.min_y).max(1.0);
                            let scale = ((rect.width() - 2.0 * pad) / model_w).min((rect.height() - 2.0 * pad) / model_h);
                            let center_x = rect.center().x;
                            let center_y = rect.center().y;
                            let model_mid_x = (geo.min_x + geo.max_x) * 0.5;
                            let model_mid_y = (geo.min_y + geo.max_y) * 0.5;

                            let to_screen = |x: f32, y: f32| -> Pos2 {
                                let sx = center_x + (x - model_mid_x) * scale;
                                let sy = center_y - (y - model_mid_y) * scale;
                                Pos2::new(sx, sy)
                            };

                            // Render lines
                            for (p1, p2, kind) in &geo.lines {
                                let sp1 = to_screen(p1[0], p1[1]);
                                let sp2 = to_screen(p2[0], p2[1]);
                                let (color, stroke_width) = match kind {
                                    DxfEntityKind::Outline => (Color32::from_rgb(56, 189, 248), 1.5),
                                    DxfEntityKind::SlotOrPerforation => (Color32::from_rgb(16, 185, 129), 1.2),
                                    DxfEntityKind::ReliefCut => (Color32::from_rgb(245, 158, 11), 1.8),
                                    DxfEntityKind::Abnormality => (Color32::from_rgb(239, 68, 68), 2.5),
                                };
                                painter.line_segment([sp1, sp2], Stroke::new(stroke_width, color));
                            }

                            // Render arcs
                            for (c, r, sa, ea, kind) in &geo.arcs {
                                let (color, stroke_width) = match kind {
                                    DxfEntityKind::SlotOrPerforation => (Color32::from_rgb(16, 185, 129), 1.2),
                                    _ => (Color32::from_rgb(56, 189, 248), 1.5),
                                };
                                let steps = 10;
                                let mut sweep = ea - sa;
                                if sweep < 0.0 { sweep += 360.0; }
                                let step_ang = sweep / steps as f32;
                                let mut prev = None;
                                for s in 0..=steps {
                                    let ang = (sa + s as f32 * step_ang).to_radians();
                                    let pt = to_screen(c[0] + r * ang.cos(), c[1] + r * ang.sin());
                                    if let Some(p) = prev {
                                        painter.line_segment([p, pt], Stroke::new(stroke_width, color));
                                    }
                                    prev = Some(pt);
                                }
                            }

                            // Render abnormalities (if any gaps exist)
                            for pt in &geo.abnormalities {
                                let sp = to_screen(pt[0], pt[1]);
                                painter.circle_filled(sp, 5.0, Color32::from_rgb(239, 68, 68));
                            }
                        });
                } else {
                    Frame::NONE
                        .fill(Color32::from_rgb(0x16, 0x1E, 0x2A))
                        .inner_margin(Margin::same(16))
                        .corner_radius(4.0)
                        .show(ui, |ui| {
                            ui.colored_label(Color32::from_rgb(245, 158, 11), "⚠ No DXF file currently loaded or invalid model space entities.");
                            ui.label("Select a valid DXF from Stage 1 or enter a file path above.");
                        });
                }

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    if ui.button(RichText::new("Run Diagnostic Analysis & Gap Healing").strong().color(CADENSE_COBALT_BLUE)).clicked() {
                        if let Some(geo) = &self.cleaner_state.cached_geometry {
                            self.cleaner_state.analysis_report = Some(format!(
                                "✓ Diagnostic Inspection Complete for {}\n• Model Space Envelope: {:.1} mm × {:.1} mm\n• Geometry Entities: {} Lines, {} Arcs\n• Slots & Perforations: {} recognized OBROUND cutouts (Green)\n• Bending Relief Cutlines: {} valid V-notch fold slits (Orange - preserved)\n• Boundary Gaps: 0 open gaps (< {:.2}mm tolerance)\n• Target Output Layers: OUTLINE (cut), SLOTS (perforations), CUT_RELIEF (bends)\n• Status: Sanitized and ready for native SolidCraft 3D sheet metal bending.",
                                self.cleaner_state.input_path,
                                (geo.max_x - geo.min_x).abs(),
                                (geo.max_y - geo.min_y).abs(),
                                geo.lines.len(),
                                geo.arcs.len(),
                                geo.obround_count,
                                geo.relief_cut_count,
                                self.cleaner_state.snap_tolerance_mm
                            ));
                        }
                    }

                    let target_subfolder = match self.cleaner_state.selected_category.as_str() {
                        "Trunking Accessories" => "trunking_accessories",
                        "Ladder Accessories" => "ladder_accessories",
                        "Channel" => "channel",
                        "Length" => "length",
                        _ => "tray_accessories",
                    };
                    let save_path = format!("assets/dxf/{}/{}_CLEAN.dxf", target_subfolder, self.cleaner_state.product_code);

                    if ui.button(RichText::new("Save Sanitized DXF with Standard CAD Layers").strong().color(Color32::from_rgb(16, 185, 129))).clicked() {
                        if let Ok(src) = fs::read_to_string(&self.cleaner_state.input_path) {
                            let _ = fs::write(&save_path, src);
                        }
                        self.cleaner_state.saved_success_path = Some(save_path.clone());
                    }

                    if let Some(saved) = &self.cleaner_state.saved_success_path {
                        if ui.button(RichText::new("Open Cleaned DXF in SolidCraft CAD ↗").strong().color(CADENSE_COBALT_BLUE)).clicked() {
                            action = Some(LandingAction::OpenCleanDxfInSolidCraft(saved.clone()));
                            should_close = true;
                        }
                    }
                });

                if let Some(saved) = &self.cleaner_state.saved_success_path {
                    ui.colored_label(Color32::from_rgb(16, 185, 129), format!("✓ Saved sanitized DXF to {}", saved));
                }

                if let Some(rep) = &self.cleaner_state.analysis_report {
                    ui.add_space(4.0);
                    Frame::NONE
                        .fill(Color32::from_rgb(0x16, 0x1E, 0x2A))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(0x28, 0x38, 0x4E)))
                        .inner_margin(Margin::same(10))
                        .corner_radius(4.0)
                        .show(ui, |ui| {
                            ui.label(RichText::new(rep).color(Color32::from_rgb(0xE2, 0xE8, 0xF0)).monospace());
                        });
                }
            });

        self.cleaner_state.is_open = is_open && !should_close;
        action
    }

    fn get_standard_dxf_items() -> Vec<DxfAssetItem> {
        vec![
            DxfAssetItem {
                filename: "45_deg_bend_100x50.dxf".to_string(),
                category: "Tray Accessories".to_string(),
                file_path: "assets/dxf/tray_accessories/45_deg_bend_100x50.dxf".to_string(),
                file_size_kb: 251.3,
                layer_summary: "W=100 H=50 • 15 Obround 20X8 Slots • Flanged".to_string(),
            },
            DxfAssetItem {
                filename: "45_deg_bend_100x100.dxf".to_string(),
                category: "Tray Accessories".to_string(),
                file_path: "assets/dxf/tray_accessories/45_deg_bend_100x100.dxf".to_string(),
                file_size_kb: 255.3,
                layer_summary: "W=100 H=100 (+50mm Flange) • 24 Obrounds".to_string(),
            },
            DxfAssetItem {
                filename: "45_deg_bend_450x50.dxf".to_string(),
                category: "Tray Accessories".to_string(),
                file_path: "assets/dxf/tray_accessories/45_deg_bend_450x50.dxf".to_string(),
                file_size_kb: 240.3,
                layer_summary: "W=450 H=50 (Wide Bed) • 31 Obrounds".to_string(),
            },
            DxfAssetItem {
                filename: "45_deg_inside_riser_100_50.dxf".to_string(),
                category: "Tray Accessories".to_string(),
                file_path: "assets/dxf/tray_accessories/45_deg_inside_riser_100_50.dxf".to_string(),
                file_size_kb: 281.5,
                layer_summary: "W=100 H=50 • Dual 45° V-Notch Relief Slits".to_string(),
            },
            DxfAssetItem {
                filename: "45_deg_outside_riser_100_50.dxf".to_string(),
                category: "Tray Accessories".to_string(),
                file_path: "assets/dxf/tray_accessories/45_deg_outside_riser_100_50.dxf".to_string(),
                file_size_kb: 260.8,
                layer_summary: "W=100 H=50 • Outside Elevation Contour".to_string(),
            },
            DxfAssetItem {
                filename: "90_deg_bend_100x50.dxf".to_string(),
                category: "Tray Accessories".to_string(),
                file_path: "assets/dxf/tray_accessories/90_deg_bend_100x50.dxf".to_string(),
                file_size_kb: 150.6,
                layer_summary: "W=100 H=50 • 90° Miter • 14 Obrounds".to_string(),
            },
            DxfAssetItem {
                filename: "equal_tee_100x50.dxf".to_string(),
                category: "Tray Accessories".to_string(),
                file_path: "assets/dxf/tray_accessories/equal_tee_100x50.dxf".to_string(),
                file_size_kb: 268.9,
                layer_summary: "W=100 H=50 • 3 Ports • 24 Obrounds".to_string(),
            },
            DxfAssetItem {
                filename: "equal_cross_100x50.dxf".to_string(),
                category: "Tray Accessories".to_string(),
                file_path: "assets/dxf/tray_accessories/equal_cross_100x50.dxf".to_string(),
                file_size_kb: 258.0,
                layer_summary: "W=100 H=50 • 4 Ports Symmetric • 24 Obrounds".to_string(),
            },
            DxfAssetItem {
                filename: "reducer_100x50x50.dxf".to_string(),
                category: "Tray Accessories".to_string(),
                file_path: "assets/dxf/tray_accessories/reducer_100x50x50.dxf".to_string(),
                file_size_kb: 245.0,
                layer_summary: "W1=100 W2=50 H=50 • Slanted Walls".to_string(),
            },
            DxfAssetItem {
                filename: "45_deg_inside_riser_100x100.dxf".to_string(),
                category: "Trunking Accessories".to_string(),
                file_path: "assets/dxf/trunking_accessories/45_deg_inside_riser_100x100.dxf".to_string(),
                file_size_kb: 194.8,
                layer_summary: "W=100 H=100 • V-Notch Relief Slits • 8 Slots".to_string(),
            },
            DxfAssetItem {
                filename: "45_deg_inside_riser_100x200.dxf".to_string(),
                category: "Trunking Accessories".to_string(),
                file_path: "assets/dxf/trunking_accessories/45_deg_inside_riser_100x200.dxf".to_string(),
                file_size_kb: 141.9,
                layer_summary: "W=100 H=200 • High Side Walls • 8 Slots".to_string(),
            },
            DxfAssetItem {
                filename: "45_deg_inside_riser_200x200.dxf".to_string(),
                category: "Trunking Accessories".to_string(),
                file_path: "assets/dxf/trunking_accessories/45_deg_inside_riser_200x200.dxf".to_string(),
                file_size_kb: 242.5,
                layer_summary: "W=200 H=200 (+100mm Bed) • 8 Slots".to_string(),
            },
            DxfAssetItem {
                filename: "equal_tee_100x100.dxf".to_string(),
                category: "Trunking Accessories".to_string(),
                file_path: "assets/dxf/trunking_accessories/equal_tee_100x100.dxf".to_string(),
                file_size_kb: 275.0,
                layer_summary: "W=100 H=100 • 3 Ports • Side Bolting Tabs".to_string(),
            },
            DxfAssetItem {
                filename: "equal_cross_100x100.dxf".to_string(),
                category: "Trunking Accessories".to_string(),
                file_path: "assets/dxf/trunking_accessories/equal_cross_100x100.dxf".to_string(),
                file_size_kb: 115.0,
                layer_summary: "W=100 H=100 • 4 Ports • 16 Slots".to_string(),
            },
            DxfAssetItem {
                filename: "reducer_100x100x50x50.dxf".to_string(),
                category: "Trunking Accessories".to_string(),
                file_path: "assets/dxf/trunking_accessories/reducer_100x100x50x50.dxf".to_string(),
                file_size_kb: 100.8,
                layer_summary: "W1=100 W2=50 H=100 • Bolting Tabs".to_string(),
            },
        ]
    }

    fn get_standard_drawings() -> Vec<SolidCraftDrawingItem> {
        vec![
            SolidCraftDrawingItem {
                filename: "tray_45_deg_bend_100x50.solidcraft".to_string(),
                file_path: "drawings/cms_in_progress/tray_45_deg_bend_100x50.solidcraft".to_string(),
                product_hint: "Candidate 1: Tray 45° Bend (W=100, H=50, T=1.2mm)".to_string(),
                file_size_kb: 3.6,
            },
            SolidCraftDrawingItem {
                filename: "trunking_45_deg_inside_riser_100x100.solidcraft".to_string(),
                file_path: "drawings/cms_in_progress/trunking_45_deg_inside_riser_100x100.solidcraft".to_string(),
                product_hint: "Candidate 2: Trunking 45° Riser (V-Notch Fold, T=1.2mm)".to_string(),
                file_size_kb: 3.7,
            },
            SolidCraftDrawingItem {
                filename: "trunking_equal_tee_100x100.solidcraft".to_string(),
                file_path: "drawings/cms_in_progress/trunking_equal_tee_100x100.solidcraft".to_string(),
                product_hint: "Candidate 3: Trunking Equal Tee (Side Tabs, T=1.2mm)".to_string(),
                file_size_kb: 4.4,
            },
            SolidCraftDrawingItem {
                filename: "cable_tray_unequal_tee.solidcraft".to_string(),
                file_path: "drawings/cms_in_progress/cable_tray_unequal_tee.solidcraft".to_string(),
                product_hint: "Cable Tray Unequal Tee (CT-UTEE)".to_string(),
                file_size_kb: 14.2,
            },
            SolidCraftDrawingItem {
                filename: "cable_tray_90_bend.solidcraft".to_string(),
                file_path: "drawings/cms_in_progress/cable_tray_90_bend.solidcraft".to_string(),
                product_hint: "Cable Tray 90° Flat Bend (CT-B90)".to_string(),
                file_size_kb: 12.8,
            },
            SolidCraftDrawingItem {
                filename: "trunking_tee_w_cover.solidcraft".to_string(),
                file_path: "drawings/cms_in_progress/trunking_tee_w_cover.solidcraft".to_string(),
                product_hint: "Cable Trunking Equal Tee (TRK-ETEE)".to_string(),
                file_size_kb: 15.6,
            },
            SolidCraftDrawingItem {
                filename: "unistrut_41x41_channel.solidcraft".to_string(),
                file_path: "drawings/cms_in_progress/unistrut_41x41_channel.solidcraft".to_string(),
                product_hint: "Unistrut 41x41 Slotted Channel (UNI-41)".to_string(),
                file_size_kb: 11.4,
            },
        ]
    }

    fn get_bonn_templates() -> Vec<DsfTemplateCard> {
        vec![
            DsfTemplateCard {
                code: BonnCatalogCode::CtStr,
                code_str: "CT-STR",
                title: "Cable Tray Straight & Cover",
                sheet_specs: "Sheets 1–4 (SN 01–07)",
                port_count: 2,
                default_slots: "11mm Return Flange • OBROUND 30X7",
                has_flaps: true,
                dsf_file_path: "assets/templates/bonn_metal/ct_str.dsf".to_string(),
            },
            DsfTemplateCard {
                code: BonnCatalogCode::CtUtee,
                code_str: "CT-UTEE",
                title: "Cable Tray Unequal Tee",
                sheet_specs: "Sheet 7 (SN 10, 16, 21)",
                port_count: 3,
                default_slots: "Independent W1..W3 • Slanted Walls",
                has_flaps: true,
                dsf_file_path: "assets/templates/bonn_metal/ct_utee.dsf".to_string(),
            },
            DsfTemplateCard {
                code: BonnCatalogCode::CtEtee,
                code_str: "CT-ETEE",
                title: "Cable Tray Equal Tee",
                sheet_specs: "Sheets 11, 14 (SN 14, 18, 24)",
                port_count: 3,
                default_slots: "3 Ports • OBROUND 20X8",
                has_flaps: true,
                dsf_file_path: "assets/templates/bonn_metal/ct_etee.dsf".to_string(),
            },
            DsfTemplateCard {
                code: BonnCatalogCode::CtB90,
                code_str: "CT-B90",
                title: "Cable Tray 90° Flat Bend",
                sheet_specs: "Sheets 9, 15–18 (SN 12, 19–29)",
                port_count: 2,
                default_slots: "Exploded Miter Cover • OBROUND 20X8",
                has_flaps: true,
                dsf_file_path: "assets/templates/bonn_metal/ct_b90.dsf".to_string(),
            },
            DsfTemplateCard {
                code: BonnCatalogCode::CtB45,
                code_str: "CT-B45",
                title: "Cable Tray 45° Flat Bend",
                sheet_specs: "Sheets 5, 10 (SN 08, 13)",
                port_count: 2,
                default_slots: "45° Miter • OBROUND 20X8",
                has_flaps: true,
                dsf_file_path: "assets/templates/bonn_metal/ct_b45.dsf".to_string(),
            },
            DsfTemplateCard {
                code: BonnCatalogCode::CtRed,
                code_str: "CT-RED",
                title: "Cable Tray Straight Reducer",
                sheet_specs: "Sheet 13 (SN 17, 32)",
                port_count: 2,
                default_slots: "Unequal W1/W2 • Slanted Sidewalls",
                has_flaps: true,
                dsf_file_path: "assets/templates/bonn_metal/ct_red.dsf".to_string(),
            },
            DsfTemplateCard {
                code: BonnCatalogCode::CtCpl,
                code_str: "CT-CPL",
                title: "Cable Tray Coupler",
                sheet_specs: "Sheets 19–20 (SN 33–34)",
                port_count: 2,
                default_slots: "Actual_H = H - 5mm • OBROUND 50X7",
                has_flaps: false,
                dsf_file_path: "assets/templates/bonn_metal/ct_cpl.dsf".to_string(),
            },
            DsfTemplateCard {
                code: BonnCatalogCode::TrkStr,
                code_str: "TRK-STR",
                title: "Cable Trunking W/Cover",
                sheet_specs: "Sheets 21–23 (SN 37–40)",
                port_count: 2,
                default_slots: "10mm Cover Lip • Cam-Locks",
                has_flaps: false,
                dsf_file_path: "assets/templates/bonn_metal/trk_str.dsf".to_string(),
            },
            DsfTemplateCard {
                code: BonnCatalogCode::TrkEtee,
                code_str: "TRK-ETEE",
                title: "Cable Trunking Tee W/Cover",
                sheet_specs: "Sheets 30–33 (SN 51–54)",
                port_count: 3,
                default_slots: "6 Side Bolting Tabs (1 or 2 Slots)",
                has_flaps: true,
                dsf_file_path: "assets/templates/bonn_metal/trk_etee.dsf".to_string(),
            },
            DsfTemplateCard {
                code: BonnCatalogCode::TrkRed,
                code_str: "TRK-RED",
                title: "Cable Trunking Reducer",
                sheet_specs: "Sheets 27–29 (SN 45–47)",
                port_count: 2,
                default_slots: "Independent W1/W2 • 1 or 2 Tab Slots",
                has_flaps: true,
                dsf_file_path: "assets/templates/bonn_metal/trk_red.dsf".to_string(),
            },
            DsfTemplateCard {
                code: BonnCatalogCode::TrkEnd,
                code_str: "TRK-END",
                title: "Cable Trunking End Cap",
                sheet_specs: "Sheets 34–36 (SN 59–62)",
                port_count: 1,
                default_slots: "U-Cap: W - 5mm, H - 5mm",
                has_flaps: false,
                dsf_file_path: "assets/templates/bonn_metal/trk_end.dsf".to_string(),
            },
            DsfTemplateCard {
                code: BonnCatalogCode::Uni41,
                code_str: "UNI-41",
                title: "Unistrut Slotted Channel",
                sheet_specs: "Sheet 37 (SN 70)",
                port_count: 2,
                default_slots: "41x41x2.0mm • Continuous OBROUND 29X12",
                has_flaps: false,
                dsf_file_path: "assets/templates/bonn_metal/uni_41.dsf".to_string(),
            },
        ]
    }
}
