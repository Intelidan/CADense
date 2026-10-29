//! Root `eframe::App` shell for `cadense-studio`.
//! Compatible with SolidCraft workspace dependencies (egui / eframe 0.36).

use crate::landing::{CadenseLandingScreen, DxfGeometry2D, LandingAction};
use crate::solid_renderer;
use cadense_dsf::{BendDirection, CmsCategory, DsfRiggedSheet, SheetMaterialRule};
use cadense_order::BonnCatalogCode;
use cadense_ui::{
    CadenseLogoWidget, CadenseSolidCraftExtension, CadenseTheme, ThemeColors, CONCEPT_23_BADGE_SVG, CONCEPT_23_HORIZONTAL_LOCKUP_SVG,
    CONCEPT_23_METALLIC_3D_SVG, CURRENT_VERSION,
};
use eframe::egui::{self, Color32, Frame, Margin, Pos2, Rect, RichText, Stroke, StrokeKind, Vec2};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioWorkspaceTab {
    Dashboard,
    DxfCleaner,
    SolidCraftAuthoring,
    CmsTruFoldWorkbench,
    OrderBatchQueue,
}

impl StudioWorkspaceTab {
    pub fn title(&self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard Hub",
            Self::DxfCleaner => "1. DXF Pre-Flight",
            Self::SolidCraftAuthoring => "2. 3D CAD Bending",
            Self::CmsTruFoldWorkbench => "3. CMS TruFold Rigging",
            Self::OrderBatchQueue => "4. Orders & TDS PDF",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Dashboard => egui_phosphor::regular::HOUSE,
            Self::DxfCleaner => egui_phosphor::regular::BROOM,
            Self::SolidCraftAuthoring => egui_phosphor::regular::CUBE,
            Self::CmsTruFoldWorkbench => egui_phosphor::regular::LIGHTNING,
            Self::OrderBatchQueue => egui_phosphor::regular::CLIPBOARD_TEXT,
        }
    }
}

/// Camera view mode for the unified bending canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasCameraMode {
    TopView2D,
    Perspective3D,
}

/// Interactive tool modes for the unified bending canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiggingToolMode {
    Select,
    AddBendLine,
}

/// An open drawing document tab in CMS TruFold workbench.
#[derive(Clone)]
pub struct TruFoldDocumentTab {
    #[allow(dead_code)]
    pub id: String,
    pub title: String,
    pub file_path: Option<String>,
    pub rigged_sheet: DsfRiggedSheet,
    pub active_dxf_geometry: Option<DxfGeometry2D>,
    pub fold_slider_deg: f32,
    pub camera_mode: CanvasCameraMode,
    pub tool_mode: RiggingToolMode,
    pub pending_p1: Option<[f64; 2]>,
    pub pan: Vec2,
    pub zoom: f32,
    pub orbit_yaw: f32,
    pub orbit_pitch: f32,
    pub show_floor_blueprint: bool,
    pub undo_stack: Vec<DsfRiggedSheet>,
    pub redo_stack: Vec<DsfRiggedSheet>,
}

pub struct CadenseStudioApp {
    pub active_tab: StudioWorkspaceTab,
    pub landing: CadenseLandingScreen,
    pub cms_ext: CadenseSolidCraftExtension,
    pub active_solidcraft_file: String,
    pub solidcraft_status: Option<String>,
    pub theme: CadenseTheme,
    pub font_scale: f32,
    pub show_settings_modal: bool,
    pub prefs_selected_page: usize,
    pub temp_custom_asset_root: String,
    pub last_applied_theme_key: Option<(CadenseTheme, u32)>,

    // === Multi-Tab TruFold Document Container ===
    pub trufold_tabs: Vec<TruFoldDocumentTab>,
    pub active_trufold_tab_idx: usize,
    pub undo_stack: Vec<DsfRiggedSheet>,
    pub redo_stack: Vec<DsfRiggedSheet>,

    // === Industrial Press-Brake Rigging & Unified Bending Engine State ===
    pub rigged_sheet: DsfRiggedSheet,
    pub rigging_camera_mode: CanvasCameraMode,
    pub rigging_tool_mode: RiggingToolMode,
    pub rigging_fold_slider_deg: f32,
    pub rigging_show_floor_blueprint: bool,
    pub rigging_pending_p1: Option<[f64; 2]>,
    pub rigging_pan: Vec2,
    pub rigging_zoom: f32,
    pub rigging_orbit_yaw: f32,
    pub rigging_orbit_pitch: f32,
    pub active_dxf_geometry: Option<DxfGeometry2D>,

    // === Separate Floating 3D Viewport Window State ===
    pub show_3d_viewport_window: bool,
    pub floating_orbit_yaw: f32,
    pub floating_orbit_pitch: f32,
    pub floating_pan: Vec2,
    pub floating_zoom: f32,
}

impl CadenseStudioApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        ensure_starter_templates_and_folders();

        let landing = CadenseLandingScreen::default();
        let asset_root = landing.resolved_asset_root.clone();

        // 1. Primary Drawing: 90_bend_100x50.dxf
        let dxf_90_path = "assets/dxf/tray_accessories/90_bend_100x50.dxf";
        let geom_90 = DxfGeometry2D::parse_from_file(dxf_90_path, 0.5)
            .or_else(|| DxfGeometry2D::parse_from_file("assets/dxf/trunking_accessories/45_deg_inside_riser_100x100.dxf", 0.5));
        let mut sheet_90 =
            DsfRiggedSheet::new(CmsCategory::CableTrunking, "BONN-90-DEG-BEND-100X50", "90 deg bend 100x50", SheetMaterialRule::bonn_metal_1_5mm());
        if let Some(ref g) = geom_90 {
            let (outer, holes) = g.to_dxf_loops();
            sheet_90.set_dxf_loops(outer, holes);
        }
        sheet_90.add_bend([-47.78, 41.86], [47.78, 41.86], 90.0, BendDirection::Up);
        sheet_90.add_bend([-48.47, -56.80], [47.53, -56.80], 90.0, BendDirection::Up);

        let tab_90 = TruFoldDocumentTab {
            id: "tab_1".to_string(),
            title: "90_bend_100x50.dxf".to_string(),
            file_path: Some(dxf_90_path.to_string()),
            rigged_sheet: sheet_90.clone(),
            active_dxf_geometry: geom_90.clone(),
            fold_slider_deg: 0.0,
            camera_mode: CanvasCameraMode::TopView2D,
            tool_mode: RiggingToolMode::Select,
            pending_p1: None,
            pan: Vec2::ZERO,
            zoom: 0.85,
            orbit_yaw: 0.60,
            orbit_pitch: 0.55,
            show_floor_blueprint: true,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        };

        // 2. Secondary Drawing: 45_deg_inside_riser_100x100.dxf
        let dxf_45_path = "assets/dxf/trunking_accessories/45_deg_inside_riser_100x100.dxf";
        let geom_45 = DxfGeometry2D::parse_from_file(dxf_45_path, 0.5);
        let mut sheet_45 = DsfRiggedSheet::new(
            CmsCategory::CableTrunking,
            "BONN-TRK-45-100X100",
            "Trunking 45° Inside Riser 100x100",
            SheetMaterialRule::bonn_metal_1_5mm(),
        );
        if let Some(ref g) = geom_45 {
            let (outer, holes) = g.to_dxf_loops();
            sheet_45.set_dxf_loops(outer, holes);
        }
        sheet_45.add_bend([-47.78, 41.86], [47.78, 41.86], 90.0, BendDirection::Up);
        sheet_45.add_bend([-48.47, -56.80], [47.53, -56.80], 90.0, BendDirection::Up);

        let tab_45 = TruFoldDocumentTab {
            id: "tab_2".to_string(),
            title: "45_deg_inside_riser_100x100.dxf".to_string(),
            file_path: Some(dxf_45_path.to_string()),
            rigged_sheet: sheet_45,
            active_dxf_geometry: geom_45,
            fold_slider_deg: 0.0,
            camera_mode: CanvasCameraMode::TopView2D,
            tool_mode: RiggingToolMode::Select,
            pending_p1: None,
            pan: Vec2::ZERO,
            zoom: 0.85,
            orbit_yaw: 0.60,
            orbit_pitch: 0.55,
            show_floor_blueprint: true,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        };

        let trufold_tabs = vec![tab_90, tab_45];

        Self {
            active_tab: StudioWorkspaceTab::Dashboard,
            landing,
            cms_ext: CadenseSolidCraftExtension::default(),
            active_solidcraft_file: dxf_90_path.to_string(),
            solidcraft_status: None,
            theme: CadenseTheme::SolidCraftCobalt, // Default to SolidCraft Signature Cobalt theme!
            font_scale: 1.05,                      // Generous legible default scaling
            show_settings_modal: false,
            prefs_selected_page: 1, // Default to 1 (🎨 Interface) matching SolidCraft screenshot!
            temp_custom_asset_root: asset_root,
            last_applied_theme_key: None,

            trufold_tabs,
            active_trufold_tab_idx: 0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),

            rigged_sheet: sheet_90,
            rigging_camera_mode: CanvasCameraMode::TopView2D,
            rigging_tool_mode: RiggingToolMode::Select,
            rigging_fold_slider_deg: 0.0,
            rigging_show_floor_blueprint: true,
            rigging_pending_p1: None,
            rigging_pan: Vec2::ZERO,
            rigging_zoom: 0.85,
            rigging_orbit_yaw: 0.60,
            rigging_orbit_pitch: 0.55,
            active_dxf_geometry: geom_90,

            show_3d_viewport_window: true,
            floating_orbit_yaw: 0.60,
            floating_orbit_pitch: 0.55,
            floating_pan: Vec2::ZERO,
            floating_zoom: 1.0,
        }
    }

    // === Multi-Tab Document Operations (User Request 3) ===

    pub fn sync_active_tab_from_state(&mut self) {
        if let Some(tab) = self.trufold_tabs.get_mut(self.active_trufold_tab_idx) {
            tab.rigged_sheet = self.rigged_sheet.clone();
            tab.active_dxf_geometry = self.active_dxf_geometry.clone();
            tab.fold_slider_deg = self.rigging_fold_slider_deg;
            tab.camera_mode = self.rigging_camera_mode;
            tab.tool_mode = self.rigging_tool_mode;
            tab.pending_p1 = self.rigging_pending_p1;
            tab.pan = self.rigging_pan;
            tab.zoom = self.rigging_zoom;
            tab.orbit_yaw = self.rigging_orbit_yaw;
            tab.orbit_pitch = self.rigging_orbit_pitch;
            tab.show_floor_blueprint = self.rigging_show_floor_blueprint;
            tab.undo_stack = self.undo_stack.clone();
            tab.redo_stack = self.redo_stack.clone();
        }
    }

    pub fn load_active_tab_into_state(&mut self) {
        if let Some(tab) = self.trufold_tabs.get_mut(self.active_trufold_tab_idx) {
            if let Some(ref g) = tab.active_dxf_geometry {
                if tab.rigged_sheet.outer_contour_2d.vertices.len() <= 4
                    && tab.rigged_sheet.outer_contour_2d.vertices.iter().any(|v| v[0].abs() == 250.0 || v[0].abs() == 200.0)
                {
                    let (outer, holes) = g.to_dxf_loops();
                    tab.rigged_sheet.set_dxf_loops(outer, holes);
                }
            }
            self.rigged_sheet = tab.rigged_sheet.clone();
            self.active_dxf_geometry = tab.active_dxf_geometry.clone();
            self.rigging_fold_slider_deg = tab.fold_slider_deg;
            self.rigging_camera_mode = tab.camera_mode;
            self.rigging_tool_mode = tab.tool_mode;
            self.rigging_pending_p1 = tab.pending_p1;
            self.rigging_pan = tab.pan;
            self.rigging_zoom = tab.zoom;
            self.rigging_orbit_yaw = tab.orbit_yaw;
            self.rigging_orbit_pitch = tab.orbit_pitch;
            self.rigging_show_floor_blueprint = tab.show_floor_blueprint;
            self.undo_stack = tab.undo_stack.clone();
            self.redo_stack = tab.redo_stack.clone();
            if let Some(fp) = &tab.file_path {
                self.active_solidcraft_file = fp.clone();
            }
        }
    }

    pub fn open_drawing_in_trufold(&mut self, path: &str) {
        self.sync_active_tab_from_state();

        let file_stem = Path::new(path).file_stem().and_then(|s| s.to_str()).unwrap_or("drawing");
        let file_name = Path::new(path).file_name().and_then(|s| s.to_str()).unwrap_or("drawing.dxf");

        if let Some(existing_idx) = self.trufold_tabs.iter().position(|t| t.file_path.as_deref() == Some(path) || t.title == file_name) {
            self.switch_trufold_tab(existing_idx);
            self.active_tab = StudioWorkspaceTab::CmsTruFoldWorkbench;
            self.solidcraft_status = Some(format!("✓ Switched to active drawing tab: {}", file_name));
            return;
        }

        let geom = if path.ends_with(".dxf") { DxfGeometry2D::parse_from_file(path, 0.5) } else { None };

        let is_tray = path.contains("tray");
        let mut rigged_sheet = DsfRiggedSheet::new(
            if is_tray { CmsCategory::CableTray } else { CmsCategory::CableTrunking },
            format!("BONN-{}", file_stem.to_uppercase().replace('_', "-")),
            file_stem.replace('_', " "),
            SheetMaterialRule::bonn_metal_1_5mm(),
        );
        if let Some(ref g) = geom {
            let (outer, holes) = g.to_dxf_loops();
            rigged_sheet.set_dxf_loops(outer, holes);
        }

        if file_stem.contains("90_bend") {
            rigged_sheet.add_bend([-47.78, 41.86], [47.78, 41.86], 90.0, BendDirection::Up);
            rigged_sheet.add_bend([-48.47, -56.80], [47.53, -56.80], 90.0, BendDirection::Up);
        } else if file_stem.contains("45_deg_inside") {
            rigged_sheet.add_bend([-47.78, 41.86], [47.78, 41.86], 90.0, BendDirection::Up);
            rigged_sheet.add_bend([-48.47, -56.80], [47.53, -56.80], 90.0, BendDirection::Up);
        }

        let new_tab = TruFoldDocumentTab {
            id: format!("tab_{}", self.trufold_tabs.len() + 1),
            title: file_name.to_string(),
            file_path: Some(path.to_string()),
            rigged_sheet: rigged_sheet.clone(),
            active_dxf_geometry: geom.clone(),
            fold_slider_deg: 0.0,
            camera_mode: CanvasCameraMode::TopView2D,
            tool_mode: RiggingToolMode::Select,
            pending_p1: None,
            pan: Vec2::ZERO,
            zoom: 0.85,
            orbit_yaw: 0.60,
            orbit_pitch: 0.55,
            show_floor_blueprint: true,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        };

        self.trufold_tabs.push(new_tab);
        self.active_trufold_tab_idx = self.trufold_tabs.len() - 1;
        self.load_active_tab_into_state();
        self.active_tab = StudioWorkspaceTab::CmsTruFoldWorkbench;
        self.solidcraft_status = Some(format!("✓ Opened {} in new TruFold tab", file_name));
    }

    pub fn close_trufold_tab(&mut self, idx: usize) {
        if idx >= self.trufold_tabs.len() {
            return;
        }
        let closed_title = self.trufold_tabs[idx].title.clone();
        self.trufold_tabs.remove(idx);

        if self.trufold_tabs.is_empty() {
            let default_dxf = "assets/dxf/tray_accessories/90_bend_100x50.dxf";
            self.open_drawing_in_trufold(default_dxf);
            self.active_tab = StudioWorkspaceTab::Dashboard;
        } else {
            if self.active_trufold_tab_idx >= self.trufold_tabs.len() {
                self.active_trufold_tab_idx = self.trufold_tabs.len() - 1;
            } else if idx < self.active_trufold_tab_idx {
                self.active_trufold_tab_idx = self.active_trufold_tab_idx.saturating_sub(1);
            }
            self.load_active_tab_into_state();
        }
        self.solidcraft_status = Some(format!("Closed tab: {}", closed_title));
    }

    pub fn switch_trufold_tab(&mut self, new_idx: usize) {
        if new_idx < self.trufold_tabs.len() && new_idx != self.active_trufold_tab_idx {
            self.sync_active_tab_from_state();
            self.active_trufold_tab_idx = new_idx;
            self.load_active_tab_into_state();
        }
    }

    // === Undo / Redo Operations (User Request 1) ===

    pub fn push_trufold_undo(&mut self) {
        self.undo_stack.push(self.rigged_sheet.clone());
        if self.undo_stack.len() > 50 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
        self.sync_active_tab_from_state();
    }

    pub fn trufold_undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.rigged_sheet.clone());
            self.rigged_sheet = prev;
            self.solidcraft_status = Some("↶ Undo: Reverted sheet modification".to_string());
            self.sync_active_tab_from_state();
        }
    }

    pub fn trufold_redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.rigged_sheet.clone());
            self.rigged_sheet = next;
            self.solidcraft_status = Some("↷ Redo: Reapplied sheet modification".to_string());
            self.sync_active_tab_from_state();
        }
    }

    fn export_concept_23_svgs(&mut self) {
        let dir = Path::new("assets/branding");
        let _ = fs::create_dir_all(dir);
        let _ = fs::write(dir.join("cadense_concept23_badge.svg"), CONCEPT_23_BADGE_SVG);
        let _ = fs::write(dir.join("cadense_concept23_metallic_3d.svg"), CONCEPT_23_METALLIC_3D_SVG);
        let _ = fs::write(dir.join("cadense_concept23_horizontal_lockup.svg"), CONCEPT_23_HORIZONTAL_LOCKUP_SVG);
        self.landing.last_export_notice = Some("Exported Concept 23 SVGs to assets/branding/ (badge, metallic_3d, horizontal_lockup).".to_string());
    }

    fn select_preset_code(&mut self, code: BonnCatalogCode) {
        if let Some(idx) = self.cms_ext.registry.entries().iter().position(|e| e.catalog_code == code) {
            self.cms_ext.selected_variation_idx = idx;
            self.cms_ext.port_dims.clear();
            for p in 1..=code.port_count() {
                self.cms_ext.port_dims.insert(p, (300.0, 50.0));
            }
            self.cms_ext.dsf_export_path = format!("assets/templates/bonn_metal/{}.dsf", code.as_str().to_lowercase().replace('-', "_"));
            self.cms_ext.last_solution =
                Some(cadense_dsf::trapezoidal_scaler::TrapezoidalMultiPortScaler::solve(&self.cms_ext.port_dims, self.cms_ext.transition_length_mm));
        }
    }

    fn launch_solidcraft_process(&mut self, target_file: Option<&str>) {
        match launch_solidcraft(target_file) {
            Ok(pid) => {
                let msg = match target_file {
                    Some(f) => format!("✓ Launched SolidCraft CAD (PID: {}) with file: {}", pid, f),
                    None => format!("✓ Launched SolidCraft CAD Application (PID: {})", pid),
                };
                self.solidcraft_status = Some(msg.clone());
                self.landing.status_notice = Some(msg);
            }
            Err(e) => {
                let err_msg = format!("⚠ SolidCraft Launch Failed: {}", e);
                self.solidcraft_status = Some(err_msg.clone());
                self.landing.status_notice = Some(err_msg);
            }
        }
    }

    fn generate_submittal_pdf(&mut self) {
        let mut meta = cadense_dds_pdf::DdsProjectMeta::default();
        meta.project_name = format!("{} SUBMITTAL", self.rigged_sheet.product_name.to_uppercase());
        meta.order_reference = format!("ORD-{}", self.rigged_sheet.product_code);

        let mut dds = cadense_dds_pdf::DdsArchive::new(meta);
        dds.populate_from_order_queue(&self.cms_ext.order_engine);

        // If order queue is empty or if exporting from the workbench, inject active rigged sheet as a TDS entry
        if dds.manifest.item_schedule.is_empty() {
            let mut dims = std::collections::BTreeMap::new();
            dims.insert("W".to_string(), 100.0);
            dims.insert("H".to_string(), 100.0);
            let row = cadense_dds_pdf::dds::DdsItemRow {
                sn: 1,
                product_code: self.rigged_sheet.product_code.clone(),
                dsf_filename: format!("{}.dsf", self.rigged_sheet.product_code.to_lowercase().replace('-', "_")),
                description: self.rigged_sheet.product_name.clone(),
                dimensions: dims,
                qty: 1,
                finish: if self.rigged_sheet.material_rule.thickness_mm >= 2.0 { "HDG".to_string() } else { "GI".to_string() },
                thickness_mm: self.rigged_sheet.material_rule.thickness_mm,
            };
            dds.manifest.item_schedule.push(row);
        }

        let pdf_bytes = cadense_dds_pdf::BonnMetalSubmittalGenerator::generate_pdf(&dds, cadense_dds_pdf::PageMode::GroupedSubmittal);

        let out_dir = Path::new("orders/customer_orders");
        let _ = fs::create_dir_all(out_dir);
        let out_file = out_dir.join("BONN_METAL_TDS_SUBMITTAL.pdf");

        if let Ok(mut f) = fs::File::create(&out_file) {
            use std::io::Write;
            let _ = f.write_all(&pdf_bytes);
            let notice = format!("✓ Generated Bonn Metal Submittal PDF: {} ({:.1} KB)", out_file.display(), pdf_bytes.len() as f64 / 1024.0);
            self.solidcraft_status = Some(notice.clone());
            self.landing.status_notice = Some(notice);
        }
    }

    fn handle_landing_action(&mut self, act: LandingAction) {
        match act {
            LandingAction::SwitchTab(tab) => {
                self.active_tab = tab;
            }
            LandingAction::OpenSettings => {
                self.show_settings_modal = true;
            }
            LandingAction::LaunchSolidCraft { file } => {
                self.launch_solidcraft_process(file.as_deref());
            }
            LandingAction::OpenCleanDxfInSolidCraft(path) => {
                self.launch_solidcraft_process(Some(&path));
            }
            LandingAction::OpenSolidCraftDrawing(path) => {
                self.launch_solidcraft_process(Some(&path));
            }
            LandingAction::SendDrawingToCmsWorkbench(path) => {
                self.open_drawing_in_trufold(&path);
            }
            LandingAction::OpenPresetInCmsWorkbench(code) => {
                self.select_preset_code(code);
                self.active_tab = StudioWorkspaceTab::CmsTruFoldWorkbench;
            }
            LandingAction::OpenDxfCleanerModal { preselect_path } => {
                if let Some(p) = preselect_path {
                    self.landing.cleaner_state.input_path = p;
                }
                self.landing.cleaner_state.is_open = true;
            }
            LandingAction::OpenCustomerOrderQueue => {
                self.cms_ext.show_order_queue = true;
                self.active_tab = StudioWorkspaceTab::OrderBatchQueue;
            }
            LandingAction::GenerateDdsAndTdsPdf => {
                self.generate_submittal_pdf();
            }
            LandingAction::ExportLogoSvgAssets => {
                self.export_concept_23_svgs();
            }
            LandingAction::OpenFolder(path) => {
                open_path_in_explorer(&path);
            }
        }
    }

    fn nav_item_solidcraft(
        ui: &mut egui::Ui,
        _id_str: &str,
        is_selected: bool,
        icon: &str,
        label: &str,
        shortcut: &str,
        colors: &ThemeColors,
    ) -> bool {
        let is_blue = colors.is_sidebar_blue();
        let height = 38.0; // SolidCraft file_nav_height
        let width = ui.available_width();
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), egui::Sense::click());

        let hovered = response.hovered() || response.has_focus();
        if is_selected || hovered {
            let fill = if is_blue {
                if is_selected {
                    Color32::from_rgba_unmultiplied(255, 255, 255, 52)
                } else {
                    Color32::from_rgba_unmultiplied(255, 255, 255, 24)
                }
            } else {
                if is_selected {
                    colors.accent_subtle
                } else {
                    Color32::from_rgba_unmultiplied(226, 234, 246, 120)
                }
            };
            ui.painter().rect_filled(rect, 8.0, fill);
            if is_selected && !is_blue {
                ui.painter().rect_stroke(rect, 8.0, Stroke::new(1.0, colors.accent), StrokeKind::Inside);
            }
        }

        let inner = rect.shrink2(Vec2::new(12.0, 0.0));
        let key_width = if !shortcut.is_empty() {
            let key_galley = ui.painter().layout_no_wrap(
                shortcut.to_owned(),
                egui::FontId::monospace(11.0),
                if is_blue { Color32::from_rgba_unmultiplied(255, 255, 255, 170) } else { colors.text_muted },
            );
            let kw = key_galley.size().x;
            ui.painter().galley(
                Pos2::new(inner.right() - kw, inner.center().y - key_galley.size().y / 2.0),
                key_galley,
                if is_blue { Color32::from_rgba_unmultiplied(255, 255, 255, 170) } else { colors.text_muted },
            );
            kw + 8.0
        } else {
            0.0
        };

        let label_text = if !icon.is_empty() { format!("{} {}", icon, label) } else { label.to_string() };
        let text_col = if is_blue {
            if is_selected {
                Color32::WHITE
            } else {
                Color32::from_rgba_unmultiplied(255, 255, 255, 225)
            }
        } else {
            if is_selected {
                colors.accent
            } else {
                colors.text_primary
            }
        };

        let font = if is_selected {
            egui::FontId::new(13.5, egui::FontFamily::Proportional)
        } else {
            egui::FontId::new(13.5, egui::FontFamily::Proportional)
        };

        let text_rect = Rect::from_min_max(inner.min, Pos2::new((inner.right() - key_width).max(inner.left()), inner.bottom()));

        let mut job = egui::text::LayoutJob::simple(label_text, font, text_col, text_rect.width());
        job.wrap.max_rows = 1;
        let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
        ui.painter().with_clip_rect(ui.clip_rect().intersect(text_rect)).galley(
            Pos2::new(text_rect.left(), text_rect.center().y - galley.size().y / 2.0),
            galley,
            text_col,
        );

        ui.add_space(2.0);
        response.clicked()
    }

    fn back_to_row(ui: &mut egui::Ui, title: &str, shortcut: &str, colors: &ThemeColors) -> bool {
        let is_blue = colors.is_sidebar_blue();
        let height = 40.0; // SolidCraft file_back_height
        let width = ui.available_width();
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), egui::Sense::click());

        if response.hovered() || response.has_focus() {
            let fill = if is_blue { Color32::from_rgba_unmultiplied(255, 255, 255, 30) } else { Color32::from_rgba_unmultiplied(226, 234, 246, 120) };
            ui.painter().rect_filled(rect, 8.0, fill);
        }

        let inner = rect.shrink2(Vec2::new(12.0, 0.0));
        let ink = if is_blue { Color32::WHITE } else { colors.text_primary };

        // Monospace key badge on right
        let key_galley = ui.painter().layout_no_wrap(shortcut.to_owned(), egui::FontId::monospace(11.0), ink);
        let key_rect = Rect::from_center_size(
            Pos2::new(inner.right() - (key_galley.size().x + 12.0) / 2.0, inner.center().y),
            key_galley.size() + Vec2::new(12.0, 6.0),
        );
        ui.painter().rect_stroke(
            key_rect,
            4.0,
            Stroke::new(1.0, if is_blue { Color32::from_rgba_unmultiplied(255, 255, 255, 120) } else { colors.border_strong }),
            StrokeKind::Inside,
        );
        ui.painter().galley(key_rect.center() - key_galley.size() / 2.0, key_galley, ink);

        // Chevron + Text on left
        let text_text = format!("‹  {}", title);
        let text_rect = Rect::from_min_max(inner.min, Pos2::new(key_rect.left() - 8.0, inner.bottom()));
        let mut job = egui::text::LayoutJob::simple(text_text, egui::FontId::new(13.5, egui::FontFamily::Proportional), ink, text_rect.width());
        job.wrap.max_rows = 1;
        let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
        ui.painter().with_clip_rect(ui.clip_rect().intersect(text_rect)).galley(
            Pos2::new(text_rect.left(), text_rect.center().y - galley.size().y / 2.0),
            galley,
            ink,
        );

        ui.add_space(4.0);
        response.clicked()
    }

    fn render_theme_preview_tile(
        ui: &mut egui::Ui,
        th: CadenseTheme,
        current_theme: CadenseTheme,
        name: &str,
        note: &str,
        rail_col: Color32,
        canvas_col: Color32,
        accent_col: Color32,
        colors: &ThemeColors,
    ) -> bool {
        let is_sel = current_theme == th;
        let width = 126.0;
        let height = 86.0;
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), egui::Sense::click());

        let hover = response.hovered();
        let border_stroke = if is_sel {
            Stroke::new(2.0, colors.accent)
        } else if hover {
            Stroke::new(1.2, colors.accent.gamma_multiply(0.7))
        } else {
            Stroke::new(1.0, colors.border_subtle)
        };

        let card_bg = if is_sel {
            colors.selected_bg
        } else if hover {
            colors.bg_card_alt
        } else {
            colors.bg_card
        };

        ui.painter().rect(rect, 8.0, card_bg, border_stroke, StrokeKind::Inside);

        // Mini Illustrated Specimen (top 38px)
        let mini_box = Rect::from_min_size(rect.min + Vec2::new(6.0, 6.0), Vec2::new(width - 12.0, 38.0));
        ui.painter().rect_filled(mini_box, 4.0, Color32::from_rgb(220, 224, 230));

        // Mini Top Bar
        let mini_top = Rect::from_min_size(mini_box.min, Vec2::new(mini_box.width(), 7.0));
        ui.painter().rect_filled(mini_top, 4.0, Color32::from_rgb(52, 56, 64));

        // Mini Sidebar Rail
        let mini_rail = Rect::from_min_max(Pos2::new(mini_box.left(), mini_top.bottom()), Pos2::new(mini_box.left() + 22.0, mini_box.bottom()));
        ui.painter().rect_filled(mini_rail, 0.0, rail_col);

        // Mini Viewport Canvas
        let mini_canvas = Rect::from_min_max(Pos2::new(mini_rail.right(), mini_top.bottom()), mini_box.max);
        ui.painter().rect_filled(mini_canvas, 0.0, canvas_col);

        // Mini accent dot in viewport
        ui.painter().circle_filled(mini_canvas.center(), 3.5, accent_col);

        // Checkmark badge in circle if selected (top right of tile)
        if is_sel {
            let badge_pos = Pos2::new(rect.right() - 11.0, rect.top() + 11.0);
            ui.painter().circle_filled(badge_pos, 7.5, colors.accent);
            let check = ui.painter().layout_no_wrap(
                egui_phosphor::regular::CHECK.to_owned(),
                egui::FontId::new(9.5, egui::FontFamily::Proportional),
                Color32::WHITE,
            );
            ui.painter().galley(badge_pos - check.size() / 2.0, check, Color32::WHITE);
        }

        // Labels at bottom
        let label_top = mini_box.bottom() + 4.0;
        let title_galley = ui.painter().layout_no_wrap(
            name.to_owned(),
            egui::FontId::new(11.5, egui::FontFamily::Proportional),
            if is_sel { colors.accent } else { colors.text_primary },
        );
        ui.painter().galley(Pos2::new(rect.left() + 8.0, label_top), title_galley, if is_sel { colors.accent } else { colors.text_primary });

        let note_galley = ui.painter().layout_no_wrap(note.to_owned(), egui::FontId::new(10.0, egui::FontFamily::Proportional), colors.text_muted);
        ui.painter().galley(Pos2::new(rect.left() + 8.0, label_top + 15.0), note_galley, colors.text_muted);

        response.clicked()
    }

    fn render_import_theme_tile(ui: &mut egui::Ui, colors: &ThemeColors) {
        let width = 110.0;
        let height = 86.0;
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), egui::Sense::click());

        if response.hovered() {
            ui.painter().rect_filled(rect, 8.0, colors.bg_card_alt);
        }

        // Dashed perimeter outline
        let p = rect.shrink(1.0);
        let pts = vec![p.left_top(), p.right_top(), p.right_bottom(), p.left_bottom(), p.left_top()];
        ui.painter().extend(egui::Shape::dashed_line(&pts, Stroke::new(1.2, colors.border_strong), 4.0, 3.0));

        // Download / Import icon & text
        let center = rect.center();
        let icon = ui.painter().layout_no_wrap(
            egui_phosphor::regular::ARROW_DOWN.to_owned(),
            egui::FontId::new(18.0, egui::FontFamily::Proportional),
            colors.text_secondary,
        );
        ui.painter().galley(Pos2::new(center.x - icon.size().x / 2.0, rect.top() + 14.0), icon, colors.text_secondary);

        let t1 = ui.painter().layout_no_wrap("Import...".to_owned(), egui::FontId::new(11.0, egui::FontFamily::Proportional), colors.text_secondary);
        ui.painter().galley(Pos2::new(center.x - t1.size().x / 2.0, rect.top() + 38.0), t1, colors.text_secondary);

        let t2 = ui.painter().layout_no_wrap("theme file".to_owned(), egui::FontId::new(10.0, egui::FontFamily::Proportional), colors.text_muted);
        ui.painter().galley(Pos2::new(center.x - t2.size().x / 2.0, rect.top() + 52.0), t2, colors.text_muted);
    }

    fn render_layout_preview_schematic(ui: &mut egui::Ui, colors: &ThemeColors) {
        let width = 230.0;
        let height = 150.0;
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), egui::Sense::hover());

        // Outer window frame
        ui.painter().rect(rect, 8.0, colors.bg_card, Stroke::new(1.2, Color32::from_rgb(180, 195, 215)), StrokeKind::Inside);

        // Mini top toolbar
        let bar_rect = Rect::from_min_size(rect.min + Vec2::new(4.0, 4.0), Vec2::new(width - 8.0, 18.0));
        ui.painter().rect_filled(bar_rect, 4.0, Color32::from_rgb(240, 244, 248));
        ui.painter().rect_stroke(bar_rect, 4.0, Stroke::new(0.8, Color32::from_rgb(205, 215, 225)), StrokeKind::Inside);

        // Tool dots
        for i in 0..9 {
            let cx = bar_rect.left() + 16.0 + (i as f32 * 20.0);
            ui.painter().circle_filled(Pos2::new(cx, bar_rect.center().y), 2.5, Color32::from_rgb(150, 165, 180));
        }

        // Left Browser Panel Card
        let browser_rect = Rect::from_min_size(Pos2::new(rect.left() + 8.0, bar_rect.bottom() + 6.0), Vec2::new(60.0, 80.0));
        ui.painter().rect(
            browser_rect,
            4.0,
            Color32::from_rgb(250, 252, 254),
            Stroke::new(0.8, Color32::from_rgb(205, 215, 225)),
            StrokeKind::Inside,
        );
        let b_label =
            ui.painter().layout_no_wrap("Browser".to_owned(), egui::FontId::new(9.5, egui::FontFamily::Proportional), Color32::from_rgb(70, 80, 95));
        ui.painter().galley(Pos2::new(browser_rect.left() + 6.0, browser_rect.top() + 5.0), b_label, Color32::from_rgb(70, 80, 95));

        // Browser lines
        for l in 0..3 {
            let ly = browser_rect.top() + 22.0 + (l as f32 * 8.0);
            ui.painter().hline((browser_rect.left() + 6.0)..=(browser_rect.right() - 8.0), ly, Stroke::new(0.8, Color32::from_rgb(210, 220, 230)));
        }

        // 3D Isometric Viewport Specimen (SolidCraft 3D Part Wireframe)
        let vp_center = Pos2::new(rect.left() + 145.0, bar_rect.bottom() + 45.0);
        let hex_pts = vec![
            Pos2::new(vp_center.x, vp_center.y - 18.0),
            Pos2::new(vp_center.x + 16.0, vp_center.y - 9.0),
            Pos2::new(vp_center.x + 16.0, vp_center.y + 11.0),
            Pos2::new(vp_center.x, vp_center.y + 20.0),
            Pos2::new(vp_center.x - 16.0, vp_center.y + 11.0),
            Pos2::new(vp_center.x - 16.0, vp_center.y - 9.0),
        ];
        ui.painter().add(egui::Shape::convex_polygon(
            hex_pts.clone(),
            Color32::from_rgb(180, 195, 215),
            Stroke::new(1.2, Color32::from_rgb(100, 120, 145)),
        ));

        // Part isometric shading line
        ui.painter().line_segment([vp_center, Pos2::new(vp_center.x, vp_center.y + 20.0)], Stroke::new(1.0, Color32::from_rgb(100, 120, 145)));
        ui.painter().line_segment([vp_center, Pos2::new(vp_center.x + 16.0, vp_center.y - 9.0)], Stroke::new(1.0, Color32::from_rgb(100, 120, 145)));
        ui.painter().line_segment([vp_center, Pos2::new(vp_center.x - 16.0, vp_center.y - 9.0)], Stroke::new(1.0, Color32::from_rgb(100, 120, 145)));

        // Pivot dot
        ui.painter().circle_filled(Pos2::new(vp_center.x + 36.0, vp_center.y - 10.0), 4.5, Color32::from_rgb(150, 165, 180));

        // Bottom History Bar
        let hist_rect = Rect::from_min_size(Pos2::new(rect.left() + 8.0, rect.bottom() - 24.0), Vec2::new(width - 16.0, 16.0));
        ui.painter().rect(hist_rect, 4.0, Color32::from_rgb(235, 242, 250), Stroke::new(0.8, Color32::from_rgb(195, 210, 225)), StrokeKind::Inside);
        let h_label =
            ui.painter().layout_no_wrap("History".to_owned(), egui::FontId::new(9.5, egui::FontFamily::Proportional), Color32::from_rgb(70, 80, 95));
        ui.painter().galley(hist_rect.center() - h_label.size() / 2.0, h_label, Color32::from_rgb(70, 80, 95));

        ui.add_space(4.0);
        ui.label(RichText::new("Preview of your choices").size(10.5).color(colors.text_muted));
    }

    fn render_settings_modal(&mut self, ctx: &egui::Context, colors: &ThemeColors) {
        let mut is_open = self.show_settings_modal;
        let mut should_close = false;
        let mut should_apply = false;
        let mut should_defaults = false;

        egui::Window::new(format!("{} Preferences", egui_phosphor::regular::GEAR))
            .open(&mut is_open)
            .default_size(Vec2::new(880.0, 620.0))
            .resizable(true)
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                    // Left sidebar navigation rail (SolidCraft Preferences Style: light grey with hairline rule)
                    let nav_w = 210.0;
                    ui.allocate_ui_with_layout(
                        Vec2::new(nav_w, ui.available_height() - 48.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            Frame::NONE
                                .fill(colors.dialog_nav)
                                .stroke(Stroke::new(1.0, colors.dialog_rule))
                                .inner_margin(Margin::same(12))
                                .corner_radius(6.0)
                                .show(ui, |ui| {
                                    ui.set_width(nav_w - 24.0);
                                    let group_col = colors.text_muted;

                                    let pages: &[(usize, &str, &str, Option<&str>)] = &[
                                        // GENERAL
                                        (0, egui_phosphor::regular::GEAR, "General", Some("GENERAL")),
                                        (1, egui_phosphor::regular::GLOBE, "Interface", None),
                                        (2, egui_phosphor::regular::RULER, "Design", None),
                                        // INPUT
                                        (3, egui_phosphor::regular::MOUSE, "Navigation", Some("INPUT")),
                                        (4, egui_phosphor::regular::KEYBOARD, "Shortcuts", None),
                                        (5, egui_phosphor::regular::PENCIL_SIMPLE, "Sketch & selection", None),
                                        // DISPLAY
                                        (6, egui_phosphor::regular::DESKTOP, "Graphics", Some("DISPLAY")),
                                        // EXTEND
                                        (7, egui_phosphor::regular::CHATS, "Assistant", Some("EXTEND")),
                                        (8, egui_phosphor::regular::PLUG, "Plugins", None),
                                        // CMS FABRICATION
                                        (9, egui_phosphor::regular::FOLDER, "Root Asset Folders", Some("CMS FABRICATION")),
                                        (10, egui_phosphor::regular::ROCKET_LAUNCH, "SolidCraft CAD Link", None),
                                        (11, egui_phosphor::regular::LIGHTNING, "Tolerance & Healing", None),
                                    ];

                                    for &(idx, icon, title, group) in pages {
                                        if let Some(grp_name) = group {
                                            if idx != 0 {
                                                ui.add_space(8.0);
                                            }
                                            ui.label(RichText::new(grp_name).size(10.0).strong().color(group_col));
                                            ui.add_space(2.0);
                                        }

                                        let is_sel = self.prefs_selected_page == idx;
                                        let (bg, fg, stroke) = if is_sel {
                                            (colors.selected_bg, colors.selected_ink, Stroke::new(1.0, Color32::from_rgb(186, 215, 255)))
                                        } else {
                                            (Color32::TRANSPARENT, colors.text_primary, Stroke::NONE)
                                        };

                                        let btn = egui::Button::new(RichText::new(format!("{} {}", icon, title)).size(13.0).strong().color(fg))
                                            .fill(bg)
                                            .stroke(stroke)
                                            .corner_radius(6.0)
                                            .min_size(Vec2::new(ui.available_width(), 30.0));

                                        if ui.add(btn).clicked() {
                                            self.prefs_selected_page = idx;
                                        }
                                        ui.add_space(2.0);
                                    }
                                });
                        },
                    );

                    ui.add_space(8.0);

                    // Right Content Pane (Pure White Background matching SolidCraft)
                    let content_size = Vec2::new(ui.available_width(), ui.available_height() - 48.0);
                    ui.allocate_ui_with_layout(content_size, egui::Layout::top_down(egui::Align::Min), |ui| {
                        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                            match self.prefs_selected_page {
                                0 => {
                                    // GENERAL PAGE
                                    ui.heading("General");
                                    ui.label(RichText::new("Application behavior, startup workflow, and session recovery.").size(12.5).color(colors.text_secondary));
                                    ui.separator();
                                    ui.add_space(6.0);

                                    ui.label(RichText::new("▾ STARTUP & SESSION").size(11.0).strong().color(colors.text_muted));
                                    ui.add_space(6.0);

                                    Frame::NONE
                                        .fill(colors.bg_card)
                                        .stroke(Stroke::new(1.0, colors.border_subtle))
                                        .inner_margin(Margin::same(12))
                                        .corner_radius(6.0)
                                        .show(ui, |ui| {
                                            let mut show_start = true;
                                            ui.checkbox(&mut show_start, "Show Start screen on launch (recommended)");
                                            ui.add_space(4.0);
                                            let mut check_updates = true;
                                            ui.checkbox(&mut check_updates, "Check for SolidCraft upstream updates on start");
                                            ui.add_space(4.0);
                                            let mut crash_recovery = true;
                                            ui.checkbox(&mut crash_recovery, "Automatic crash recovery and session restoration");
                                        });

                                    ui.add_space(12.0);
                                    ui.label(RichText::new("▾ UNITS & PRECISION").size(11.0).strong().color(colors.text_muted));
                                    ui.add_space(6.0);

                                    egui::Grid::new("prefs_general_units_grid")
                                        .num_columns(2)
                                        .spacing(Vec2::new(20.0, 8.0))
                                        .show(ui, |ui| {
                                            ui.label(RichText::new("Linear Units:").strong());
                                            ui.label("Millimetres (mm) — Standard Sheet Metal Fabrication");
                                            ui.end_row();

                                            ui.label(RichText::new("Angular Units:").strong());
                                            ui.label("Degrees (°)");
                                            ui.end_row();

                                            ui.label(RichText::new("Decimal Display:").strong());
                                            ui.label("2 Decimal Places (0.01 mm precision)");
                                            ui.end_row();

                                            ui.label(RichText::new("Autosave Frequency:").strong());
                                            ui.label("Every 5 minutes (drawings/cms_in_progress/autosave/)");
                                            ui.end_row();
                                        });
                                }
                                1 => {
                                    // INTERFACE PAGE (Directly reproducing the user's reference screenshot!)
                                    ui.heading("Interface");
                                    ui.label(RichText::new("How SolidCraft and CADense Studio look and where their parts sit.").size(12.5).color(colors.text_secondary));
                                    ui.separator();
                                    ui.add_space(6.0);

                                    ui.label(RichText::new("▾ THEME").size(11.0).strong().color(colors.text_muted));
                                    ui.add_space(6.0);

                                    // Theme swatch cards rail (matching SolidCraft's theme swatches)
                                    ui.horizontal(|ui| {
                                        if Self::render_theme_preview_tile(
                                            ui,
                                            CadenseTheme::SolidCraftCobalt,
                                            self.theme,
                                            "Light",
                                            "Built in",
                                            Color32::from_rgb(35, 110, 173),
                                            Color32::from_rgb(247, 249, 251),
                                            Color32::from_rgb(38, 120, 218),
                                            colors,
                                        ) {
                                            self.theme = CadenseTheme::SolidCraftCobalt;
                                        }
                                        ui.add_space(4.0);

                                        if Self::render_theme_preview_tile(
                                            ui,
                                            CadenseTheme::DeepObsidian,
                                            self.theme,
                                            "Dark",
                                            "Built in",
                                            Color32::from_rgb(30, 80, 128),
                                            Color32::from_rgb(11, 17, 32),
                                            Color32::from_rgb(56, 189, 248),
                                            colors,
                                        ) {
                                            self.theme = CadenseTheme::DeepObsidian;
                                        }
                                        ui.add_space(4.0);

                                        if Self::render_theme_preview_tile(
                                            ui,
                                            CadenseTheme::ModernLight,
                                            self.theme,
                                            "System",
                                            "Follows the OS",
                                            Color32::from_rgb(241, 245, 249),
                                            Color32::from_rgb(248, 250, 252),
                                            Color32::from_rgb(37, 99, 235),
                                            colors,
                                        ) {
                                            self.theme = CadenseTheme::ModernLight;
                                        }
                                        ui.add_space(4.0);

                                        if Self::render_theme_preview_tile(
                                            ui,
                                            CadenseTheme::GruvboxDark,
                                            self.theme,
                                            "Gruvbox Dark",
                                            "Built in",
                                            Color32::from_rgb(29, 32, 33),
                                            Color32::from_rgb(40, 40, 40),
                                            Color32::from_rgb(142, 192, 124),
                                            colors,
                                        ) {
                                            self.theme = CadenseTheme::GruvboxDark;
                                        }
                                        ui.add_space(4.0);

                                        if Self::render_theme_preview_tile(
                                            ui,
                                            CadenseTheme::WarmClay,
                                            self.theme,
                                            "Warm Clay",
                                            "Built in",
                                            Color32::from_rgb(236, 231, 220),
                                            Color32::from_rgb(245, 242, 235),
                                            Color32::from_rgb(194, 94, 62),
                                            colors,
                                        ) {
                                            self.theme = CadenseTheme::WarmClay;
                                        }
                                        ui.add_space(4.0);

                                        Self::render_import_theme_tile(ui, colors);
                                    });

                                    ui.add_space(10.0);
                                    ui.label(RichText::new("The accent follows the active tab").size(11.5).color(colors.text_secondary));
                                    ui.add_space(4.0);

                                    // Tab accent indicators (SolidCraft 6 accents)
                                    ui.horizontal(|ui| {
                                        let tab_accents = [
                                            ("Model", Color32::from_rgb(38, 120, 218)),
                                            ("Surface", Color32::from_rgb(164, 80, 0)),
                                            ("Mesh", Color32::from_rgb(224, 178, 0)),
                                            ("Form", Color32::from_rgb(115, 73, 205)),
                                            ("Sheet Metal", Color32::from_rgb(30, 111, 150)),
                                            ("Render", Color32::from_rgb(168, 50, 110)),
                                        ];
                                        for (name, col) in tab_accents {
                                            ui.vertical(|ui| {
                                                ui.painter().rect_filled(Rect::from_min_size(ui.cursor().min, Vec2::new(76.0, 4.0)), 2.0, col);
                                                ui.add_space(6.0);
                                                ui.label(RichText::new(name).size(11.0).color(colors.text_muted));
                                            });
                                            ui.add_space(6.0);
                                        }
                                    });

                                    ui.add_space(10.0);
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("Viewport:").strong().color(colors.text_primary));
                                        let _ = ui.selectable_label(true, "Follow theme");
                                        let _ = ui.selectable_label(false, "Light");
                                        let _ = ui.selectable_label(false, "Dark");
                                    });

                                    ui.add_space(14.0);
                                    ui.separator();
                                    ui.add_space(6.0);

                                    // ▾ LAYOUT Section (Dual column with vector preview on right!)
                                    ui.label(RichText::new("▾ LAYOUT").size(11.0).strong().color(colors.text_muted));
                                    ui.add_space(8.0);

                                    ui.horizontal(|ui| {
                                        // Left column: form options
                                        ui.allocate_ui_with_layout(Vec2::new(ui.available_width() - 250.0, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| {
                                            egui::Grid::new("prefs_layout_grid")
                                                .num_columns(2)
                                                .spacing(Vec2::new(18.0, 10.0))
                                                .show(ui, |ui| {
                                                    ui.label(RichText::new("Workspace").color(colors.text_secondary));
                                                    ui.horizontal(|ui| {
                                                        let _ = ui.selectable_label(true, "Floating");
                                                        let _ = ui.selectable_label(false, "Compact");
                                                    });
                                                    ui.end_row();

                                                    ui.label(RichText::new("Toolbar").color(colors.text_secondary));
                                                    ui.horizontal(|ui| {
                                                        let _ = ui.selectable_label(true, "Labels");
                                                        let _ = ui.selectable_label(false, "Icons only");
                                                    });
                                                    ui.end_row();

                                                    ui.label(RichText::new("Dialog buttons").color(colors.text_secondary));
                                                    ui.horizontal(|ui| {
                                                        let _ = ui.selectable_label(true, "Bottom");
                                                        let _ = ui.selectable_label(false, "Top");
                                                    });
                                                    ui.end_row();

                                                    ui.label(RichText::new("History").color(colors.text_secondary));
                                                    ui.horizontal(|ui| {
                                                        let _ = ui.selectable_label(true, "Bottom");
                                                        let _ = ui.selectable_label(false, "Left");
                                                        let _ = ui.selectable_label(false, "Under browser");
                                                    });
                                                    ui.end_row();

                                                    ui.label(RichText::new("Sketches").color(colors.text_secondary));
                                                    ui.horizontal(|ui| {
                                                        let _ = ui.selectable_label(true, "In order");
                                                        let _ = ui.selectable_label(false, "Under their feature");
                                                    });
                                                    ui.end_row();

                                                    ui.label("");
                                                    let mut origin_top = false;
                                                    ui.checkbox(&mut origin_top, "Show origin and bodies at the top");
                                                    ui.end_row();

                                                    ui.label(RichText::new("Browser").color(colors.text_secondary));
                                                    ui.horizontal(|ui| {
                                                        let _ = ui.selectable_label(true, "Opaque");
                                                        let _ = ui.selectable_label(false, "Translucent");
                                                    });
                                                    ui.end_row();

                                                    ui.label("");
                                                    let mut ok_corner = false;
                                                    ui.checkbox(&mut ok_corner, "OK and Cancel in the view's corner");
                                                    ui.end_row();
                                                });

                                            ui.add_space(8.0);
                                            ui.label(RichText::new("Compact docks panels to the window edges. Browser translucency is available in Floating. Change the Assistant side on the Assistant page.").size(10.5).color(colors.text_muted));
                                        });

                                        ui.add_space(10.0);

                                        // Right column: Vector Wireframe Schematic
                                        ui.vertical(|ui| {
                                            Self::render_layout_preview_schematic(ui, colors);
                                        });
                                    });
                                }
                                2 => {
                                    // DESIGN PAGE
                                    ui.heading("Design & Sheet Metal");
                                    ui.label(RichText::new("Default sheet metal parameters, bend deductions, and neutral axis K-Factors.").size(12.5).color(colors.text_secondary));
                                    ui.separator();
                                    ui.add_space(6.0);

                                    ui.label(RichText::new("▾ MATERIAL SPECIFICATION").size(11.0).strong().color(colors.text_muted));
                                    ui.add_space(6.0);

                                    egui::Grid::new("prefs_design_grid")
                                        .num_columns(2)
                                        .spacing(Vec2::new(20.0, 8.0))
                                        .striped(true)
                                        .show(ui, |ui| {
                                            ui.label(RichText::new("Default Material:").strong());
                                            ui.label("Galvanized Iron (GI) — Hot-Dip Coated to BS EN 10346");
                                            ui.end_row();

                                            ui.label(RichText::new("Standard Thickness:").strong());
                                            ui.label("1.50 mm (16 Gauge) / 2.00 mm (14 Gauge)");
                                            ui.end_row();

                                            ui.label(RichText::new("Neutral Axis K-Factor:").strong());
                                            ui.label("0.44 (Air Bending Standard for Mild Steel & GI)");
                                            ui.end_row();

                                            ui.label(RichText::new("Inner Bend Radius (R):").strong());
                                            ui.label("1.50 mm (1.0 × Thickness)");
                                            ui.end_row();

                                            ui.label(RichText::new("Standard Flange Heights:").strong());
                                            ui.label("50 mm, 75 mm, 100 mm (Site Technology & Bonn Metal Standards)");
                                            ui.end_row();
                                        });
                                }
                                3 => {
                                    // NAVIGATION PAGE
                                    ui.heading("Navigation");
                                    ui.label(RichText::new("Viewport camera interaction, mouse orbit style, and zoom behavior.").size(12.5).color(colors.text_secondary));
                                    ui.separator();
                                    ui.add_space(6.0);

                                    ui.label(RichText::new("▾ MOUSE & CAMERA CONTROLS").size(11.0).strong().color(colors.text_muted));
                                    ui.add_space(6.0);

                                    egui::Grid::new("prefs_nav_grid")
                                        .num_columns(2)
                                        .spacing(Vec2::new(20.0, 8.0))
                                        .striped(true)
                                        .show(ui, |ui| {
                                            ui.label(RichText::new("3D Camera Orbit:").strong());
                                            ui.horizontal(|ui| {
                                                let _ = ui.selectable_label(true, "Middle Mouse Button");
                                                let _ = ui.selectable_label(false, "Shift + MMB");
                                            });
                                            ui.end_row();

                                            ui.label(RichText::new("2D Pan:").strong());
                                            ui.label("Shift + MMB or Right Mouse Drag");
                                            ui.end_row();

                                            ui.label(RichText::new("Zoom:").strong());
                                            ui.label("Mouse Scroll Wheel (Zoom to Cursor)");
                                            ui.end_row();

                                            ui.label(RichText::new("Zoom Extents (Fit):").strong());
                                            ui.label("F Key or Double-Click MMB");
                                            ui.end_row();
                                        });
                                }
                                4 => {
                                    // SHORTCUTS PAGE
                                    ui.heading("Shortcuts");
                                    ui.label(RichText::new("Keyboard accelerators across CADense Studio and SolidCraft CAD.").size(12.5).color(colors.text_secondary));
                                    ui.separator();
                                    ui.add_space(6.0);

                                    ui.label(RichText::new("▾ WORKSPACE & COMMAND SHORTCUTS").size(11.0).strong().color(colors.text_muted));
                                    ui.add_space(6.0);

                                    let shortcuts = [
                                        ("Start / Dashboard Hub", "Esc / Ctrl+1"),
                                        ("Stage 1: Pre-Flight DXF Cleaner", "Ctrl+2"),
                                        ("Stage 2: SolidCraft 3D CAD Bending", "Ctrl+3"),
                                        ("Stage 3: CMS TruFold Rigging", "Ctrl+4"),
                                        ("Stage 4: Orders & Submittal TDS", "Ctrl+5"),
                                        ("Preferences & Settings", "Ctrl+,"),
                                        ("Launch SolidCraft Standalone Process", "Ctrl+L"),
                                        ("Open Root Assets in Explorer", "Ctrl+Shift+A"),
                                        ("Open In-Progress Drawings", "Ctrl+Shift+D"),
                                        ("Zoom Extents / Fit View", "F"),
                                        ("Command Palette", "Space"),
                                    ];

                                    egui::Grid::new("prefs_shortcuts_grid")
                                        .num_columns(2)
                                        .spacing(Vec2::new(30.0, 8.0))
                                        .striped(true)
                                        .show(ui, |ui| {
                                            for (action, key) in shortcuts {
                                                ui.label(RichText::new(action).color(colors.text_primary));
                                                ui.label(RichText::new(key).monospace().strong().color(colors.accent));
                                                ui.end_row();
                                            }
                                        });
                                }
                                5 => {
                                    // SKETCH & SELECTION PAGE
                                    ui.heading("Sketch & selection");
                                    ui.label(RichText::new("Geometric inferencing, snap radii, and loop closure solvers.").size(12.5).color(colors.text_secondary));
                                    ui.separator();
                                    ui.add_space(6.0);

                                    ui.label(RichText::new("▾ SNAPPING & GEOMETRIC INFERENCING").size(11.0).strong().color(colors.text_muted));
                                    ui.add_space(6.0);

                                    Frame::NONE
                                        .fill(colors.bg_card)
                                        .stroke(Stroke::new(1.0, colors.border_subtle))
                                        .inner_margin(Margin::same(12))
                                        .corner_radius(6.0)
                                        .show(ui, |ui| {
                                            let mut snap_endpoints = true;
                                            ui.checkbox(&mut snap_endpoints, "Snap to curve endpoints and polygon vertices");
                                            ui.add_space(4.0);
                                            let mut snap_midpoints = true;
                                            ui.checkbox(&mut snap_midpoints, "Snap to segment midpoints and arc centers");
                                            ui.add_space(4.0);
                                            let mut auto_horizontal = true;
                                            ui.checkbox(&mut auto_horizontal, "Auto-infer horizontal and vertical constraints");
                                            ui.add_space(4.0);
                                            let mut merge_coincident = true;
                                            ui.checkbox(&mut merge_coincident, "Auto-heal coincident endpoints within gap threshold");
                                        });
                                }
                                6 => {
                                    // GRAPHICS PAGE
                                    ui.heading("Graphics");
                                    ui.label(RichText::new("Hardware acceleration, display antialiasing, and GPU rasterization.").size(12.5).color(colors.text_secondary));
                                    ui.separator();
                                    ui.add_space(6.0);

                                    ui.label(RichText::new("▾ RENDERING PIPELINE").size(11.0).strong().color(colors.text_muted));
                                    ui.add_space(6.0);

                                    egui::Grid::new("prefs_graphics_grid")
                                        .num_columns(2)
                                        .spacing(Vec2::new(20.0, 8.0))
                                        .striped(true)
                                        .show(ui, |ui| {
                                            ui.label(RichText::new("Graphics Driver:").strong());
                                            ui.colored_label(Color32::from_rgb(16, 185, 129), "wgpu (Direct3D 12 / Vulkan Hardware Acceleration)");
                                            ui.end_row();

                                            ui.label(RichText::new("Antialiasing (MSAA):").strong());
                                            ui.label("4x MSAA (High Quality)");
                                            ui.end_row();

                                            ui.label(RichText::new("Target Refresh Rate:").strong());
                                            ui.label("VSync Synchronized (Matches Display)");
                                            ui.end_row();
                                        });
                                }
                                7 => {
                                    // ASSISTANT PAGE
                                    ui.heading("Assistant");
                                    ui.label(RichText::new("AI pair-programming bridge and SolidCraft MCP protocol tools.").size(12.5).color(colors.text_secondary));
                                    ui.separator();
                                    ui.add_space(6.0);

                                    ui.label(RichText::new("▾ CONTROL CHANNEL & AGENT PROTOCOL").size(11.0).strong().color(colors.text_muted));
                                    ui.add_space(6.0);

                                    egui::Grid::new("prefs_assistant_grid")
                                        .num_columns(2)
                                        .spacing(Vec2::new(20.0, 8.0))
                                        .striped(true)
                                        .show(ui, |ui| {
                                            ui.label(RichText::new("Control Channel Server:").strong());
                                            ui.colored_label(Color32::from_rgb(16, 185, 129), "127.0.0.1:27878 (Listening)");
                                            ui.end_row();

                                            ui.label(RichText::new("Protocol Standard:").strong());
                                            ui.label("Model Context Protocol (crates/mcp)");
                                            ui.end_row();

                                            ui.label(RichText::new("Assistant Dock Side:").strong());
                                            ui.horizontal(|ui| {
                                                let _ = ui.selectable_label(true, "Right");
                                                let _ = ui.selectable_label(false, "Left");
                                            });
                                            ui.end_row();
                                        });
                                }
                                8 => {
                                    // PLUGINS PAGE
                                    ui.heading("Plugins");
                                    ui.label(RichText::new("Loaded modular extensions and container handlers.").size(12.5).color(colors.text_secondary));
                                    ui.separator();
                                    ui.add_space(6.0);

                                    ui.label(RichText::new("▾ INSTALLED EXTENSIONS").size(11.0).strong().color(colors.text_muted));
                                    ui.add_space(6.0);

                                    let plugins = [
                                        ("cadense-dsf", "v0.2.0", "Danish Sheet Fold (.dsf) container parser & port assigner", "Active"),
                                        ("cadense-order", "v0.2.0", "Customer Order CSV batch runner & formula engine", "Active"),
                                        ("cadense-dds-pdf", "v0.2.0", "3D-to-2D HLR vector projector & TDS generator", "Active"),
                                        ("cadense-ui", "v0.2.0", "SolidCraft design system components & themes", "Active"),
                                    ];

                                    egui::Grid::new("prefs_plugins_grid")
                                        .num_columns(4)
                                        .spacing(Vec2::new(16.0, 8.0))
                                        .striped(true)
                                        .show(ui, |ui| {
                                            for (name, ver, desc, status) in plugins {
                                                ui.label(RichText::new(name).monospace().strong().color(colors.accent));
                                                ui.label(RichText::new(ver).color(colors.text_secondary));
                                                ui.label(RichText::new(desc).color(colors.text_primary));
                                                ui.colored_label(Color32::from_rgb(16, 185, 129), status);
                                                ui.end_row();
                                            }
                                        });
                                }
                                9 => {
                                    // ROOT ASSET FOLDERS PAGE
                                    ui.heading("Root Asset Folders & Storage");
                                    ui.label(RichText::new("Root directory location for 2D CNC laser-cut DXFs, DSF templates, and customer orders.").size(12.5).color(colors.text_secondary));
                                    ui.separator();
                                    ui.add_space(8.0);

                                    ui.label(RichText::new("▾ ACTIVE ROOT ASSET DIRECTORY").size(11.0).strong().color(colors.text_muted));
                                    ui.add_space(6.0);

                                    Frame::NONE
                                        .fill(colors.bg_card)
                                        .stroke(Stroke::new(1.0, colors.border_subtle))
                                        .inner_margin(Margin::same(12))
                                        .corner_radius(6.0)
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(RichText::new("Path:").strong().color(colors.text_primary));
                                                ui.add(egui::TextEdit::singleline(&mut self.temp_custom_asset_root).desired_width(ui.available_width() - 140.0));
                                                if ui.button("Apply Path").clicked() {
                                                    self.landing.custom_asset_root = Some(self.temp_custom_asset_root.clone());
                                                    self.landing.refresh_dxf_library();
                                                }
                                            });
                                            ui.add_space(6.0);
                                            ui.horizontal(|ui| {
                                                if ui.button(format!("{} Open Folder in Explorer", egui_phosphor::regular::FOLDER_OPEN)).clicked() {
                                                    open_path_in_explorer(&self.landing.resolved_asset_root);
                                                }
                                                if ui.button(format!("{} Rescan All Subfolders", egui_phosphor::regular::ARROWS_CLOCKWISE)).clicked() {
                                                    self.landing.refresh_dxf_library();
                                                }
                                                if ui.button("Reset to Auto-Detect").clicked() {
                                                    self.landing.custom_asset_root = None;
                                                    self.landing.refresh_dxf_library();
                                                    self.temp_custom_asset_root = self.landing.resolved_asset_root.clone();
                                                }
                                            });
                                        });

                                    ui.add_space(14.0);
                                    ui.label(RichText::new("▾ INDEXED SUBFOLDERS INVENTORY").size(11.0).strong().color(colors.text_muted));
                                    ui.add_space(6.0);

                                    let tray_count = self.landing.scanned_dxf_items.iter().filter(|i| i.category.contains("Tray")).count();
                                    let trunk_count = self.landing.scanned_dxf_items.iter().filter(|i| i.category.contains("Trunking")).count();

                                    let rows = [
                                        ("CNC DXF Flat Patterns", "assets/dxf/", format!("✓ {} drawings ({} Tray, {} Trunking)", self.landing.scanned_dxf_items.len(), tray_count, trunk_count)),
                                        ("Parametric DSF Templates", "assets/templates/bonn_metal/", "✓ 16 Catalog Presets (Bonn Metal Specs)".to_string()),
                                        ("In-Progress 3D Models", "drawings/cms_in_progress/", "✓ Active SolidCraft Native Drawings".to_string()),
                                        ("Customer Orders CSV", "orders/customer_orders/", "✓ Automated Batch Order Queue".to_string()),
                                    ];

                                    egui::Grid::new("prefs_inventory_grid")
                                        .num_columns(3)
                                        .spacing(Vec2::new(16.0, 8.0))
                                        .striped(true)
                                        .show(ui, |ui| {
                                            for (title, rel_path, status) in rows {
                                                ui.label(RichText::new(title).strong().color(colors.text_primary));
                                                ui.label(RichText::new(rel_path).color(colors.text_secondary));
                                                ui.colored_label(Color32::from_rgb(16, 185, 129), status);
                                                ui.end_row();
                                            }
                                        });
                                }
                                10 => {
                                    // SOLIDCRAFT ENGINE LINK
                                    ui.heading("SolidCraft CAD Engine Link");
                                    ui.label(RichText::new("IPC control channel, core B-rep kernel, and SolidCraft process integration.").size(12.5).color(colors.text_secondary));
                                    ui.separator();
                                    ui.add_space(8.0);

                                    ui.label(RichText::new("▾ PROCESS & VERSION MATRIX").size(11.0).strong().color(colors.text_muted));
                                    ui.add_space(6.0);

                                    egui::Grid::new("prefs_engine_grid")
                                        .num_columns(2)
                                        .spacing(Vec2::new(20.0, 8.0))
                                        .striped(true)
                                        .show(ui, |ui| {
                                            ui.label(RichText::new("Executable Candidate:").strong());
                                            ui.label("target/debug/solidcraft.exe");
                                            ui.end_row();

                                            ui.label(RichText::new("Control Channel Server:").strong());
                                            ui.colored_label(Color32::from_rgb(16, 185, 129), "127.0.0.1:27878 (Listening)");
                                            ui.end_row();

                                            ui.label(RichText::new("Design Format:").strong());
                                            ui.label(format!("Format {} (SolidCraft {})", CURRENT_VERSION.solidcraft_format, CURRENT_VERSION.solidcraft_version));
                                            ui.end_row();

                                            ui.label(RichText::new("Compliance Layer:").strong());
                                            ui.colored_label(Color32::from_rgb(16, 185, 129), "Rule R1 Zero-Touch Additive (tangency-labs/solidcraft)");
                                            ui.end_row();
                                        });

                                    ui.add_space(12.0);
                                    if ui.button(RichText::new(format!("{} Test Launch Standalone SolidCraft CAD", egui_phosphor::regular::ROCKET_LAUNCH)).strong()).clicked() {
                                        self.launch_solidcraft_process(None);
                                    }
                                }
                                _ => {
                                    // TOLERANCE & GAP HEALING
                                    ui.heading("Tolerance & Gap Healing");
                                    ui.label(RichText::new("Tolerance thresholds for 2D CNC laser-cut gap snapping and loop closure.").size(12.5).color(colors.text_secondary));
                                    ui.separator();
                                    ui.add_space(8.0);

                                    ui.label(RichText::new("▾ 2D PRE-FLIGHT GAP SNAP TOLERANCE").size(11.0).strong().color(colors.text_muted));
                                    ui.add_space(6.0);

                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("Default Gap Snap Tolerance:").strong().color(colors.text_primary));
                                        let slider = egui::Slider::new(&mut self.landing.cleaner_state.snap_tolerance_mm, 0.05..=1.50)
                                            .step_by(0.05)
                                            .suffix(" mm");
                                        ui.add(slider);
                                    });

                                    ui.add_space(8.0);
                                    ui.label(RichText::new("Gaps smaller than this distance are automatically snapped and healed to guarantee closed B-rep sheet metal profile wire loops.").size(11.0).color(colors.text_muted));
                                }
                            }
                        });
                    });
                });

                ui.separator();
                ui.add_space(4.0);

                // Bottom Footer Bar (Exact SolidCraft layout: Defaults on left, Cancel / Apply / OK on right)
                ui.horizontal(|ui| {
                    if ui.button("Restore Defaults").clicked() {
                        should_defaults = true;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let ok_btn = egui::Button::new(RichText::new("OK").strong().color(Color32::WHITE))
                            .fill(colors.accent)
                            .corner_radius(5.0)
                            .min_size(Vec2::new(75.0, 28.0));
                        if ui.add(ok_btn).clicked() {
                            should_close = true;
                        }

                        let apply_btn = egui::Button::new(RichText::new("Apply").color(colors.text_primary))
                            .corner_radius(5.0)
                            .min_size(Vec2::new(75.0, 28.0));
                        if ui.add(apply_btn).clicked() {
                            should_apply = true;
                        }

                        let cancel_btn = egui::Button::new(RichText::new("Cancel").color(colors.text_secondary))
                            .corner_radius(5.0)
                            .min_size(Vec2::new(75.0, 28.0));
                        if ui.add(cancel_btn).clicked() {
                            should_close = true;
                        }
                    });
                });
            });

        if should_defaults {
            self.theme = CadenseTheme::SolidCraftCobalt;
            self.font_scale = 1.05;
            self.landing.custom_asset_root = None;
            self.landing.refresh_dxf_library();
            self.temp_custom_asset_root = self.landing.resolved_asset_root.clone();
        }
        if should_apply {
            self.landing.custom_asset_root = Some(self.temp_custom_asset_root.clone());
            self.landing.refresh_dxf_library();
        }

        self.show_settings_modal = is_open && !should_close;
    }

    fn project_3d_point(&self, x: f32, y: f32, z: f32, center: Pos2) -> Pos2 {
        match self.rigging_camera_mode {
            CanvasCameraMode::TopView2D => {
                let sx = center.x + self.rigging_pan.x + x * self.rigging_zoom;
                let sy = center.y + self.rigging_pan.y - y * self.rigging_zoom;
                Pos2::new(sx, sy)
            }
            CanvasCameraMode::Perspective3D => {
                let yaw = self.rigging_orbit_yaw;
                let pitch = self.rigging_orbit_pitch;
                let cos_y = yaw.cos();
                let sin_y = yaw.sin();
                let x1 = x * cos_y - y * sin_y;
                let y1 = x * sin_y + y * cos_y;
                let z1 = z;

                let cos_p = pitch.cos();
                let sin_p = pitch.sin();
                let x2 = x1;
                let y2 = y1 * cos_p - z1 * sin_p;
                let depth = y1 * sin_p + z1 * cos_p;
                let fov_scale = (1.0 + depth * 0.0006).clamp(0.7, 1.4);

                let sx = center.x + self.rigging_pan.x + x2 * self.rigging_zoom * fov_scale;
                let sy = center.y + self.rigging_pan.y - y2 * self.rigging_zoom * fov_scale;
                Pos2::new(sx, sy)
            }
        }
    }

    fn inverse_project_to_plane(&self, screen_pos: Pos2, center: Pos2) -> [f32; 2] {
        match self.rigging_camera_mode {
            CanvasCameraMode::TopView2D => {
                let x = (screen_pos.x - (center.x + self.rigging_pan.x)) / self.rigging_zoom;
                let y = -(screen_pos.y - (center.y + self.rigging_pan.y)) / self.rigging_zoom;
                [x, y]
            }
            CanvasCameraMode::Perspective3D => {
                let yaw = self.rigging_orbit_yaw;
                let pitch = self.rigging_orbit_pitch.max(0.1);
                let cos_p = pitch.cos();
                let x2 = (screen_pos.x - (center.x + self.rigging_pan.x)) / self.rigging_zoom;
                let y2 = -(screen_pos.y - (center.y + self.rigging_pan.y)) / self.rigging_zoom;
                let y1 = if cos_p.abs() > 0.05 { y2 / cos_p } else { y2 };
                let x1 = x2;
                let cos_y = yaw.cos();
                let sin_y = yaw.sin();
                let x = x1 * cos_y + y1 * sin_y;
                let y = -x1 * sin_y + y1 * cos_y;
                [x, y]
            }
        }
    }

    /// Computes both the 3D folded position and the 3D surface normal vector for a 2D sheet blank point.
    /// Strictly limits folding to the extent of the bend line segment, preventing adjacent webs or side wings from warping.
    fn compute_folded_3d_vertex_and_normal(&self, orig: [f32; 2]) -> ([f32; 3], [f32; 3]) {
        solid_renderer::evaluate_chained_fold_point_and_normal(&self.rigged_sheet, self.rigging_fold_slider_deg, orig, None)
    }
    /// Computes folded 3D coordinates for a slot / perforation vertex using the slot's center point.
    /// This prevents slots from being torn apart or stretched into spikes across bend boundaries.
    #[allow(dead_code)]
    fn compute_folded_slot_vertex_and_normal(&self, orig: [f32; 2], slot_center: [f32; 2]) -> ([f32; 3], [f32; 3]) {
        solid_renderer::evaluate_chained_fold_point_and_normal(&self.rigged_sheet, self.rigging_fold_slider_deg, orig, Some(slot_center))
    }

    #[allow(dead_code)]
    fn compute_folded_3d_vertex(&self, orig: [f32; 2]) -> [f32; 3] {
        self.compute_folded_3d_vertex_and_normal(orig).0
    }
    fn find_snap_vertex(&self, mouse_pos: Pos2, center: Pos2, snap_pixel_tol: f32) -> Option<([f32; 2], &'static str)> {
        let mut closest: Option<([f32; 2], &'static str, f32)> = None;

        if let Some(geom) = &self.active_dxf_geometry {
            for (p1, p2, kind) in &geom.lines {
                let label = match kind {
                    crate::landing::DxfEntityKind::ReliefCut => "Relief Notch",
                    crate::landing::DxfEntityKind::Outline => "Blank Perimeter",
                    crate::landing::DxfEntityKind::SlotOrPerforation => "Coupler Slot / Hole",
                    _ => "Drawing Vertex",
                };

                for pt in [*p1, *p2] {
                    let (f3d, _) = self.compute_folded_3d_vertex_and_normal(pt);
                    let screen_pt = self.project_3d_point(f3d[0], f3d[1], f3d[2], center);
                    let d = screen_pt.distance(mouse_pos);
                    if d <= snap_pixel_tol {
                        match closest {
                            Some((_, _, best_d)) if d >= best_d => {}
                            _ => closest = Some((pt, label, d)),
                        }
                    }
                }
            }
        }

        closest.map(|(pt, label, _)| (pt, label))
    }
    /// Dedicated 3D rendering pipeline for the folded sheet metal part.
    /// Can be rendered either in the main canvas (3D Perspective Mode) or in the separate floating 3D window.
    fn paint_3d_sheet_metal(
        &self,
        painter: &egui::Painter,
        center: Pos2,
        pan: Vec2,
        zoom: f32,
        orbit_yaw: f32,
        orbit_pitch: f32,
        colors: &ThemeColors,
    ) {
        if let Some(geom) = &self.active_dxf_geometry {
            let cos_y = orbit_yaw.cos();
            let sin_y = orbit_yaw.sin();
            let cos_p = orbit_pitch.cos();
            let sin_p = orbit_pitch.sin();
            let view_dir = [-sin_y * cos_p, cos_y * cos_p, sin_p];

            let solid_model = solid_renderer::CompiledSolidSheetMetal::build(geom, &self.rigged_sheet, self.rigging_fold_slider_deg, view_dir);

            solid_model.paint(painter, center, pan, zoom, orbit_yaw, orbit_pitch, self.rigging_show_floor_blueprint, colors);
        } else {
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                "No DXF Flat Pattern Loaded.\nOpen a drawing in Stage 1 or select a CMS preset.",
                egui::FontId::proportional(14.0),
                colors.text_muted,
            );
        }
    }

    /// Renders the floating, resizable 3D Viewport window actively linked to the 2D CAD flat drawing.
    fn render_floating_3d_viewport_window(&mut self, ctx: &egui::Context, colors: &ThemeColors) {
        if !self.show_3d_viewport_window || self.active_tab != StudioWorkspaceTab::CmsTruFoldWorkbench {
            return;
        }

        let mut is_open = self.show_3d_viewport_window;
        let sheet_t = self.rigged_sheet.material_rule.thickness_mm as f32;

        egui::Window::new("🧊 CMS 3D TruFold Viewport")
            .open(&mut is_open)
            .default_size([580.0, 500.0])
            .default_pos([720.0, 140.0])
            .resizable(true)
            .collapsible(true)
            .show(ctx, |ui| {
                // Toolbar in 3D floating window
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Camera:").size(11.5).strong().color(colors.text_muted));
                    if ui
                        .button(RichText::new(format!("{} Iso", egui_phosphor::regular::CUBE)).size(11.0))
                        .on_hover_text("Isometric 3D perspective")
                        .clicked()
                    {
                        self.floating_orbit_yaw = 0.60;
                        self.floating_orbit_pitch = 0.55;
                        self.floating_pan = Vec2::ZERO;
                    }
                    if ui
                        .button(RichText::new(format!("{} Top", egui_phosphor::regular::ARROW_FAT_UP)).size(11.0))
                        .on_hover_text("Top-down 3D view")
                        .clicked()
                    {
                        self.floating_orbit_yaw = 0.0;
                        self.floating_orbit_pitch = 1.55;
                        self.floating_pan = Vec2::ZERO;
                    }
                    if ui
                        .button(RichText::new(format!("{} Front", egui_phosphor::regular::ARROW_FAT_RIGHT)).size(11.0))
                        .on_hover_text("Front 3D view")
                        .clicked()
                    {
                        self.floating_orbit_yaw = 0.0;
                        self.floating_orbit_pitch = 0.05;
                        self.floating_pan = Vec2::ZERO;
                    }
                    if ui
                        .button(RichText::new(format!("{} Fit", egui_phosphor::regular::CROSSHAIR)).size(11.0))
                        .on_hover_text("Center 3D model")
                        .clicked()
                    {
                        self.floating_pan = Vec2::ZERO;
                        self.floating_zoom = 1.0;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(format!("Gauge: {:.1}mm", sheet_t)).size(11.0).strong().color(Color32::from_rgb(56, 189, 248)));
                    });
                });

                ui.separator();

                // Fold Sync Slider & Presets
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Fold:").size(12.0).strong().color(colors.text_secondary));
                    let slider = egui::Slider::new(&mut self.rigging_fold_slider_deg, 0.0..=90.0).suffix("°");
                    ui.add_sized([120.0, 16.0], slider);

                    if ui.button("0° Flat").clicked() {
                        self.rigging_fold_slider_deg = 0.0;
                    }
                    if ui.button("45° Formed").clicked() {
                        self.rigging_fold_slider_deg = 45.0;
                    }
                    if ui.button("90° Folded").clicked() {
                        self.rigging_fold_slider_deg = 90.0;
                    }
                });

                ui.separator();

                // 3D Canvas
                let avail = ui.available_size();
                let canvas_size = Vec2::new(avail.x.max(260.0), avail.y.max(240.0));
                let (rect, response) = ui.allocate_exact_size(canvas_size, egui::Sense::click_and_drag());
                let painter = ui.painter_at(rect);
                let center = rect.center();

                painter.rect_filled(rect, 4.0, colors.canvas_bg);
                painter.rect_stroke(rect, 4.0, Stroke::new(1.0, colors.border_subtle), StrokeKind::Inside);

                // Mouse controls for floating 3D window
                if response.hovered() {
                    let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
                    if scroll_delta.abs() > 0.01 {
                        let factor = 1.0 + (scroll_delta * 0.002).clamp(-0.3, 0.3);
                        self.floating_zoom = (self.floating_zoom * factor).clamp(0.2, 10.0);
                    }
                }

                if response.dragged() {
                    let delta = response.drag_delta();
                    let is_pan = ui.input(|i| {
                        i.pointer.button_down(egui::PointerButton::Secondary) || i.pointer.button_down(egui::PointerButton::Middle) || i.modifiers.alt
                    });
                    if is_pan {
                        self.floating_pan += delta;
                    } else {
                        self.floating_orbit_yaw += delta.x * 0.01;
                        self.floating_orbit_pitch = (self.floating_orbit_pitch - delta.y * 0.01).clamp(0.05, 1.55);
                    }
                }

                self.paint_3d_sheet_metal(
                    &painter,
                    center,
                    self.floating_pan,
                    self.floating_zoom,
                    self.floating_orbit_yaw,
                    self.floating_orbit_pitch,
                    colors,
                );
            });

        self.show_3d_viewport_window = is_open;
    }

    fn render_interactive_canvas(&mut self, ui: &mut egui::Ui, size: Vec2, colors: &ThemeColors) {
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
        let painter = ui.painter_at(rect);
        let center = rect.center();

        // 1. Blueprint Canvas Background & Border
        painter.rect_filled(rect, 6.0, colors.canvas_bg);
        painter.rect_stroke(rect, 6.0, Stroke::new(1.0, colors.border_subtle), StrokeKind::Inside);

        let is_hovered = response.hovered();

        // 2. Mouse Scroll Zoom
        if is_hovered {
            let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll_delta.abs() > 0.01 {
                let factor = 1.0 + (scroll_delta * 0.002).clamp(-0.3, 0.3);
                self.rigging_zoom = (self.rigging_zoom * factor).clamp(0.15, 20.0);
            }
        }

        // 3. Mouse Pan / Orbit Drag (Accurate 1:1 screen pixel tracking, non-inverted!)
        if response.dragged() {
            let drag_delta = response.drag_delta();
            let is_sec_or_alt = ui.input(|i| i.pointer.button_down(egui::PointerButton::Secondary) || i.modifiers.alt);
            if is_sec_or_alt {
                self.rigging_pan += drag_delta;
            } else if self.rigging_camera_mode == CanvasCameraMode::Perspective3D && self.rigging_tool_mode == RiggingToolMode::Select {
                self.rigging_orbit_yaw += drag_delta.x * 0.01;
                self.rigging_orbit_pitch = (self.rigging_orbit_pitch - drag_delta.y * 0.01).clamp(0.05, 1.55);
            } else {
                self.rigging_pan += drag_delta;
            }
        }

        // 4. Magnetic Snapping & Click Handling
        let mut snap_info = None;
        if is_hovered {
            if let Some(mouse_pos) = response.hover_pos() {
                snap_info = self.find_snap_vertex(mouse_pos, center, 16.0);

                if response.clicked() {
                    let world_pt = if let Some((snap_pt, _)) = snap_info { snap_pt } else { self.inverse_project_to_plane(mouse_pos, center) };

                    if self.rigging_tool_mode == RiggingToolMode::AddBendLine {
                        if let Some(p1) = self.rigging_pending_p1 {
                            let p2 = [world_pt[0] as f64, world_pt[1] as f64];
                            let dx = p2[0] - p1[0];
                            let dy = p2[1] - p1[1];
                            let dist = (dx * dx + dy * dy).sqrt();
                            if dist > 2.0 {
                                self.push_trufold_undo();
                                self.rigged_sheet.add_bend(p1, p2, 90.0, BendDirection::Up);
                                self.solidcraft_status = Some(format!("✓ Added Bend Line B{} [L: {:.1}mm]", self.rigged_sheet.bends.len(), dist));
                            }
                            self.rigging_pending_p1 = None;
                        } else {
                            self.rigging_pending_p1 = Some([world_pt[0] as f64, world_pt[1] as f64]);
                        }
                    }
                }
            }
        }

        // Dispatch based on Camera Mode:
        if self.rigging_camera_mode == CanvasCameraMode::Perspective3D {
            self.paint_3d_sheet_metal(
                &painter,
                center,
                self.rigging_pan,
                self.rigging_zoom,
                self.rigging_orbit_yaw,
                self.rigging_orbit_pitch,
                colors,
            );
        } else {
            // === 2D CAD FLAT PATTERN VIEWPORT ===
            let to_screen = |x: f32, y: f32| -> Pos2 {
                Pos2::new(center.x + (x * self.rigging_zoom) + self.rigging_pan.x, center.y - (y * self.rigging_zoom) + self.rigging_pan.y)
            };

            // 1. Origin Axes at (0, 0)
            let o_s = to_screen(0.0, 0.0);
            let x_s = to_screen(40.0, 0.0);
            let y_s = to_screen(0.0, 40.0);
            painter.line_segment([o_s, x_s], Stroke::new(2.0, Color32::from_rgba_unmultiplied(239, 68, 68, 200)));
            painter.line_segment([o_s, y_s], Stroke::new(2.0, Color32::from_rgba_unmultiplied(16, 185, 129, 200)));

            if let Some(geom) = &self.active_dxf_geometry {
                // Lines
                for (p1, p2, kind) in &geom.lines {
                    let s1 = to_screen(p1[0], p1[1]);
                    let s2 = to_screen(p2[0], p2[1]);

                    let stroke = match kind {
                        crate::landing::DxfEntityKind::Outline => Stroke::new(2.0, colors.accent),
                        crate::landing::DxfEntityKind::ReliefCut => Stroke::new(1.8, Color32::from_rgb(239, 68, 68)),
                        crate::landing::DxfEntityKind::SlotOrPerforation => Stroke::new(1.6, Color32::from_rgb(16, 185, 129)),
                        _ => Stroke::new(1.2, colors.text_secondary),
                    };
                    painter.line_segment([s1, s2], stroke);
                }

                // Arcs with corrected counter-clockwise sweep
                for (arc_c, r, a_start, a_end, kind) in &geom.arcs {
                    let stroke = match kind {
                        crate::landing::DxfEntityKind::ReliefCut => Stroke::new(1.8, Color32::from_rgb(239, 68, 68)),
                        crate::landing::DxfEntityKind::SlotOrPerforation => Stroke::new(1.6, Color32::from_rgb(16, 185, 129)),
                        _ => Stroke::new(2.0, colors.accent),
                    };

                    let steps = 14;
                    let mut prev_pt = None;
                    let mut sweep = a_end - a_start;
                    if sweep <= 0.0 {
                        sweep += 360.0;
                    }

                    for s in 0..=steps {
                        let t = s as f32 / steps as f32;
                        let ang = (a_start + t * sweep).to_radians();
                        let pt = [arc_c[0] + r * ang.cos(), arc_c[1] + r * ang.sin()];
                        let s_cur = to_screen(pt[0], pt[1]);
                        if let Some(s_prev) = prev_pt {
                            painter.line_segment([s_prev, s_cur], stroke);
                        }
                        prev_pt = Some(s_cur);
                    }
                }

                // Bend lines with badges
                for bend in &self.rigged_sheet.bends {
                    let p1 = [bend.start_mm[0] as f32, bend.start_mm[1] as f32];
                    let p2 = [bend.end_mm[0] as f32, bend.end_mm[1] as f32];
                    let bs1 = to_screen(p1[0], p1[1]);
                    let bs2 = to_screen(p2[0], p2[1]);

                    painter.line_segment([bs1, bs2], Stroke::new(2.4, Color32::from_rgb(245, 158, 11)));
                    let mid_pos = Pos2::new((bs1.x + bs2.x) * 0.5, (bs1.y + bs2.y) * 0.5);
                    let badge_text =
                        format!("#{} {} {:.0}°", bend.sequence, if bend.direction == BendDirection::Up { "UP" } else { "DN" }, bend.angle_deg);
                    let badge_rect = Rect::from_center_size(mid_pos, Vec2::new(76.0, 18.0));
                    painter.rect_filled(badge_rect, 4.0, Color32::from_rgba_unmultiplied(245, 158, 11, 230));
                    painter.text(mid_pos, egui::Align2::CENTER_CENTER, badge_text, egui::FontId::monospace(11.0), Color32::BLACK);
                }
            } else {
                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    "No DXF Flat Pattern Loaded.\nOpen a drawing in Stage 1 or select a CMS preset.",
                    egui::FontId::proportional(14.0),
                    colors.text_muted,
                );
            }
        }

        // Rubber-Band Bend Line Creation Preview
        if let Some(p1) = self.rigging_pending_p1 {
            let s1 = self.project_3d_point(p1[0] as f32, p1[1] as f32, 0.0, center);
            let cur_screen = if let Some((snap_pt, _)) = snap_info {
                self.project_3d_point(snap_pt[0], snap_pt[1], 0.0, center)
            } else if let Some(m_pos) = response.hover_pos() {
                m_pos
            } else {
                s1
            };
            painter.line_segment([s1, cur_screen], Stroke::new(2.2, Color32::from_rgb(234, 179, 8)));
            painter.circle_filled(s1, 4.0, Color32::from_rgb(234, 179, 8));
            painter.circle_filled(cur_screen, 4.0, Color32::from_rgb(234, 179, 8));
        }

        // Magnetic Snapping Glowing Ring & Tooltip
        if let Some((snap_pt, snap_lbl)) = snap_info {
            let snap_screen = self.project_3d_point(snap_pt[0], snap_pt[1], 0.0, center);
            painter.circle_stroke(snap_screen, 8.5, Stroke::new(2.0, Color32::from_rgb(16, 185, 129)));
            painter.circle_filled(snap_screen, 3.0, Color32::from_rgb(16, 185, 129));

            let tip_pos = Pos2::new(snap_screen.x + 14.0, snap_screen.y - 12.0);
            let tip_str = format!("📍 {} [{:.1}, {:.1}]", snap_lbl, snap_pt[0], snap_pt[1]);
            let tip_box = Rect::from_min_size(tip_pos, Vec2::new(tip_str.len() as f32 * 6.8 + 12.0, 18.0));
            painter.rect_filled(tip_box, 3.0, Color32::from_rgba_unmultiplied(15, 23, 42, 230));
            painter.rect_stroke(tip_box, 3.0, Stroke::new(1.0, Color32::from_rgb(16, 185, 129)), StrokeKind::Middle);
            painter.text(
                Pos2::new(tip_pos.x + 6.0, tip_pos.y + 9.0),
                egui::Align2::LEFT_CENTER,
                tip_str,
                egui::FontId::proportional(10.5),
                Color32::from_rgb(52, 211, 153),
            );
        }

        // Viewport HUD Footer Info Bar
        let hud_rect = Rect::from_min_size(Pos2::new(rect.min.x + 12.0, rect.max.y - 30.0), Vec2::new(450.0, 20.0));
        painter.rect_filled(hud_rect, 4.0, Color32::from_rgba_unmultiplied(15, 23, 42, 180));
        let mode_str = match self.rigging_camera_mode {
            CanvasCameraMode::TopView2D => "Camera: 2D Flat CAD View",
            CanvasCameraMode::Perspective3D => "Camera: 3D Perspective Orbit",
        };
        let fold_str =
            if self.rigging_fold_slider_deg > 0.0 { format!("Fold: {:.0}°", self.rigging_fold_slider_deg) } else { "Flat Pattern".to_string() };
        painter.text(
            Pos2::new(hud_rect.min.x + 8.0, hud_rect.center().y),
            egui::Align2::LEFT_CENTER,
            format!("{} | {} | Drag to Pan/Orbit | Scroll to Zoom", mode_str, fold_str),
            egui::FontId::proportional(10.0),
            Color32::from_rgb(148, 163, 184),
        );
    }

    fn render_bend_sequence_inspector(&mut self, ui: &mut egui::Ui, width: f32, colors: &ThemeColors) {
        ui.allocate_ui_with_layout(Vec2::new(width, ui.available_height()), egui::Layout::top_down(egui::Align::Min), |ui| {
            Frame::NONE.fill(colors.bg_card).stroke(Stroke::new(1.0, colors.border_subtle)).inner_margin(Margin::same(12)).corner_radius(6.0).show(
                ui,
                |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Bending Sequence").strong().size(13.5).color(colors.text_primary));
                        ui.colored_label(Color32::from_rgb(16, 185, 129), format!("({} Bends)", self.rigged_sheet.bends.len()));
                    });
                    ui.separator();

                    egui::ScrollArea::vertical().max_height(ui.available_height() - 120.0).show(ui, |ui| {
                        if self.rigged_sheet.bends.is_empty() {
                            ui.vertical_centered(|ui| {
                                ui.add_space(20.0);
                                ui.label(RichText::new("No bend lines defined.").italics().color(colors.text_muted));
                                ui.label(RichText::new("Click ⚡ Add Bend Line to begin.").size(11.0).color(colors.text_muted));
                            });
                        }

                        let mut to_delete = None;
                        let mut to_toggle_dir = None;
                        let mut to_move_up = None;

                        for (idx, bend) in self.rigged_sheet.bends.iter_mut().enumerate() {
                            Frame::NONE
                                .fill(colors.bg_card_alt)
                                .stroke(Stroke::new(1.0, colors.border_subtle))
                                .inner_margin(Margin::same(8))
                                .corner_radius(4.0)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        // Clean sequence number badge
                                        let seq_text = format!("#{}", bend.sequence);
                                        Frame::NONE.fill(colors.accent_subtle).corner_radius(4.0).inner_margin(Margin::symmetric(6, 2)).show(
                                            ui,
                                            |ui| {
                                                ui.label(RichText::new(seq_text).size(12.5).strong().color(colors.accent));
                                            },
                                        );

                                        ui.label(RichText::new(bend.id.to_string()).size(13.0).strong().color(colors.text_primary));

                                        let (dir_icon, dir_lbl, dir_color) = if bend.direction == BendDirection::Up {
                                            (egui_phosphor::regular::ARROW_FAT_UP, "UP", Color32::from_rgb(16, 185, 129))
                                        } else {
                                            (egui_phosphor::regular::ARROW_FAT_DOWN, "DOWN", Color32::from_rgb(245, 158, 11))
                                        };

                                        if ui
                                            .button(RichText::new(format!("{} {}", dir_icon, dir_lbl)).size(11.5).strong().color(dir_color))
                                            .on_hover_text("Toggle UP / DOWN fold direction")
                                            .clicked()
                                        {
                                            to_toggle_dir = Some(bend.id.clone());
                                        }

                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            let del_btn = egui::Button::new(
                                                RichText::new(egui_phosphor::regular::TRASH).size(13.0).color(Color32::from_rgb(239, 68, 68)),
                                            )
                                            .fill(Color32::from_rgba_unmultiplied(239, 68, 68, 25))
                                            .corner_radius(4.0);
                                            if ui.add(del_btn).on_hover_text("Delete bend line").clicked() {
                                                to_delete = Some(bend.id.clone());
                                            }
                                            if idx > 0 {
                                                let up_btn = egui::Button::new(
                                                    RichText::new(egui_phosphor::regular::ARROW_UP).size(13.0).color(colors.text_secondary),
                                                )
                                                .fill(colors.bg_card)
                                                .corner_radius(4.0);
                                                if ui.add(up_btn).on_hover_text("Move up in sequence").clicked() {
                                                    to_move_up = Some(idx);
                                                }
                                            }
                                        });
                                    });

                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("Angle:").size(11.0).color(colors.text_muted));
                                        let mut angle = bend.angle_deg;
                                        if ui.add_sized([100.0, 18.0], egui::Slider::new(&mut angle, 0.0..=180.0).suffix("°")).changed() {
                                            bend.angle_deg = angle;
                                        }
                                        ui.label(RichText::new(format!("L: {:.1}mm", bend.length_mm())).size(10.5).color(colors.text_secondary));
                                    });

                                    ui.label(
                                        RichText::new(format!(
                                            "[{:.1}, {:.1}] → [{:.1}, {:.1}]",
                                            bend.start_mm[0], bend.start_mm[1], bend.end_mm[0], bend.end_mm[1]
                                        ))
                                        .size(9.5)
                                        .color(colors.text_muted),
                                    );
                                });
                            ui.add_space(4.0);
                        }

                        if let Some(id) = to_delete {
                            self.push_trufold_undo();
                            self.rigged_sheet.remove_bend(&id);
                        }
                        if let Some(id) = to_toggle_dir {
                            self.push_trufold_undo();
                            self.rigged_sheet.toggle_bend_direction(&id);
                        }
                        if let Some(idx) = to_move_up {
                            self.push_trufold_undo();
                            self.rigged_sheet.bends.swap(idx, idx - 1);
                            for (i, b) in self.rigged_sheet.bends.iter_mut().enumerate() {
                                b.sequence = i + 1;
                            }
                        }
                    });

                    ui.separator();

                    // Press-Brake Tooling Specs Card
                    Frame::NONE
                        .fill(colors.bg_card_alt)
                        .stroke(Stroke::new(1.0, colors.border_subtle))
                        .inner_margin(Margin::same(8))
                        .corner_radius(4.0)
                        .show(ui, |ui| {
                            ui.label(RichText::new("Press-Brake Tooling (DIN 6935)").size(11.0).strong().color(colors.text_primary));
                            ui.label(RichText::new("• Punch: Standard Straight R1.5").size(10.0).color(colors.text_secondary));
                            let v_die = (self.rigged_sheet.material_rule.thickness_mm * 6.5).round();
                            ui.label(RichText::new(format!("• V-Die Opening: {:.0} mm (88°)", v_die)).size(10.0).color(colors.text_secondary));
                            ui.label(RichText::new("• Backgauge: Automatic 2-Finger Z-Stop").size(10.0).color(colors.text_secondary));
                        });
                },
            );
        });
    }

    fn render_unified_bending_workbench(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
        // --- SECTION 0: DOCUMENT TAB STRIP WITH [ ← Back to Dashboard ], MULTI-TABS, AND CLOSE [×] ---
        Frame::NONE
            .fill(if colors.is_sidebar_blue() { Color32::from_rgb(24, 28, 36) } else { colors.bg_sidebar })
            .stroke(Stroke::new(1.0, colors.border_subtle))
            .inner_margin(Margin::symmetric(10, 6))
            .corner_radius(6.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // 1. Dedicated Back to Dashboard button (User Request 3)
                    let back_btn = egui::Button::new(
                        RichText::new(format!("{} Back to Dashboard", egui_phosphor::regular::ARROW_LEFT)).size(12.0).strong().color(colors.accent),
                    )
                    .fill(colors.accent_subtle)
                    .corner_radius(4.0);
                    if ui.add(back_btn).on_hover_text("Return to Start / Dashboard (tabs remain open)").clicked() {
                        self.active_tab = StudioWorkspaceTab::Dashboard;
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // 2. Open Document Tabs (User Request 3)
                    let mut to_close_idx = None;
                    let mut to_switch_idx = None;

                    for (idx, tab) in self.trufold_tabs.iter().enumerate() {
                        let is_active = idx == self.active_trufold_tab_idx;
                        let tab_bg = if is_active { colors.bg_card } else { Color32::from_rgba_unmultiplied(255, 255, 255, 8) };
                        let border_stroke = if is_active { Stroke::new(1.0, colors.accent) } else { Stroke::new(1.0, colors.border_subtle) };

                        Frame::NONE.fill(tab_bg).stroke(border_stroke).inner_margin(Margin::symmetric(8, 4)).corner_radius(4.0).show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 4.0;
                                let icon = egui_phosphor::regular::FILE_CODE;
                                ui.label(RichText::new(icon).size(12.0).color(if is_active { colors.accent } else { colors.text_muted }));

                                let tab_label = ui.selectable_label(is_active, RichText::new(&tab.title).size(12.0).strong());
                                if tab_label.clicked() && !is_active {
                                    to_switch_idx = Some(idx);
                                }

                                // Close button [×] for each tab
                                let close_btn = ui.add(
                                    egui::Button::new(RichText::new(egui_phosphor::regular::X).size(11.0).color(colors.text_muted)).frame(false),
                                );
                                if close_btn.on_hover_text("Close this drawing tab").clicked() {
                                    to_close_idx = Some(idx);
                                }
                            });
                        });

                        ui.add_space(4.0);
                    }

                    if let Some(idx) = to_switch_idx {
                        self.switch_trufold_tab(idx);
                    }
                    if let Some(idx) = to_close_idx {
                        self.close_trufold_tab(idx);
                    }

                    // 3. [+] Button to open new drawing tab from assets/dxf
                    egui::ComboBox::from_id_salt("open_tab_selector")
                        .selected_text(RichText::new(format!("{} +", egui_phosphor::regular::PLUS)).size(12.0).strong().color(colors.accent))
                        .show_ui(ui, |ui| {
                            ui.label(RichText::new("Open Drawing in new TruFold Tab:").size(11.0).strong().color(colors.text_muted));
                            ui.separator();
                            let samples = [
                                ("90_bend_100x50.dxf", "assets/dxf/tray_accessories/90_bend_100x50.dxf"),
                                ("45_deg_inside_riser_100x100.dxf", "assets/dxf/trunking_accessories/45_deg_inside_riser_100x100.dxf"),
                                ("45_deg_bend_100x50.dxf", "assets/dxf/tray_accessories/45_deg_bend_100x50.dxf"),
                                ("90_deg_bend_100x50.dxf", "assets/dxf/tray_accessories/90_deg_bend_100x50.dxf"),
                                ("equal_tee_100x50.dxf", "assets/dxf/tray_accessories/equal_tee_100x50.dxf"),
                                ("equal_cross_100x100.dxf", "assets/dxf/trunking_accessories/equal_cross_100x100.dxf"),
                            ];
                            for (name, path) in samples {
                                if ui.button(format!("📄 {}", name)).clicked() {
                                    self.open_drawing_in_trufold(path);
                                }
                            }
                        });
                });
            });

        ui.add_space(6.0);

        // --- SECTION 1: PRODUCT METADATA & DIN 6935 RULES HEADER ---
        Frame::NONE
            .fill(colors.bg_card)
            .stroke(Stroke::new(1.0, colors.border_subtle))
            .inner_margin(Margin::symmetric(14, 10))
            .corner_radius(6.0)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("CMS Product Category:").size(12.5).strong().color(colors.text_secondary));
                    let prev_cat = self.rigged_sheet.category;
                    egui::ComboBox::from_id_salt("cat_selector")
                        .selected_text(match self.rigged_sheet.category {
                            CmsCategory::CableTrunking => format!("{} Cable Trunking", egui_phosphor::regular::PACKAGE),
                            CmsCategory::CableTray => format!("{} Cable Tray", egui_phosphor::regular::LADDER),
                            CmsCategory::UnistrutChannel => format!("{} Unistrut Channel", egui_phosphor::regular::COLUMNS),
                            CmsCategory::AccessoryCouplerOrEndCap => format!("{} Fittings & Couplers", egui_phosphor::regular::LINK),
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.rigged_sheet.category,
                                CmsCategory::CableTrunking,
                                format!("{} Cable Trunking", egui_phosphor::regular::PACKAGE),
                            );
                            ui.selectable_value(
                                &mut self.rigged_sheet.category,
                                CmsCategory::CableTray,
                                format!("{} Cable Tray", egui_phosphor::regular::LADDER),
                            );
                            ui.selectable_value(
                                &mut self.rigged_sheet.category,
                                CmsCategory::UnistrutChannel,
                                format!("{} Unistrut Channel", egui_phosphor::regular::COLUMNS),
                            );
                            ui.selectable_value(
                                &mut self.rigged_sheet.category,
                                CmsCategory::AccessoryCouplerOrEndCap,
                                format!("{} Fittings & Couplers", egui_phosphor::regular::LINK),
                            );
                        });

                    let cat_matches = |dsf: CmsCategory, ord: cadense_order::CmsCategory| -> bool {
                        match dsf {
                            CmsCategory::CableTrunking => {
                                matches!(ord, cadense_order::CmsCategory::CableTrunking | cadense_order::CmsCategory::TrunkingAccessories)
                            }
                            CmsCategory::CableTray => {
                                matches!(ord, cadense_order::CmsCategory::CableTray | cadense_order::CmsCategory::TrayAccessories)
                            }
                            CmsCategory::UnistrutChannel => matches!(ord, cadense_order::CmsCategory::SupportChannels),
                            CmsCategory::AccessoryCouplerOrEndCap => {
                                matches!(ord, cadense_order::CmsCategory::TrayAccessories | cadense_order::CmsCategory::TrunkingAccessories)
                            }
                        }
                    };

                    if self.rigged_sheet.category != prev_cat {
                        self.push_trufold_undo();
                        // Pick first product of new category
                        if let Some(first_match) =
                            self.cms_ext.registry.entries().iter().find(|e| cat_matches(self.rigged_sheet.category, e.category))
                        {
                            self.rigged_sheet.product_name = first_match.variation_name.clone();
                            self.rigged_sheet.product_code = format!("BONN-{}", first_match.catalog_code.as_str());
                            self.rigged_sheet.material_rule.thickness_mm = first_match.default_thickness_mm;
                        }
                    }

                    ui.separator();

                    // Product Name dropdown populated from product_variations.csv (Item I)
                    ui.label(RichText::new("Product Name:").size(12.5).strong().color(colors.text_secondary));
                    let current_name = if self.rigged_sheet.product_name.is_empty() {
                        "Select CMS Product...".to_string()
                    } else {
                        self.rigged_sheet.product_name.clone()
                    };

                    egui::ComboBox::from_id_salt("product_name_selector").selected_text(RichText::new(current_name).strong()).width(280.0).show_ui(
                        ui,
                        |ui| {
                            let matching: Vec<(String, String, f64, bool)> = self
                                .cms_ext
                                .registry
                                .entries()
                                .iter()
                                .filter(|e| cat_matches(self.rigged_sheet.category, e.category))
                                .map(|e| {
                                    (
                                        e.variation_name.clone(),
                                        e.catalog_code.as_str().to_string(),
                                        e.default_thickness_mm,
                                        self.rigged_sheet.product_name == e.variation_name,
                                    )
                                })
                                .collect();

                            for (v_name, c_code, def_t, is_sel) in matching {
                                let label = format!("{} ({})", v_name, c_code);
                                if ui.selectable_label(is_sel, label).clicked() {
                                    self.push_trufold_undo();
                                    self.rigged_sheet.product_name = v_name;
                                    self.rigged_sheet.product_code = format!("BONN-{}", c_code);
                                    self.rigged_sheet.material_rule.thickness_mm = def_t;
                                }
                            }
                        },
                    );

                    ui.label(RichText::new("SKU Code:").size(12.5).strong().color(colors.text_secondary));
                    ui.add(egui::TextEdit::singleline(&mut self.rigged_sheet.product_code).desired_width(170.0));

                    ui.separator();

                    ui.label(RichText::new("Sheet Gauge:").size(12.5).strong().color(colors.text_secondary));
                    let curr_t = self.rigged_sheet.material_rule.thickness_mm;
                    egui::ComboBox::from_id_salt("gauge_selector")
                        .selected_text(format!("{:.1} mm ({})", curr_t, if curr_t >= 2.0 { "HDG" } else { "GI" }))
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(curr_t == 1.2, "1.2 mm - Pre-Galvanized").clicked() {
                                self.push_trufold_undo();
                                self.rigged_sheet.material_rule = SheetMaterialRule::bonn_metal_1_2mm();
                            }
                            if ui.selectable_label(curr_t == 1.5, "1.5 mm - Pre-Galvanized (Standard)").clicked() {
                                self.push_trufold_undo();
                                self.rigged_sheet.material_rule = SheetMaterialRule::bonn_metal_1_5mm();
                            }
                            if ui.selectable_label(curr_t == 2.0, "2.0 mm - Hot Dip Galvanized").clicked() {
                                self.push_trufold_undo();
                                self.rigged_sheet.material_rule = SheetMaterialRule::bonn_metal_2_0mm();
                            }
                        });

                    let ba = self.rigged_sheet.material_rule.bend_allowance(90.0);
                    let bd = self.rigged_sheet.material_rule.bend_deduction(90.0);
                    Frame::NONE
                        .fill(Color32::from_rgba_unmultiplied(16, 185, 129, 24))
                        .stroke(Stroke::new(1.0, Color32::from_rgba_unmultiplied(16, 185, 129, 90)))
                        .inner_margin(Margin::symmetric(9, 4))
                        .corner_radius(4.0)
                        .show(ui, |ui| {
                            ui.colored_label(
                                Color32::from_rgb(16, 185, 129),
                                format!(
                                    "DIN 6935: t={:.1}mm | K={:.2} | 90° BA={:.2}mm | BD={:.2}mm",
                                    self.rigged_sheet.material_rule.thickness_mm, self.rigged_sheet.material_rule.k_factor, ba, bd
                                ),
                            );
                        });
                });
            });

        ui.add_space(6.0);

        // --- SECTION 2: CLEAN TWO-ROW WORKBENCH TOOLBAR (Non-overlapping) ---
        // ROW 1: Viewport Camera, Navigation & Tool Modes
        Frame::NONE
            .fill(colors.bg_card)
            .stroke(Stroke::new(1.0, colors.border_subtle))
            .inner_margin(Margin::symmetric(12, 6))
            .corner_radius(6.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;

                    // Camera toggle
                    ui.label(RichText::new("Camera:").size(12.5).strong().color(colors.text_secondary));
                    let is_2d = self.rigging_camera_mode == CanvasCameraMode::TopView2D;
                    if ui.selectable_label(is_2d, format!("{} 2D Top View (Flat)", egui_phosphor::regular::FRAME_CORNERS)).clicked() {
                        self.rigging_camera_mode = CanvasCameraMode::TopView2D;
                    }
                    if ui.selectable_label(!is_2d, format!("{} 3D Perspective", egui_phosphor::regular::CUBE)).clicked() {
                        self.rigging_camera_mode = CanvasCameraMode::Perspective3D;
                        if self.rigging_orbit_pitch < 0.2 {
                            self.rigging_orbit_pitch = 0.60;
                            self.rigging_orbit_yaw = -0.75;
                        }
                    }

                    if ui
                        .button(format!("{} Center", egui_phosphor::regular::CROSSHAIR))
                        .on_hover_text("Reset viewport pan and zoom to origin")
                        .clicked()
                    {
                        self.rigging_pan = Vec2::ZERO;
                        self.rigging_zoom = 0.85;
                        self.rigging_orbit_yaw = 0.60;
                        self.rigging_orbit_pitch = 0.55;
                    }

                    ui.separator();

                    // Tools
                    ui.label(RichText::new("Tool:").size(12.5).strong().color(colors.text_secondary));
                    let is_select = self.rigging_tool_mode == RiggingToolMode::Select;
                    if ui.selectable_label(is_select, format!("{} Inspect & Orbit", egui_phosphor::regular::HAND_PALM)).clicked() {
                        self.rigging_tool_mode = RiggingToolMode::Select;
                        self.rigging_pending_p1 = None;
                    }
                    if ui
                        .selectable_label(!is_select, format!("{} Add Bend Line", egui_phosphor::regular::PENCIL_SIMPLE_LINE))
                        .on_hover_text("Click two points or relief notches to add a bend line")
                        .clicked()
                    {
                        self.rigging_tool_mode = RiggingToolMode::AddBendLine;
                        self.rigging_pending_p1 = None;
                    }

                    // Undo / Redo in Toolbar (User Request 1)
                    let can_undo = !self.undo_stack.is_empty();
                    let can_redo = !self.redo_stack.is_empty();
                    if ui
                        .add_enabled(
                            can_undo,
                            egui::Button::new(RichText::new(format!("{} Undo", egui_phosphor::regular::ARROW_COUNTER_CLOCKWISE)).size(11.5)),
                        )
                        .on_hover_text("Undo bend line / sheet operation (Ctrl+Z)")
                        .clicked()
                    {
                        self.trufold_undo();
                    }
                    if ui
                        .add_enabled(
                            can_redo,
                            egui::Button::new(RichText::new(format!("{} Redo", egui_phosphor::regular::ARROW_CLOCKWISE)).size(11.5)),
                        )
                        .on_hover_text("Redo bend line / sheet operation (Ctrl+Y)")
                        .clicked()
                    {
                        self.trufold_redo();
                    }

                    ui.separator();

                    // Floor Blueprint toggle
                    let bp_color = if self.rigging_show_floor_blueprint { Color32::from_rgb(6, 182, 212) } else { colors.text_muted };
                    let bp_btn = egui::Button::new(
                        RichText::new(format!(
                            "{} Floor Blueprint: {}",
                            egui_phosphor::regular::EYE,
                            if self.rigging_show_floor_blueprint { "ON" } else { "OFF" }
                        ))
                        .strong()
                        .color(bp_color),
                    )
                    .fill(if self.rigging_show_floor_blueprint { Color32::from_rgba_unmultiplied(6, 182, 212, 30) } else { colors.bg_card_alt })
                    .corner_radius(4.0);
                    if ui
                        .add(bp_btn)
                        .on_hover_text("Projects translucent flat 2D blueprint with bend lines onto the floor (Z=0) under the 3D folded model")
                        .clicked()
                    {
                        self.rigging_show_floor_blueprint = !self.rigging_show_floor_blueprint;
                    }

                    let win_3d_text = if self.show_3d_viewport_window {
                        format!("{} 3D Window: OPEN", egui_phosphor::regular::CUBE)
                    } else {
                        format!("{} 3D Window: CLOSED", egui_phosphor::regular::CUBE)
                    };
                    let win_3d_btn = egui::Button::new(RichText::new(win_3d_text).strong().color(if self.show_3d_viewport_window {
                        Color32::from_rgb(168, 85, 247)
                    } else {
                        colors.text_muted
                    }))
                    .fill(if self.show_3d_viewport_window { Color32::from_rgba_unmultiplied(168, 85, 247, 30) } else { colors.bg_card_alt })
                    .corner_radius(4.0);
                    if ui
                        .add(win_3d_btn)
                        .on_hover_text("Toggle floating, resizable 3D viewport window actively synchronized with 2D CAD operations")
                        .clicked()
                    {
                        self.show_3d_viewport_window = !self.show_3d_viewport_window;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!("Sheet Thickness: {:.1}mm (Physical Gauge)", self.rigged_sheet.material_rule.thickness_mm))
                                .size(11.5)
                                .strong()
                                .color(Color32::from_rgb(56, 189, 248)),
                        );
                    });
                });
            });

        ui.add_space(4.0);

        // ROW 2: Fold Simulation Controls & Pipeline Actions
        Frame::NONE
            .fill(colors.bg_card)
            .stroke(Stroke::new(1.0, colors.border_subtle))
            .inner_margin(Margin::symmetric(12, 6))
            .corner_radius(6.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;

                    // Fold Progress Slider
                    ui.label(RichText::new("Fold Simulation:").size(12.5).strong().color(colors.text_secondary));
                    let slider_res = ui.add_sized([130.0, 18.0], egui::Slider::new(&mut self.rigging_fold_slider_deg, 0.0..=90.0).suffix("°"));
                    if slider_res.changed() && self.rigging_fold_slider_deg > 0.0 && self.rigging_camera_mode == CanvasCameraMode::TopView2D {
                        self.rigging_camera_mode = CanvasCameraMode::Perspective3D;
                        self.rigging_orbit_pitch = 0.55;
                        self.rigging_orbit_yaw = 0.60;
                    }

                    if ui.button("0° Flat").on_hover_text("Reset to flat 2D blank").clicked() {
                        self.rigging_fold_slider_deg = 0.0;
                    }
                    if ui.button("45° Formed").on_hover_text("Inspect intermediate 45° bend angle").clicked() {
                        self.rigging_fold_slider_deg = 45.0;
                        self.rigging_camera_mode = CanvasCameraMode::Perspective3D;
                    }
                    if ui.button("90° Folded").on_hover_text("Complete full 90° flange fold").clicked() {
                        self.rigging_fold_slider_deg = 90.0;
                        self.rigging_camera_mode = CanvasCameraMode::Perspective3D;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;

                        let sc_btn = egui::Button::new(
                            RichText::new(format!("{} SolidCraft CAD ↗", egui_phosphor::regular::ROCKET_LAUNCH))
                                .size(12.0)
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(colors.accent)
                        .corner_radius(5.0);
                        if ui.add(sc_btn).on_hover_text("Execute compiled sheet.fold pipeline in SolidCraft CAD").clicked() {
                            let cmds = self.rigged_sheet.compile_solidcraft_fold_commands();
                            self.solidcraft_status = Some(format!("✓ Compiled {} SolidCraft sheet.fold commands", cmds.len()));
                            let active_file = self.active_solidcraft_file.clone();
                            self.launch_solidcraft_process(Some(&active_file));
                        }

                        let pdf_btn = egui::Button::new(
                            RichText::new(format!("{} Export TDS PDF", egui_phosphor::regular::FILE_PDF)).size(12.0).strong().color(colors.accent),
                        )
                        .fill(colors.accent_subtle)
                        .corner_radius(5.0);
                        if ui.add(pdf_btn).on_hover_text("Generate Bonn Metal Technical Data Sheet (TDS) PDF with rigged dimensions").clicked() {
                            self.generate_submittal_pdf();
                        }

                        let save_btn = egui::Button::new(
                            RichText::new(format!("{} Save .DSF", egui_phosphor::regular::FLOPPY_DISK))
                                .size(12.0)
                                .strong()
                                .color(Color32::from_rgb(16, 185, 129)),
                        )
                        .fill(Color32::from_rgba_unmultiplied(16, 185, 129, 36))
                        .stroke(Stroke::new(1.0, Color32::from_rgba_unmultiplied(16, 185, 129, 140)))
                        .corner_radius(5.0);
                        if ui
                            .add(save_btn)
                            .on_hover_text("Dual-Save as both native SolidCraft v0.25.1 (.solidcraft Format 41) and .DSF container")
                            .clicked()
                        {
                            let base_path =
                                format!("assets/templates/bonn_metal/{}", self.rigged_sheet.product_code.to_lowercase().replace('-', "_"));
                            let meta = cadense_dsf::bonn_metal_catalog()[0].clone();
                            if let Ok(container) = cadense_dsf::DsfContainer::from_rigged_sheet(self.rigged_sheet.clone(), meta) {
                                if let Ok((sc_path, dsf_path)) = container.save_dual_solidcraft_and_dsf(&base_path) {
                                    self.solidcraft_status = Some(format!("✓ Dual-Saved {} AND {}", sc_path.display(), dsf_path.display()));
                                }
                            }
                        }
                    });
                });
            });

        ui.add_space(8.0);

        // --- SECTION 3: UNIFIED VIEWPORT CANVAS + RIGHT RAIL INSPECTOR ---
        let avail_size = ui.available_size();
        let right_rail_w = 300.0_f32;
        let canvas_w = (avail_size.x - right_rail_w - 14.0).max(350.0);
        let canvas_h = avail_size.y.max(400.0);

        ui.horizontal(|ui| {
            // LEFT: Interactive Unified Canvas
            self.render_interactive_canvas(ui, Vec2::new(canvas_w, canvas_h), colors);

            ui.add_space(8.0);

            // RIGHT: Bending Sequence & Tooling Inspector
            self.render_bend_sequence_inspector(ui, right_rail_w, colors);
        });
    }
}

impl eframe::App for CadenseStudioApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Check if theme or font scale changed and apply dynamically
        let theme_key = (self.theme, (self.font_scale * 1000.0) as u32);
        if self.last_applied_theme_key != Some(theme_key) {
            self.theme.apply_to_ctx(&ctx, self.font_scale);
            self.last_applied_theme_key = Some(theme_key);
        }

        let colors = self.theme.colors();

        // Global Keyboard Shortcuts (matching SolidCraft conventions)
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Num1)) {
            self.active_tab = StudioWorkspaceTab::Dashboard;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Num2)) {
            self.active_tab = StudioWorkspaceTab::DxfCleaner;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Num3)) {
            self.active_tab = StudioWorkspaceTab::SolidCraftAuthoring;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Num4)) {
            self.active_tab = StudioWorkspaceTab::CmsTruFoldWorkbench;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Num5)) {
            self.active_tab = StudioWorkspaceTab::OrderBatchQueue;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Comma)) {
            self.show_settings_modal = !self.show_settings_modal;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::L)) {
            self.launch_solidcraft_process(None);
        }
        // Undo / Redo Shortcuts (User Request 1)
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Z)) {
            if self.active_tab == StudioWorkspaceTab::CmsTruFoldWorkbench {
                self.trufold_undo();
            }
        }
        if ctx.input_mut(|i| {
            i.consume_key(egui::Modifiers::COMMAND, egui::Key::Y) || i.consume_key(egui::Modifiers::COMMAND | egui::Modifiers::SHIFT, egui::Key::Z)
        }) {
            if self.active_tab == StudioWorkspaceTab::CmsTruFoldWorkbench {
                self.trufold_redo();
            }
        }

        // Escape Key Behavior (User Request 2):
        // Esc key should NOT close the TruFold view to Dashboard.
        // In TruFold, Esc only cancels an actively drawn bend line point or resets the tool to Select.
        if !self.show_settings_modal && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            if self.active_tab == StudioWorkspaceTab::CmsTruFoldWorkbench {
                if self.rigging_pending_p1.is_some() {
                    self.rigging_pending_p1 = None;
                    self.solidcraft_status = Some("Cancelled bend line drawing point (Esc)".to_string());
                } else if self.rigging_tool_mode != RiggingToolMode::Select {
                    self.rigging_tool_mode = RiggingToolMode::Select;
                    self.solidcraft_status = Some("Tool: Select & Orbit (Esc)".to_string());
                }
            } else {
                self.active_tab = StudioWorkspaceTab::Dashboard;
            }
        }

        // 1. TOP APP BAR (SolidCraft Core Chrome: Logo lockup, Home button, File/Save/Undo/Redo, Document Tab, Quick Actions)
        Frame::NONE
            .fill(if colors.is_sidebar_blue() { Color32::from_rgb(20, 24, 30) } else { colors.bg_sidebar })
            .stroke(Stroke::new(1.0, colors.border_subtle))
            .inner_margin(Margin::symmetric(14, 8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    CadenseLogoWidget::show_horizontal_lockup(ui, 28.0, 16.0);
                    ui.add_space(8.0);

                    // Home icon button (SolidCraft house button)
                    let home_hover = self.active_tab == StudioWorkspaceTab::Dashboard;
                    if ui.selectable_label(home_hover, egui_phosphor::regular::HOUSE).on_hover_text("Return to Start / Dashboard (Esc)").clicked() {
                        self.active_tab = StudioWorkspaceTab::Dashboard;
                    }
                    ui.add_space(4.0);

                    // Quick File / Save / Undo / Redo tools
                    ui.label(RichText::new("File").size(12.5).color(Color32::from_rgb(200, 210, 225)));
                    ui.add_space(4.0);
                    if ui.button(egui_phosphor::regular::FLOPPY_DISK).on_hover_text("Save / Export active workflow artifacts").clicked() {
                        self.generate_submittal_pdf();
                    }
                    if ui.button(egui_phosphor::regular::ARROW_COUNTER_CLOCKWISE).on_hover_text("Undo (Ctrl+Z)").clicked() {}
                    if ui.button(egui_phosphor::regular::ARROW_CLOCKWISE).on_hover_text("Redo (Ctrl+Y)").clicked() {}

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Document Tab pill (matching SolidCraft's tab strip: [ Untitled ] [+])
                    Frame::NONE
                        .fill(if colors.is_sidebar_blue() { Color32::from_rgb(35, 40, 50) } else { colors.bg_card })
                        .stroke(Stroke::new(1.0, Color32::from_rgb(60, 70, 85)))
                        .inner_margin(Margin::symmetric(10, 5))
                        .corner_radius(4.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("CMS Sheet Metal Studio").size(12.0).strong().color(Color32::WHITE));
                                ui.colored_label(Color32::from_rgb(150, 165, 185), "•");
                                ui.label(RichText::new(self.active_tab.title()).size(12.0).color(colors.accent));
                            });
                        });

                    // Right-side actions: Clean, uncluttered, professional (Theme ComboBox + Preferences + SolidCraft CAD)
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Prominent SolidCraft CAD launcher
                        let sc_launch_btn = egui::Button::new(
                            RichText::new(format!("{} SolidCraft CAD", egui_phosphor::regular::ROCKET_LAUNCH))
                                .size(12.5)
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(colors.accent)
                        .corner_radius(6.0);

                        if ui.add(sc_launch_btn).on_hover_text("Launch native SolidCraft CAD (Ctrl+L)").clicked() {
                            self.launch_solidcraft_process(None);
                        }

                        // Settings Preferences Button
                        let settings_btn = egui::Button::new(
                            RichText::new(format!("{} Preferences", egui_phosphor::regular::GEAR)).size(12.5).strong().color(colors.text_primary),
                        )
                        .fill(colors.bg_card)
                        .stroke(Stroke::new(1.0, colors.border_subtle))
                        .corner_radius(6.0);
                        if ui.add(settings_btn).on_hover_text("Open Preferences, Theming, and Version Matrix (Ctrl+,)").clicked() {
                            self.show_settings_modal = true;
                            self.prefs_selected_page = 1;
                        }

                        // Compact Theme Selector Dropdown
                        ui.add_space(6.0);
                        egui::ComboBox::from_id_salt("top_app_bar_theme_combo")
                            .selected_text(match self.theme {
                                CadenseTheme::SolidCraftCobalt => "🔷 Cobalt",
                                CadenseTheme::DeepObsidian => "🌙 Navy",
                                CadenseTheme::GruvboxDark => "🌲 Gruvbox",
                                CadenseTheme::WarmClay => "☕ Clay",
                                CadenseTheme::ModernLight => "☀ Light",
                            })
                            .width(105.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.theme, CadenseTheme::SolidCraftCobalt, "🔷 Cobalt (SolidCraft)");
                                ui.selectable_value(&mut self.theme, CadenseTheme::DeepObsidian, "🌙 Navy (Dark)");
                                ui.selectable_value(&mut self.theme, CadenseTheme::GruvboxDark, "🌲 Gruvbox Dark");
                                ui.selectable_value(&mut self.theme, CadenseTheme::WarmClay, "☕ Warm Clay");
                                ui.selectable_value(&mut self.theme, CadenseTheme::ModernLight, "☀ Modern Light");
                            });
                    });
                });
            });

        ui.separator();

        // 2. MAIN HORIZONTAL SPLIT (Left Navigation Drawer + Active Tab Workspace Canvas)
        let bottom_bar_h = 28.0;
        let content_avail_h = (ui.available_height() - bottom_bar_h).max(100.0);

        ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
            // --- LEFT NAVIGATION DRAWER (SolidCraft signature rail) ---
            let sidebar_w = 248.0;
            let full_avail_h = content_avail_h;
            ui.allocate_ui_with_layout(Vec2::new(sidebar_w, full_avail_h), egui::Layout::top_down(egui::Align::Min), |ui| {
                Frame::NONE.fill(colors.bg_sidebar).stroke(Stroke::new(1.0, colors.border_subtle)).inner_margin(Margin::symmetric(14, 16)).show(
                    ui,
                    |ui| {
                        ui.set_width(sidebar_w - 28.0);
                        ui.vertical(|ui| {
                            let is_blue = colors.is_sidebar_blue();
                            let lockup_fill = if is_blue { Color32::from_rgba_unmultiplied(0, 0, 0, 45) } else { colors.bg_card };
                            let lockup_stroke = if is_blue {
                                Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 45))
                            } else {
                                Stroke::new(1.0, colors.border_subtle)
                            };

                            if self.active_tab != StudioWorkspaceTab::Dashboard {
                                if Self::back_to_row(ui, "Back to Dashboard", "Esc", &colors) {
                                    self.active_tab = StudioWorkspaceTab::Dashboard;
                                }
                                ui.add_space(4.0);
                            } else {
                                Frame::NONE.fill(lockup_fill).stroke(lockup_stroke).inner_margin(Margin::symmetric(10, 8)).corner_radius(6.0).show(
                                    ui,
                                    |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(CURRENT_VERSION.app_version).size(12.5).strong().color(if is_blue {
                                                Color32::WHITE
                                            } else {
                                                colors.accent
                                            }));
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                ui.label(
                                                    RichText::new(format!("Format {}", CURRENT_VERSION.solidcraft_format))
                                                        .size(11.0)
                                                        .color(if is_blue { Color32::from_rgb(186, 215, 255) } else { colors.text_muted }),
                                                );
                                            });
                                        });
                                        ui.label(
                                            RichText::new(format!("{} {}", CURRENT_VERSION.core_engine, CURRENT_VERSION.core_version))
                                                .size(11.0)
                                                .color(if is_blue { Color32::from_rgb(220, 235, 255) } else { colors.text_secondary }),
                                        );
                                    },
                                );
                                ui.add_space(8.0);
                            }

                            ui.add_space(6.0);
                            let group_header_col = if is_blue { Color32::from_rgba_unmultiplied(255, 255, 255, 175) } else { colors.text_muted };
                            ui.label(RichText::new("WORKFLOW PIPELINE").size(10.5).strong().color(group_header_col));
                            ui.add_space(4.0);

                            let tabs = [
                                (StudioWorkspaceTab::Dashboard, "Start / Hub", "Ctrl+1"),
                                (StudioWorkspaceTab::DxfCleaner, "1. DXF Pre-Flight", "Ctrl+2"),
                                (StudioWorkspaceTab::SolidCraftAuthoring, "2. 3D CAD Bending", "Ctrl+3"),
                                (StudioWorkspaceTab::CmsTruFoldWorkbench, "3. CMS TruFold", "Ctrl+4"),
                                (StudioWorkspaceTab::OrderBatchQueue, "4. Orders & TDS", "Ctrl+5"),
                            ];

                            for (tab, title, shortcut) in tabs {
                                let is_sel = self.active_tab == tab;
                                if Self::nav_item_solidcraft(ui, tab.title(), is_sel, tab.icon(), title, shortcut, &colors) {
                                    self.active_tab = tab;
                                }
                            }

                            ui.add_space(10.0);
                            if is_blue {
                                ui.painter().hline(
                                    ui.available_rect_before_wrap().x_range(),
                                    ui.cursor().top(),
                                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 45)),
                                );
                                ui.add_space(6.0);
                            } else {
                                ui.separator();
                            }
                            ui.add_space(8.0);

                            ui.label(RichText::new("ACTIONS & SHORTCUTS").size(10.5).strong().color(group_header_col));
                            ui.add_space(4.0);

                            if Self::nav_item_solidcraft(
                                ui,
                                "sc_launch",
                                false,
                                egui_phosphor::regular::ROCKET_LAUNCH,
                                "Launch SolidCraft CAD",
                                "Ctrl+L",
                                &colors,
                            ) {
                                self.launch_solidcraft_process(None);
                            }

                            if Self::nav_item_solidcraft(
                                ui,
                                "open_assets",
                                false,
                                egui_phosphor::regular::FOLDER_OPEN,
                                "Open Assets Folder",
                                "Ctrl+Shift+A",
                                &colors,
                            ) {
                                open_path_in_explorer(&self.landing.resolved_asset_root);
                            }

                            if Self::nav_item_solidcraft(
                                ui,
                                "open_drawings",
                                false,
                                egui_phosphor::regular::FOLDER_OPEN,
                                "Open Drawings Folder",
                                "Ctrl+Shift+D",
                                &colors,
                            ) {
                                open_path_in_explorer("drawings\\cms_in_progress");
                            }

                            ui.add_space(10.0);
                            if is_blue {
                                ui.painter().hline(
                                    ui.available_rect_before_wrap().x_range(),
                                    ui.cursor().top(),
                                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 45)),
                                );
                                ui.add_space(6.0);
                            } else {
                                ui.separator();
                            }
                            ui.add_space(8.0);

                            ui.label(RichText::new("SYSTEM").size(10.5).strong().color(group_header_col));
                            ui.add_space(4.0);

                            if Self::nav_item_solidcraft(
                                ui,
                                "prefs_btn",
                                self.show_settings_modal,
                                egui_phosphor::regular::GEAR,
                                "Preferences",
                                "Ctrl+,",
                                &colors,
                            ) {
                                self.show_settings_modal = true;
                                self.prefs_selected_page = 1;
                            }

                            if Self::nav_item_solidcraft(
                                ui,
                                "shortcuts_btn",
                                false,
                                egui_phosphor::regular::KEYBOARD,
                                "Keyboard shortcuts",
                                "",
                                &colors,
                            ) {
                                self.show_settings_modal = true;
                                self.prefs_selected_page = 4;
                            }

                            if Self::nav_item_solidcraft(ui, "about_btn", false, egui_phosphor::regular::INFO, "About SolidCraft", "", &colors) {
                                self.show_settings_modal = true;
                                self.prefs_selected_page = 10;
                            }

                            // Bottom Pin: Architecture & Engine Status
                            let rem_h = ui.available_height() - 95.0;
                            if rem_h > 0.0 {
                                ui.add_space(rem_h);
                            }
                            Frame::NONE.fill(lockup_fill).stroke(lockup_stroke).inner_margin(Margin::same(10)).corner_radius(6.0).show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.colored_label(Color32::from_rgb(16, 185, 129), egui_phosphor::regular::CHECK_CIRCLE);
                                    ui.label(RichText::new("TruFold Core v0.2.0").size(11.5).strong().color(if is_blue {
                                        Color32::WHITE
                                    } else {
                                        colors.text_primary
                                    }));
                                });
                                ui.label(RichText::new(format!("SolidCraft {}", CURRENT_VERSION.solidcraft_version)).size(11.0).color(if is_blue {
                                    Color32::from_rgb(220, 235, 255)
                                } else {
                                    colors.text_secondary
                                }));
                                ui.label(RichText::new("Rule R1 Zero-Touch Additive").size(10.5).color(if is_blue {
                                    Color32::from_rgb(186, 215, 255)
                                } else {
                                    colors.text_muted
                                }));
                            });
                        });
                    },
                );
            });

            // --- MAIN WORKSPACE CANVAS (Always starts at Top y = 0) ---
            let main_avail = ui.available_size();
            ui.allocate_ui_with_layout(main_avail, egui::Layout::top_down(egui::Align::Min), |ui| {
                Frame::NONE.fill(colors.bg_app).inner_margin(Margin::symmetric(16, 12)).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    let mut pending_action = None;

                    match self.active_tab {
                        StudioWorkspaceTab::Dashboard => {
                            if let Some(act) = self.landing.render_dashboard_hub(ui, &colors) {
                                pending_action = Some(act);
                            }
                        }
                        StudioWorkspaceTab::DxfCleaner => {
                            if let Some(act) = self.landing.render_stage_1_dxf_assets(ui, &colors) {
                                pending_action = Some(act);
                            }
                        }
                        StudioWorkspaceTab::SolidCraftAuthoring => {
                            if let Some(act) = self.landing.render_stage_2_solidcraft_drawings(ui, &colors) {
                                pending_action = Some(act);
                            }
                        }
                        StudioWorkspaceTab::CmsTruFoldWorkbench => {
                            self.cms_ext.show_floating_windows(&ctx);
                            self.render_unified_bending_workbench(ui, &colors);
                        }
                        StudioWorkspaceTab::OrderBatchQueue => {
                            if let Some(act) = self.landing.render_stage_4_orders_and_submittal(ui, &colors) {
                                pending_action = Some(act);
                            }
                        }
                    }

                    if let Some(act) = pending_action {
                        self.handle_landing_action(act);
                    }
                });
            });
        });

        // 3. BOTTOM STATUS BAR (Clean, non-intrusive status, PID diagnostics & quick controls)
        Frame::NONE
            .fill(if colors.is_sidebar_blue() { Color32::from_rgb(18, 22, 28) } else { colors.bg_sidebar })
            .stroke(Stroke::new(1.0, colors.border_subtle))
            .inner_margin(Margin::symmetric(14, 5))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(Color32::from_rgb(16, 185, 129), egui_phosphor::regular::CHECK_CIRCLE);
                    ui.label(
                        RichText::new(format!(
                            "{} {} • SolidCraft {} Format {}",
                            CURRENT_VERSION.core_engine,
                            CURRENT_VERSION.core_version,
                            CURRENT_VERSION.solidcraft_version,
                            CURRENT_VERSION.solidcraft_format
                        ))
                        .size(11.0)
                        .color(colors.text_muted),
                    );

                    if let Some(status) = &self.solidcraft_status.clone() {
                        ui.separator();
                        Frame::NONE.fill(colors.badge_success_bg).inner_margin(Margin::symmetric(8, 2)).corner_radius(4.0).show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(status).size(11.0).strong().color(colors.badge_success_text));
                                if ui.small_button(egui_phosphor::regular::X).on_hover_text("Dismiss status notice").clicked() {
                                    self.solidcraft_status = None;
                                    self.landing.status_notice = None;
                                }
                            });
                        });
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new("Ready • Units: mm • Neutral Axis K: 0.44").size(11.0).color(colors.text_muted));
                    });
                });
            });

        // 4. SETTINGS & THEMING MODAL WINDOW
        if self.show_settings_modal {
            self.render_settings_modal(&ctx, &colors);
        }

        // 5. FLOATING & RESIZABLE 3D VIEWPORT WINDOW
        self.render_floating_3d_viewport_window(&ctx, &colors);
    }
}

pub fn launch_solidcraft(target_file: Option<&str>) -> Result<u32, String> {
    let mut candidates = vec![
        PathBuf::from("target/debug/solidcraft.exe"),
        PathBuf::from("target/release/solidcraft.exe"),
        PathBuf::from("../target/debug/solidcraft.exe"),
    ];
    if let Ok(cur) = std::env::current_exe() {
        if let Some(parent) = cur.parent() {
            candidates.insert(0, parent.join("solidcraft.exe"));
        }
    }

    for exe in &candidates {
        if exe.exists() {
            let mut cmd = std::process::Command::new(exe);
            if let Some(f) = target_file {
                cmd.arg(f);
            }
            match cmd.spawn() {
                Ok(child) => return Ok(child.id()),
                Err(e) => return Err(format!("Could not spawn {}: {}", exe.display(), e)),
            }
        }
    }

    // Fallback: cargo run -p solidcraft -- [file]
    let mut cmd = std::process::Command::new("cargo");
    cmd.args(["run", "-p", "solidcraft", "--"]);
    if let Some(f) = target_file {
        cmd.arg(f);
    }
    match cmd.spawn() {
        Ok(child) => Ok(child.id()),
        Err(e) => Err(format!("Could not execute cargo run -p solidcraft: {}", e)),
    }
}

pub fn open_path_in_explorer(path: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer").arg(path).spawn();
    }
}

fn ensure_starter_templates_and_folders() {
    let template_dir = Path::new("assets/templates/bonn_metal");
    let _ = fs::create_dir_all(template_dir);

    for meta in cadense_dsf::bonn_metal_catalog() {
        let filename = format!("{}.dsf", meta.product_code.to_lowercase().replace('-', "_"));
        let target_path = template_dir.join(&filename);
        if !target_path.exists() {
            let container = cadense_dsf::DsfContainer::new(Vec::new(), meta);
            let _ = container.save_to_file(&target_path);
        }
    }
}
