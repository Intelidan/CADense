//! CADense Concept 23 Brand Identity & Vector Logo Renderer.
//!
//! Implements Concept 23 (Concept 19 in Deep Cobalt Steel-Blue `#3B82F6` with
//! interlocking T-Bar Tongue and Brushed Silver-White `#E2E8F0` Chamfered `C`
//! inside a dark anthracite octagonal badge `#1E2229`).

use egui::{Color32, Pos2, Rect, RichText, Sense, Shape, Stroke, Ui, Vec2};

pub const CADENSE_COBALT_BLUE: Color32 = Color32::from_rgb(0x3B, 0x82, 0xF6);
pub const CADENSE_COBALT_SHADOW: Color32 = Color32::from_rgb(0x1D, 0x4E, 0xD8);
pub const CADENSE_SILVER_WHITE: Color32 = Color32::from_rgb(0xE2, 0xE8, 0xF0);
pub const CADENSE_SILVER_BEVEL: Color32 = Color32::from_rgb(0x94, 0xA3, 0xB8);
pub const CADENSE_BADGE_BG: Color32 = Color32::from_rgb(0x1E, 0x22, 0x29);
pub const CADENSE_BADGE_BORDER: Color32 = Color32::from_rgb(0x33, 0x39, 0x44);

pub const CONCEPT_23_BADGE_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256">
  <polygon points="76,12 180,12 244,76 244,180 180,244 76,244 12,180 12,76"
           fill="#1E2229" stroke="#333944" stroke-width="4"/>
  <polygon points="122,48 76,48 48,76 48,180 76,208 122,208 122,164 96,164 96,180 82,180 76,174 76,82 82,76 96,76 96,92 122,92"
           fill="#E2E8F0"/>
  <polygon points="122,92 96,92 96,76 106,84 122,84" fill="#94A3B8"/>
  <polygon points="122,164 96,164 96,180 106,172 122,172" fill="#94A3B8"/>
  <path d="M 134,48 L 180,48 L 208,76 L 208,180 L 180,208 L 134,208 L 134,148 L 160,148 L 160,180 L 174,180 L 180,174 L 180,82 L 174,76 L 160,76 L 160,108 L 134,108 Z"
        fill="#3B82F6"/>
  <polygon points="88,128 96,116 164,116 164,102 174,102 174,154 164,154 164,140 96,140"
           fill="#3B82F6"/>
</svg>"##;

pub const CONCEPT_23_METALLIC_3D_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256">
  <defs>
    <linearGradient id="brushedSilver" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#F8FAFC"/>
      <stop offset="50%" stop-color="#CBD5E1"/>
      <stop offset="100%" stop-color="#94A3B8"/>
    </linearGradient>
    <linearGradient id="cobaltSteel" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#60A5FA"/>
      <stop offset="50%" stop-color="#3B82F6"/>
      <stop offset="100%" stop-color="#1D4ED8"/>
    </linearGradient>
  </defs>
  <polygon points="122,44 74,44 44,74 44,182 74,212 122,212 122,162 94,162 94,180 80,180 74,174 74,82 80,76 94,76 94,94 122,94"
           fill="url(#brushedSilver)" stroke="#64748B" stroke-width="2"/>
  <path d="M 134,44 L 182,44 L 212,74 L 212,182 L 182,212 L 134,212 L 134,148 L 162,148 L 162,180 L 176,180 L 182,174 L 182,82 L 176,76 L 162,76 L 162,108 L 134,108 Z"
        fill="url(#cobaltSteel)" stroke="#1E40AF" stroke-width="2"/>
  <polygon points="86,128 95,115 164,115 164,100 175,100 175,156 164,156 164,141 95,141"
           fill="url(#cobaltSteel)" stroke="#1E3A8A" stroke-width="2"/>
</svg>"##;

pub const CONCEPT_23_HORIZONTAL_LOCKUP_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 680 180" width="680" height="180">
  <g transform="translate(10,10) scale(0.625)">
    <polygon points="76,12 180,12 244,76 244,180 180,244 76,244 12,180 12,76"
             fill="#1E2229" stroke="#333944" stroke-width="4"/>
    <polygon points="122,48 76,48 48,76 48,180 76,208 122,208 122,164 96,164 96,180 82,180 76,174 76,82 82,76 96,76 96,92 122,92"
             fill="#E2E8F0"/>
    <path d="M 134,48 L 180,48 L 208,76 L 208,180 L 180,208 L 134,208 L 134,148 L 160,148 L 160,180 L 174,180 L 180,174 L 180,82 L 174,76 L 160,76 L 160,108 L 134,108 Z"
          fill="#3B82F6"/>
    <polygon points="88,128 96,116 164,116 164,102 174,102 174,154 164,154 164,140 96,140"
             fill="#3B82F6"/>
  </g>
  <text x="195" y="118" font-family="Inter, Segoe UI, Arial, sans-serif" font-weight="800" font-size="84">
    <tspan fill="#E2E8F0">CAD</tspan><tspan fill="#3B82F6">ense</tspan>
  </text>
</svg>"##;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CadenseLogoStyle {
    OctagonBadge,
    BeveledEmblem,
}

pub struct CadenseLogoWidget;

impl CadenseLogoWidget {
    pub fn paint_emblem(ui: &mut Ui, size: f32, style: CadenseLogoStyle) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
        if !ui.is_rect_visible(rect) {
            return response;
        }

        let painter = ui.painter_at(rect);
        let map_pt = |nx: f32, ny: f32| -> Pos2 { Pos2::new(rect.min.x + (nx / 256.0) * size, rect.min.y + (ny / 256.0) * size) };

        if style == CadenseLogoStyle::OctagonBadge {
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
            painter.add(Shape::convex_polygon(oct, CADENSE_BADGE_BG, Stroke::new((size * 0.015).max(1.0), CADENSE_BADGE_BORDER)));
        }

        let c_stroke =
            if style == CadenseLogoStyle::BeveledEmblem { Stroke::new((size * 0.012).max(1.0), CADENSE_SILVER_BEVEL) } else { Stroke::NONE };

        let d_stroke =
            if style == CadenseLogoStyle::BeveledEmblem { Stroke::new((size * 0.012).max(1.0), CADENSE_COBALT_SHADOW) } else { Stroke::NONE };

        painter.add(Shape::convex_polygon(
            vec![map_pt(76.0, 48.0), map_pt(122.0, 48.0), map_pt(122.0, 76.0), map_pt(48.0, 76.0)],
            CADENSE_SILVER_WHITE,
            c_stroke,
        ));
        painter.add(Shape::convex_polygon(
            vec![map_pt(48.0, 76.0), map_pt(76.0, 76.0), map_pt(76.0, 180.0), map_pt(48.0, 180.0)],
            CADENSE_SILVER_WHITE,
            c_stroke,
        ));
        painter.add(Shape::convex_polygon(
            vec![map_pt(48.0, 180.0), map_pt(122.0, 180.0), map_pt(122.0, 208.0), map_pt(76.0, 208.0)],
            CADENSE_SILVER_WHITE,
            c_stroke,
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

        painter.add(Shape::convex_polygon(
            vec![map_pt(134.0, 48.0), map_pt(180.0, 48.0), map_pt(208.0, 76.0), map_pt(134.0, 76.0)],
            CADENSE_COBALT_BLUE,
            d_stroke,
        ));
        painter.add(Shape::convex_polygon(
            vec![map_pt(180.0, 76.0), map_pt(208.0, 76.0), map_pt(208.0, 180.0), map_pt(180.0, 180.0)],
            CADENSE_COBALT_BLUE,
            d_stroke,
        ));
        painter.add(Shape::convex_polygon(
            vec![map_pt(134.0, 180.0), map_pt(208.0, 180.0), map_pt(180.0, 208.0), map_pt(134.0, 208.0)],
            CADENSE_COBALT_BLUE,
            d_stroke,
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

        painter.add(Shape::convex_polygon(
            vec![map_pt(86.0, 128.0), map_pt(96.0, 116.0), map_pt(166.0, 116.0), map_pt(166.0, 140.0), map_pt(96.0, 140.0)],
            CADENSE_COBALT_BLUE,
            d_stroke,
        ));
        painter.add(Shape::rect_filled(Rect::from_min_max(map_pt(162.0, 102.0), map_pt(174.0, 154.0)), 2.0, CADENSE_COBALT_BLUE));

        response
    }

    pub fn show_horizontal_lockup(ui: &mut Ui, emblem_size: f32, font_size: f32) {
        ui.horizontal(|ui| {
            Self::paint_emblem(ui, emblem_size, CadenseLogoStyle::OctagonBadge);
            ui.add_space(8.0);
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.label(RichText::new("CAD").size(font_size).strong().color(CADENSE_SILVER_WHITE));
            ui.label(RichText::new("ense").size(font_size).strong().color(CADENSE_COBALT_BLUE));
        });
    }

    pub fn generate_window_icon_rgba() -> (Vec<u8>, u32, u32) {
        let w = 64u32;
        let h = 64u32;
        let mut rgba = vec![0u8; (w * h * 4) as usize];

        for y in 0..h {
            for x in 0..w {
                let nx = (x as f32 / w as f32) * 256.0;
                let ny = (y as f32 / h as f32) * 256.0;

                let in_oct =
                    nx >= 12.0 && nx <= 244.0 && ny >= 12.0 && ny <= 244.0 && (nx + ny) >= 88.0 && (nx + ny) <= 424.0 && (nx - ny).abs() <= 168.0;

                if !in_oct {
                    continue;
                }

                let in_c = (nx >= 48.0 && nx <= 122.0 && ny >= 48.0 && ny <= 76.0)
                    || (nx >= 48.0 && nx <= 76.0 && ny >= 76.0 && ny <= 180.0)
                    || (nx >= 48.0 && nx <= 122.0 && ny >= 180.0 && ny <= 208.0)
                    || (nx >= 96.0 && nx <= 122.0 && (ny >= 76.0 && ny <= 94.0 || ny >= 162.0 && ny <= 180.0));

                let in_d = (nx >= 134.0 && nx <= 208.0 && ny >= 48.0 && ny <= 76.0)
                    || (nx >= 180.0 && nx <= 208.0 && ny >= 76.0 && ny <= 180.0)
                    || (nx >= 134.0 && nx <= 208.0 && ny >= 180.0 && ny <= 208.0)
                    || (nx >= 134.0 && nx <= 160.0 && (ny >= 76.0 && ny <= 108.0 || ny >= 148.0 && ny <= 180.0));

                let in_tbar = (nx >= 88.0 && nx <= 166.0 && ny >= 116.0 && ny <= 140.0) || (nx >= 162.0 && nx <= 174.0 && ny >= 102.0 && ny <= 154.0);

                let (r, g, b, a) = if in_tbar || in_d {
                    (0x3B, 0x82, 0xF6, 0xFF)
                } else if in_c {
                    (0xE2, 0xE8, 0xF0, 0xFF)
                } else {
                    (0x1E, 0x22, 0x29, 0xFF)
                };

                let idx = ((y * w + x) * 4) as usize;
                rgba[idx] = r;
                rgba[idx + 1] = g;
                rgba[idx + 2] = b;
                rgba[idx + 3] = a;
            }
        }

        (rgba, w, h)
    }
}
