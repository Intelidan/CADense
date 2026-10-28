//! Root `eframe::App` shell for `cadense-studio`.
//! Compatible with SolidCraft workspace dependencies (egui / eframe 0.36).

use crate::landing::{CadenseLandingScreen, LandingAction};
use cadense_order::BonnCatalogCode;
use cadense_ui::{
    CadenseLogoWidget, CadenseSolidCraftExtension, CadenseTheme, ThemeColors,
    CURRENT_VERSION, CONCEPT_23_BADGE_SVG, CONCEPT_23_HORIZONTAL_LOCKUP_SVG,
    CONCEPT_23_METALLIC_3D_SVG,
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
}

impl CadenseStudioApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        ensure_starter_templates_and_folders();

        let landing = CadenseLandingScreen::default();
        let asset_root = landing.resolved_asset_root.clone();

        Self {
            active_tab: StudioWorkspaceTab::Dashboard,
            landing,
            cms_ext: CadenseSolidCraftExtension::default(),
            active_solidcraft_file: "drawings/cms_in_progress/cable_tray_unequal_tee.solidcraft".to_string(),
            solidcraft_status: None,
            theme: CadenseTheme::SolidCraftCobalt, // Default to SolidCraft Signature Cobalt theme!
            font_scale: 1.05,                     // Generous legible default scaling
            show_settings_modal: false,
            prefs_selected_page: 1,               // Default to 1 (🎨 Interface) matching SolidCraft screenshot!
            temp_custom_asset_root: asset_root,
            last_applied_theme_key: None,
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
        let mut dds = cadense_dds_pdf::DdsArchive::new(cadense_dds_pdf::DdsProjectMeta::default());
        dds.populate_from_order_queue(&self.cms_ext.order_engine);
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
                self.active_solidcraft_file = path;
                self.active_tab = StudioWorkspaceTab::CmsTruFoldWorkbench;
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
            if is_selected { Color32::WHITE } else { Color32::from_rgba_unmultiplied(255, 255, 255, 225) }
        } else {
            if is_selected { colors.accent } else { colors.text_primary }
        };

        let font = if is_selected {
            egui::FontId::new(13.5, egui::FontFamily::Proportional)
        } else {
            egui::FontId::new(13.5, egui::FontFamily::Proportional)
        };

        let text_rect = Rect::from_min_max(
            inner.min,
            Pos2::new((inner.right() - key_width).max(inner.left()), inner.bottom()),
        );

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

    fn back_to_row(
        ui: &mut egui::Ui,
        title: &str,
        shortcut: &str,
        colors: &ThemeColors,
    ) -> bool {
        let is_blue = colors.is_sidebar_blue();
        let height = 40.0; // SolidCraft file_back_height
        let width = ui.available_width();
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), egui::Sense::click());

        if response.hovered() || response.has_focus() {
            let fill = if is_blue {
                Color32::from_rgba_unmultiplied(255, 255, 255, 30)
            } else {
                Color32::from_rgba_unmultiplied(226, 234, 246, 120)
            };
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
        let mini_rail = Rect::from_min_max(
            Pos2::new(mini_box.left(), mini_top.bottom()),
            Pos2::new(mini_box.left() + 22.0, mini_box.bottom()),
        );
        ui.painter().rect_filled(mini_rail, 0.0, rail_col);

        // Mini Viewport Canvas
        let mini_canvas = Rect::from_min_max(
            Pos2::new(mini_rail.right(), mini_top.bottom()),
            mini_box.max,
        );
        ui.painter().rect_filled(mini_canvas, 0.0, canvas_col);

        // Mini accent dot in viewport
        ui.painter().circle_filled(mini_canvas.center(), 3.5, accent_col);

        // Checkmark badge in circle if selected (top right of tile)
        if is_sel {
            let badge_pos = Pos2::new(rect.right() - 11.0, rect.top() + 11.0);
            ui.painter().circle_filled(badge_pos, 7.5, colors.accent);
            let check = ui.painter().layout_no_wrap(egui_phosphor::regular::CHECK.to_owned(), egui::FontId::new(9.5, egui::FontFamily::Proportional), Color32::WHITE);
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

        let note_galley = ui.painter().layout_no_wrap(
            note.to_owned(),
            egui::FontId::new(10.0, egui::FontFamily::Proportional),
            colors.text_muted,
        );
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
        let pts = vec![
            p.left_top(), p.right_top(), p.right_bottom(), p.left_bottom(), p.left_top(),
        ];
        ui.painter().extend(egui::Shape::dashed_line(&pts, Stroke::new(1.2, colors.border_strong), 4.0, 3.0));

        // Download / Import icon & text
        let center = rect.center();
        let icon = ui.painter().layout_no_wrap(egui_phosphor::regular::ARROW_DOWN.to_owned(), egui::FontId::new(18.0, egui::FontFamily::Proportional), colors.text_secondary);
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
        ui.painter().rect(
            rect,
            8.0,
            colors.bg_card,
            Stroke::new(1.2, Color32::from_rgb(180, 195, 215)),
            StrokeKind::Inside,
        );

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
        let b_label = ui.painter().layout_no_wrap("Browser".to_owned(), egui::FontId::new(9.5, egui::FontFamily::Proportional), Color32::from_rgb(70, 80, 95));
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
        ui.painter().rect(
            hist_rect,
            4.0,
            Color32::from_rgb(235, 242, 250),
            Stroke::new(0.8, Color32::from_rgb(195, 210, 225)),
            StrokeKind::Inside,
        );
        let h_label = ui.painter().layout_no_wrap("History".to_owned(), egui::FontId::new(9.5, egui::FontFamily::Proportional), Color32::from_rgb(70, 80, 95));
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

    fn render_dual_viewport_canvas(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
        let avail = ui.available_size();
        let half_w = ((avail.x - 16.0) * 0.5).max(200.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 16.0;
            ui.allocate_ui_with_layout(Vec2::new(half_w, avail.y.max(300.0)), egui::Layout::top_down(egui::Align::Min), |ui| {
                egui::Frame::canvas(ui.style())
                    .fill(colors.canvas_bg)
                    .stroke(Stroke::new(1.0, colors.border_subtle))
                    .corner_radius(6.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("3D Folded Sheet-Metal Viewport").strong().color(colors.text_primary));
                            if self.cms_ext.coupler_flaps {
                                ui.colored_label(Color32::from_rgb(16, 185, 129), format!("• Flaps Active ({})", self.cms_ext.coupler_hole_pattern));
                            } else {
                                ui.colored_label(Color32::from_rgb(245, 158, 11), "• Flaps OFF (Butt-Joint)");
                            }
                        });

                        let (_, rect) = ui.allocate_space(Vec2::new(half_w - 20.0, (avail.y - 40.0).max(280.0)));
                        let painter = ui.painter_at(rect);
                        let center = rect.center();

                        if let Some(sol) = &self.cms_ext.last_solution {
                            for (idx, wall) in sol.transition_walls.iter().enumerate() {
                                let y_off = (idx as f32 - 1.0) * 65.0;
                                let p_start = Pos2::new(center.x - 110.0, center.y + y_off);
                                let p_end = Pos2::new(center.x + 110.0, center.y + y_off);
                                let h_start = (wall.start_height_mm as f32) * 0.65;
                                let h_end = (wall.end_height_mm as f32) * 0.65;

                                let quad = vec![p_start, p_end, Pos2::new(p_end.x, p_end.y - h_end), Pos2::new(p_start.x, p_start.y - h_start)];
                                painter.add(egui::Shape::convex_polygon(
                                    quad,
                                    Color32::from_rgba_unmultiplied(0x25, 0x63, 0xEB, 70),
                                    Stroke::new(2.0_f32, colors.accent),
                                ));
                            }
                        }
                    });
            });

            ui.allocate_ui_with_layout(Vec2::new(half_w, avail.y.max(300.0)), egui::Layout::top_down(egui::Align::Min), |ui| {
                egui::Frame::canvas(ui.style())
                    .fill(colors.canvas_bg)
                    .stroke(Stroke::new(1.0, colors.border_subtle))
                    .corner_radius(6.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("2D Unfolded Flat Pattern (sheet.flat_pattern)")
                                    .strong()
                                    .color(colors.accent),
                            );
                            ui.colored_label(colors.text_secondary, format!("Template: {}", self.cms_ext.dsf_export_path));
                        });

                        let (_, rect) = ui.allocate_space(Vec2::new(half_w - 20.0, (avail.y - 40.0).max(280.0)));
                        let painter = ui.painter_at(rect);

                        if let Some(sol) = &self.cms_ext.last_solution {
                            let mut y_cursor = rect.min.y + 30.0;
                            for wall in &sol.transition_walls {
                                let scale = 0.75_f32;
                                let l = (wall.bend_line_length_mm as f32 * scale).clamp(90.0, 320.0);
                                let h0 = (wall.start_height_mm as f32 * scale).clamp(25.0, 120.0);
                                let h1 = (wall.end_height_mm as f32 * scale).clamp(25.0, 120.0);
                                let x0 = rect.min.x + 40.0;

                                let trap = vec![
                                    Pos2::new(x0, y_cursor + h0.max(h1)),
                                    Pos2::new(x0 + l, y_cursor + h0.max(h1)),
                                    Pos2::new(x0 + l, y_cursor + h0.max(h1) - h1),
                                    Pos2::new(x0, y_cursor + h0.max(h1) - h0),
                                ];
                                painter.add(egui::Shape::convex_polygon(
                                    trap,
                                    Color32::from_rgba_unmultiplied(148, 163, 184, 50),
                                    Stroke::new(1.5_f32, colors.text_primary),
                                ));

                                // Render coupler flap slots if enabled
                                if self.cms_ext.coupler_flaps {
                                    let count = self.cms_ext.coupler_slot_count.min(wall.start_port_slot_count.max(1));
                                    let off_x = (self.cms_ext.coupler_hole_offset_mm as f32 * 0.45).clamp(8.0, 26.0);
                                    for s in 0..count {
                                        let sy = y_cursor + h0.max(h1) - 14.0 - (s as f32 * 18.0);
                                        let slot_rect = Rect::from_min_size(Pos2::new(x0 + off_x, sy), Vec2::new(16.0, 6.5));
                                        painter.rect_stroke(slot_rect, 3.0, Stroke::new(1.5_f32, Color32::from_rgb(16, 185, 129)), StrokeKind::Middle);
                                    }
                                }

                                y_cursor += h0.max(h1) + 28.0;
                            }
                        }
                    });
            });
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
        if !self.show_settings_modal && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            self.active_tab = StudioWorkspaceTab::Dashboard;
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
                        let settings_btn = egui::Button::new(RichText::new(format!("{} Preferences", egui_phosphor::regular::GEAR)).size(12.5).strong().color(colors.text_primary))
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
            ui.allocate_ui_with_layout(
                Vec2::new(sidebar_w, full_avail_h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    Frame::NONE
                        .fill(colors.bg_sidebar)
                        .stroke(Stroke::new(1.0, colors.border_subtle))
                        .inner_margin(Margin::symmetric(14, 16))
                        .show(ui, |ui| {
                            ui.set_width(sidebar_w - 28.0);
                            ui.vertical(|ui| {
                                let is_blue = colors.is_sidebar_blue();
                                let lockup_fill = if is_blue {
                                    Color32::from_rgba_unmultiplied(0, 0, 0, 45)
                                } else {
                                    colors.bg_card
                                };
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
                                    Frame::NONE
                                        .fill(lockup_fill)
                                        .stroke(lockup_stroke)
                                        .inner_margin(Margin::symmetric(10, 8))
                                        .corner_radius(6.0)
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(RichText::new(CURRENT_VERSION.app_version).size(12.5).strong().color(if is_blue { Color32::WHITE } else { colors.accent }));
                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    ui.label(RichText::new(format!("Format {}", CURRENT_VERSION.solidcraft_format)).size(11.0).color(if is_blue { Color32::from_rgb(186, 215, 255) } else { colors.text_muted }));
                                                });
                                            });
                                            ui.label(RichText::new(format!("{} {}", CURRENT_VERSION.core_engine, CURRENT_VERSION.core_version)).size(11.0).color(if is_blue { Color32::from_rgb(220, 235, 255) } else { colors.text_secondary }));
                                        });
                                    ui.add_space(8.0);
                                }

                                ui.add_space(6.0);
                                let group_header_col = if is_blue {
                                    Color32::from_rgba_unmultiplied(255, 255, 255, 175)
                                } else {
                                    colors.text_muted
                                };
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

                                if Self::nav_item_solidcraft(ui, "sc_launch", false, egui_phosphor::regular::ROCKET_LAUNCH, "Launch SolidCraft CAD", "Ctrl+L", &colors) {
                                    self.launch_solidcraft_process(None);
                                }

                                if Self::nav_item_solidcraft(ui, "open_assets", false, egui_phosphor::regular::FOLDER_OPEN, "Open Assets Folder", "Ctrl+Shift+A", &colors) {
                                    open_path_in_explorer(&self.landing.resolved_asset_root);
                                }

                                if Self::nav_item_solidcraft(ui, "open_drawings", false, egui_phosphor::regular::FOLDER_OPEN, "Open Drawings Folder", "Ctrl+Shift+D", &colors) {
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

                                if Self::nav_item_solidcraft(ui, "prefs_btn", self.show_settings_modal, egui_phosphor::regular::GEAR, "Preferences", "Ctrl+,", &colors) {
                                    self.show_settings_modal = true;
                                    self.prefs_selected_page = 1;
                                }

                                if Self::nav_item_solidcraft(ui, "shortcuts_btn", false, egui_phosphor::regular::KEYBOARD, "Keyboard shortcuts", "", &colors) {
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
                                Frame::NONE
                                    .fill(lockup_fill)
                                    .stroke(lockup_stroke)
                                    .inner_margin(Margin::same(10))
                                    .corner_radius(6.0)
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.colored_label(Color32::from_rgb(16, 185, 129), egui_phosphor::regular::CHECK_CIRCLE);
                                            ui.label(RichText::new("TruFold Core v0.2.0").size(11.5).strong().color(if is_blue { Color32::WHITE } else { colors.text_primary }));
                                        });
                                        ui.label(RichText::new(format!("SolidCraft {}", CURRENT_VERSION.solidcraft_version)).size(11.0).color(if is_blue { Color32::from_rgb(220, 235, 255) } else { colors.text_secondary }));
                                        ui.label(RichText::new("Rule R1 Zero-Touch Additive").size(10.5).color(if is_blue { Color32::from_rgb(186, 215, 255) } else { colors.text_muted }));
                                    });
                            });
                        });
                },
            );

            // --- MAIN WORKSPACE CANVAS (Always starts at Top y = 0) ---
            let main_avail = ui.available_size();
            ui.allocate_ui_with_layout(
                main_avail,
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    Frame::NONE
                        .fill(colors.bg_app)
                        .inner_margin(Margin::symmetric(16, 12))
                        .show(ui, |ui| {
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
                                    // Secondary Ribbon for CMS Rigging
                                    ui.horizontal(|ui| {
                                        self.cms_ext.show_ribbon_bar(ui);
                                    });
                                    ui.separator();

                                    // Actions toolbar
                                    ui.horizontal(|ui| {
                                        if ui.button(RichText::new("💾 Save Finalized .DSF Template").strong().color(Color32::from_rgb(16, 185, 129))).clicked() {
                                            if let Some(entry) = self.cms_ext.registry.entries().get(self.cms_ext.selected_variation_idx) {
                                                let out_path = format!("assets/templates/bonn_metal/{}.dsf", entry.catalog_code.as_str().to_lowercase().replace('-', "_"));
                                                let container = cadense_dsf::DsfContainer::new(Vec::new(), cadense_dsf::bonn_metal_catalog()[0].clone());
                                                let _ = container.save_to_file(&out_path);
                                                self.solidcraft_status = Some(format!("✓ Saved finalized template to {}", out_path));
                                            }
                                        }

                                        if ui.button(RichText::new("📄 Export Bonn Metal TDS Submittal (PDF)").strong().color(colors.accent)).clicked() {
                                            self.generate_submittal_pdf();
                                        }

                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.button("📂 Open templates/ Folder").clicked() {
                                                open_path_in_explorer("assets\\templates\\bonn_metal");
                                            }
                                        });
                                    });

                                    ui.separator();
                                    self.cms_ext.show_floating_windows(&ctx);
                                    self.render_dual_viewport_canvas(ui, &colors);
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
                },
            );
        });

        // 3. BOTTOM STATUS BAR (Clean, non-intrusive status, PID diagnostics & quick controls)
        Frame::NONE
            .fill(if colors.is_sidebar_blue() { Color32::from_rgb(18, 22, 28) } else { colors.bg_sidebar })
            .stroke(Stroke::new(1.0, colors.border_subtle))
            .inner_margin(Margin::symmetric(14, 5))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(Color32::from_rgb(16, 185, 129), egui_phosphor::regular::CHECK_CIRCLE);
                    ui.label(RichText::new(format!("{} {} • SolidCraft {} Format {}", CURRENT_VERSION.core_engine, CURRENT_VERSION.core_version, CURRENT_VERSION.solidcraft_version, CURRENT_VERSION.solidcraft_format)).size(11.0).color(colors.text_muted));

                    if let Some(status) = &self.solidcraft_status.clone() {
                        ui.separator();
                        Frame::NONE
                            .fill(colors.badge_success_bg)
                            .inner_margin(Margin::symmetric(8, 2))
                            .corner_radius(4.0)
                            .show(ui, |ui| {
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
