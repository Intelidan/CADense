use crate::pdf_writer::PdfStreamWriter;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InsetKind {
    TrayInsideReturnFlange11mm { thickness_mm: f64 },
    CoverLip15mm { thickness_mm: f64 },
    CoverLip10mm { thickness_mm: f64 },
    UnistrutChannel41x41 { thickness_mm: f64 },
}

pub struct SectionalInset {
    pub kind: InsetKind,
    pub top_left_origin: [f64; 2],
    pub box_width_pt: f64,
    pub box_height_pt: f64,
}

impl SectionalInset {
    pub fn new(kind: InsetKind, origin: [f64; 2]) -> Self {
        Self { kind, top_left_origin: origin, box_width_pt: 155.0, box_height_pt: 120.0 }
    }

    pub fn render_to_pdf(&self, pdf: &mut PdfStreamWriter) {
        let x = self.top_left_origin[0];
        let y = self.top_left_origin[1];
        let w = self.box_width_pt;
        let h = self.box_height_pt;

        // Inset boundary frame
        pdf.set_stroke_color(0.2, 0.2, 0.2);
        pdf.set_line_width(0.75);
        pdf.draw_rect(x, y - h, w, h);

        // Header Title
        pdf.set_fill_color(0.0, 0.0, 0.0);
        pdf.draw_text_bold((x + 10.0) as f32, (y - 14.0) as f32, "SECTIONAL VIEW", 9.0);

        match self.kind {
            InsetKind::TrayInsideReturnFlange11mm { thickness_mm } => {
                self.draw_tray_flange_inset(pdf, x, y, thickness_mm);
            }
            InsetKind::CoverLip15mm { thickness_mm } => {
                self.draw_cover_lip_inset(pdf, x, y, 15.0, thickness_mm);
            }
            InsetKind::CoverLip10mm { thickness_mm } => {
                self.draw_cover_lip_inset(pdf, x, y, 10.0, thickness_mm);
            }
            InsetKind::UnistrutChannel41x41 { thickness_mm } => {
                self.draw_unistrut_inset(pdf, x, y, thickness_mm);
            }
        }
    }

    fn draw_tray_flange_inset(&self, pdf: &mut PdfStreamWriter, x: f64, y: f64, t: f64) {
        let cx = x + 40.0;
        let cy = y - 95.0;

        pdf.set_stroke_color(0.1, 0.1, 0.1);
        pdf.set_line_width(1.5);

        // Profile: Bottom base -> Upright wall -> Inside 11mm return bend
        pdf.draw_line(cx, cy, cx + 55.0, cy);
        pdf.draw_line(cx, cy, cx, cy + 60.0);
        pdf.draw_line(cx, cy + 60.0, cx + 22.0, cy + 60.0);
        pdf.draw_line(cx + 22.0, cy + 60.0, cx + 22.0, cy + 45.0);

        // Red dimension callout for 11mm flange
        pdf.set_stroke_color(0.85, 0.1, 0.1);
        pdf.set_line_width(0.8);
        pdf.draw_line(cx + 28.0, cy + 60.0, cx + 28.0, cy + 45.0);
        pdf.draw_text((cx + 32.0) as f32, (cy + 50.0) as f32, "11 mm", 8.0);
        pdf.draw_text((x + 10.0) as f32, (y - 110.0) as f32, &format!("T = {:.1} mm", t), 8.0);
    }

    fn draw_cover_lip_inset(&self, pdf: &mut PdfStreamWriter, x: f64, y: f64, lip_mm: f64, t: f64) {
        let cx = x + 35.0;
        let cy = y - 70.0;

        pdf.set_stroke_color(0.1, 0.1, 0.1);
        pdf.set_line_width(1.5);

        // Inverted U-shape cover lip
        pdf.draw_line(cx, cy, cx + 70.0, cy);
        pdf.draw_line(cx, cy, cx, cy - 30.0);
        pdf.draw_line(cx + 70.0, cy, cx + 70.0, cy - 30.0);

        pdf.set_stroke_color(0.85, 0.1, 0.1);
        pdf.draw_line(cx - 8.0, cy, cx - 8.0, cy - 30.0);
        pdf.draw_text((cx - 28.0) as f32, (cy - 18.0) as f32, &format!("{:.0} mm", lip_mm), 8.0);
        pdf.draw_text((x + 10.0) as f32, (y - 110.0) as f32, &format!("COVER LIP | T={:.1}mm", t), 8.0);
    }

    fn draw_unistrut_inset(&self, pdf: &mut PdfStreamWriter, x: f64, y: f64, t: f64) {
        let cx = x + 50.0;
        let cy = y - 90.0;

        pdf.set_stroke_color(0.1, 0.1, 0.1);
        pdf.set_line_width(1.5);

        // 41x41 Channel cross-section with inward return lips
        pdf.draw_line(cx, cy, cx + 55.0, cy);
        pdf.draw_line(cx, cy, cx, cy + 55.0);
        pdf.draw_line(cx + 55.0, cy, cx + 55.0, cy + 55.0);
        pdf.draw_line(cx, cy + 55.0, cx + 15.0, cy + 55.0);
        pdf.draw_line(cx + 55.0, cy + 55.0, cx + 40.0, cy + 55.0);
        pdf.draw_line(cx + 15.0, cy + 55.0, cx + 15.0, cy + 45.0);
        pdf.draw_line(cx + 40.0, cy + 55.0, cx + 40.0, cy + 45.0);

        pdf.set_stroke_color(0.85, 0.1, 0.1);
        pdf.draw_text((x + 10.0) as f32, (y - 110.0) as f32, &format!("41 x 41 x {:.1} mm", t), 8.0);
    }
}
