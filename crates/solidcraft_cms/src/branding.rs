//! CADense Brand Identity, Icon Generation, and Start Page Integration.
//!
//! Provides the Concept 23 visual brand identity:
//! - Deep Cobalt Steel-Blue (`#3B82F6`) interlocking T-Bar tongue
//! - Brushed Silver-White (`#E2E8F0`) chamfered `C`
//! - Dark anthracite octagonal badge (`#1E2229`)
//! - Window icon generation (64x64 RGBA)
//! - Application header lockup & Home/Landing hero banner

use egui::{Color32, FontId, Pos2, Rect, RichText, Sense, Shape, Stroke, Ui, pos2, vec2};

pub const CADENSE_COBALT_BLUE: Color32 = Color32::from_rgb(0x3B, 0x82, 0xF6);
pub const CADENSE_COBALT_SHADOW: Color32 = Color32::from_rgb(0x1D, 0x4E, 0xD8);
pub const CADENSE_SILVER_WHITE: Color32 = Color32::from_rgb(0xE2, 0xE8, 0xF0);
pub const CADENSE_SILVER_BEVEL: Color32 = Color32::from_rgb(0x94, 0xA3, 0xB8);
pub const CADENSE_BADGE_BG: Color32 = Color32::from_rgb(0x1E, 0x22, 0x29);
pub const CADENSE_BADGE_BORDER: Color32 = Color32::from_rgb(0x33, 0x39, 0x44);

/// CADense Version and Engine Metainfo
pub struct CadenseVersion;

impl CadenseVersion {
    pub const APP_NAME: &'static str = "CADense";
    pub const FULL_TITLE: &'static str = "CADense — Sheet Metal & CMS Automation";
    pub const VERSION: &'static str = "1.0.0";
    pub const TRUFOLD_ENGINE_VERSION: &'static str = "0.2.0";
    pub const UPSTREAM_ENGINE: &'static str = "SolidCraft v0.26.1 (Format 43, Tangency Labs)";
    pub const TAGLINE: &'static str = "Industrial Sheet-Metal and Cable Management Systems (CMS) Automation Platform";
}

/// Paints the CADense Concept 23 Octagonal Badge Logo.
pub fn paint_cadense_brand_mark(painter: &egui::Painter, center: Pos2, size: f32) {
    let rect = Rect::from_center_size(center, vec2(size, size));
    let map_pt = |nx: f32, ny: f32| -> Pos2 {
        pos2(rect.min.x + (nx / 256.0) * size, rect.min.y + (ny / 256.0) * size)
    };

    // Octagonal background badge
    let oct = vec![
        map_pt(76.0, 12.0),
        map_pt(180.0, 12.0),
        map_pt(244.0, 76.0),
        map_pt(244.0, 180.0),
        map_pt(180.0, 244.0),
        map_pt(76.0, 244.0),
        map_pt(12.0, 180.0),
        map_pt(12.0, 76.0),
    ];
    painter.add(Shape::convex_polygon(
        oct,
        CADENSE_BADGE_BG,
        Stroke::new((size * 0.02).max(1.0), CADENSE_BADGE_BORDER),
    ));

    // Silver-White Chamfered C
    painter.add(Shape::convex_polygon(
        vec![map_pt(76.0, 48.0), map_pt(122.0, 48.0), map_pt(122.0, 76.0), map_pt(48.0, 76.0)],
        CADENSE_SILVER_WHITE,
        Stroke::NONE,
    ));
    painter.add(Shape::convex_polygon(
        vec![map_pt(48.0, 76.0), map_pt(76.0, 76.0), map_pt(76.0, 180.0), map_pt(48.0, 180.0)],
        CADENSE_SILVER_WHITE,
        Stroke::NONE,
    ));
    painter.add(Shape::convex_polygon(
        vec![map_pt(48.0, 180.0), map_pt(122.0, 180.0), map_pt(122.0, 208.0), map_pt(76.0, 208.0)],
        CADENSE_SILVER_WHITE,
        Stroke::NONE,
    ));
    painter.add(Shape::convex_polygon(
        vec![map_pt(96.0, 76.0), map_pt(122.0, 76.0), map_pt(122.0, 94.0), map_pt(96.0, 94.0)],
        CADENSE_SILVER_BEVEL,
        Stroke::NONE,
    ));
    painter.add(Shape::convex_polygon(
        vec![map_pt(96.0, 162.0), map_pt(122.0, 162.0), map_pt(122.0, 180.0), map_pt(96.0, 180.0)],
        CADENSE_SILVER_BEVEL,
        Stroke::NONE,
    ));

    // Cobalt Blue Interlocking D / T-Bar
    painter.add(Shape::convex_polygon(
        vec![map_pt(134.0, 48.0), map_pt(180.0, 48.0), map_pt(208.0, 76.0), map_pt(134.0, 76.0)],
        CADENSE_COBALT_BLUE,
        Stroke::NONE,
    ));
    painter.add(Shape::convex_polygon(
        vec![map_pt(180.0, 76.0), map_pt(208.0, 76.0), map_pt(208.0, 180.0), map_pt(180.0, 180.0)],
        CADENSE_COBALT_BLUE,
        Stroke::NONE,
    ));
    painter.add(Shape::convex_polygon(
        vec![map_pt(134.0, 180.0), map_pt(208.0, 180.0), map_pt(180.0, 208.0), map_pt(134.0, 208.0)],
        CADENSE_COBALT_BLUE,
        Stroke::NONE,
    ));
    painter.add(Shape::convex_polygon(
        vec![map_pt(134.0, 76.0), map_pt(160.0, 76.0), map_pt(160.0, 108.0), map_pt(134.0, 108.0)],
        CADENSE_COBALT_BLUE,
        Stroke::NONE,
    ));
    painter.add(Shape::convex_polygon(
        vec![map_pt(134.0, 148.0), map_pt(160.0, 148.0), map_pt(160.0, 180.0), map_pt(134.0, 180.0)],
        CADENSE_COBALT_BLUE,
        Stroke::NONE,
    ));

    // Center T-Bar tongue
    painter.add(Shape::convex_polygon(
        vec![map_pt(86.0, 128.0), map_pt(96.0, 116.0), map_pt(166.0, 116.0), map_pt(166.0, 140.0), map_pt(96.0, 140.0)],
        CADENSE_COBALT_BLUE,
        Stroke::NONE,
    ));
    painter.add(Shape::rect_filled(
        Rect::from_min_max(map_pt(162.0, 102.0), map_pt(174.0, 154.0)),
        2.0,
        CADENSE_COBALT_BLUE,
    ));
}

/// Generates a 64x64 RGBA pixel buffer of the CADense icon for the OS window and taskbar.
pub fn generate_window_icon_rgba() -> (Vec<u8>, u32, u32) {
    let w = 64u32;
    let h = 64u32;
    let mut rgba = vec![0u8; (w * h * 4) as usize];

    for y in 0..h {
        for x in 0..w {
            let idx = ((y * w + x) * 4) as usize;
            let nx = x as f32 / w as f32;
            let ny = y as f32 / h as f32;

            // Simple anti-aliased octagonal distance metric
            let dx = (nx - 0.5).abs();
            let dy = (ny - 0.5).abs();
            let dist_corner = dx + dy;

            if dx <= 0.48 && dy <= 0.48 && dist_corner <= 0.72 {
                if dist_corner >= 0.69 || dx >= 0.46 || dy >= 0.46 {
                    // Outer border
                    rgba[idx] = 0x33;
                    rgba[idx + 1] = 0x39;
                    rgba[idx + 2] = 0x44;
                    rgba[idx + 3] = 0xFF;
                } else if nx < 0.48 && ny > 0.18 && ny < 0.82 && (nx < 0.28 || ny < 0.32 || ny > 0.68) {
                    // C shape in Silver-White
                    rgba[idx] = 0xE2;
                    rgba[idx + 1] = 0xE8;
                    rgba[idx + 2] = 0xF0;
                    rgba[idx + 3] = 0xFF;
                } else if (nx > 0.52 && nx < 0.82 && ny > 0.18 && ny < 0.82) || (nx > 0.34 && nx < 0.68 && ny > 0.44 && ny < 0.56) {
                    // T-Bar / D shape in Cobalt Blue
                    rgba[idx] = 0x3B;
                    rgba[idx + 1] = 0x82;
                    rgba[idx + 2] = 0xF6;
                    rgba[idx + 3] = 0xFF;
                } else {
                    // Octagon Badge background
                    rgba[idx] = 0x1E;
                    rgba[idx + 1] = 0x22;
                    rgba[idx + 2] = 0x29;
                    rgba[idx + 3] = 0xFF;
                }
            } else {
                // Transparent exterior
                rgba[idx + 3] = 0x00;
            }
        }
    }

    (rgba, w, h)
}

/// Renders the CADense Hero Banner and Preset CMS Fittings Quick-Launcher on the Home/Landing page.
pub fn render_cadense_home_hero(ui: &mut Ui, on_select_preset: &mut dyn FnMut(&str)) {
    ui.vertical(|ui| {
        // Hero Card Container
        egui::Frame::new()
            .fill(Color32::from_rgb(26, 31, 40))
            .stroke(Stroke::new(1.0, Color32::from_rgb(45, 55, 72)))
            .corner_radius(8.0)
            .inner_margin(egui::Margin::symmetric(24, 16))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Large Badge Emblem
                    let (rect, _) = ui.allocate_exact_size(vec2(48.0, 48.0), Sense::hover());
                    paint_cadense_brand_mark(ui.painter(), rect.center(), 48.0);

                    ui.add_space(16.0);

                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            ui.label(
                                RichText::new("CAD")
                                    .font(FontId::proportional(22.0))
                                    .strong()
                                    .color(CADENSE_SILVER_WHITE),
                            );
                            ui.label(
                                RichText::new("ense")
                                    .font(FontId::proportional(22.0))
                                    .strong()
                                    .color(CADENSE_COBALT_BLUE),
                            );
                            ui.add_space(10.0);
                            ui.label(
                                RichText::new("truFold CMS Studio")
                                    .font(FontId::proportional(14.0))
                                    .color(Color32::from_rgb(148, 163, 184)),
                            );
                        });

                        ui.label(
                            RichText::new("Industrial Sheet-Metal and Cable Management Systems (CMS) Automation Platform")
                                .font(FontId::proportional(12.0))
                                .color(Color32::from_rgb(160, 174, 192)),
                        );
                    });
                });

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);

                // Quick Launch CMS Fitting Presets
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("START FROM CMS TEMPLATE:")
                            .font(FontId::proportional(11.0))
                            .strong()
                            .color(Color32::from_rgb(203, 213, 225)),
                    );

                    let presets = [
                        ("Reducer", "Tapered Reducer (300mm → 150mm)"),
                        ("Tee", "Unequal Tee (Run 300mm, Branch 150mm)"),
                        ("Bend", "90° Standard Bend (200mm)"),
                        ("Cross", "4-Way Cross Fitting (200mm)"),
                    ];

                    for (key, label) in presets {
                        let btn = ui.add(
                            egui::Button::new(
                                RichText::new(format!("⚡ {key}"))
                                    .font(FontId::proportional(11.5))
                                    .color(Color32::WHITE),
                            )
                            .fill(Color32::from_rgb(37, 50, 70))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(59, 130, 246)))
                            .corner_radius(4.0),
                        );
                        if btn.on_hover_text(label).clicked() {
                            on_select_preset(key);
                        }
                    }
                });
            });
    });
}
