use crate::pdf_writer::PdfStreamWriter;

#[derive(Debug, Clone)]
pub struct DimensionStyle {
    pub line_color_rgb: [f32; 3],
    pub line_width_pt: f32,
    pub text_size_pt: f32,
    pub arrow_size_pt: f32,
    pub extension_overshoot_pt: f32,
}

impl Default for DimensionStyle {
    fn default() -> Self {
        Self {
            line_color_rgb: [0.85, 0.10, 0.10], // Technical Drawing Red (#D91A1A)
            line_width_pt: 0.85,
            text_size_pt: 8.5,
            arrow_size_pt: 6.0,
            extension_overshoot_pt: 5.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DimensionAnnotation {
    pub p1_2d: [f64; 2],
    pub p2_2d: [f64; 2],
    pub offset_distance_pt: f64,
    pub label_text: String,
    pub leader_angle_deg: Option<f64>,
}

pub struct DimensionSystem {
    pub style: DimensionStyle,
    pub dimensions: Vec<DimensionAnnotation>,
}

impl DimensionSystem {
    pub fn new(style: DimensionStyle) -> Self {
        Self { style, dimensions: Vec::new() }
    }

    pub fn add_linear(&mut self, p1: [f64; 2], p2: [f64; 2], offset_pt: f64, label: impl Into<String>) {
        self.dimensions.push(DimensionAnnotation {
            p1_2d: p1,
            p2_2d: p2,
            offset_distance_pt: offset_pt,
            label_text: label.into(),
            leader_angle_deg: None,
        });
    }

    pub fn add_radial_or_slot(&mut self, center: [f64; 2], leader_vec: [f64; 2], label: impl Into<String>) {
        let p2 = [center[0] + leader_vec[0], center[1] + leader_vec[1]];
        self.dimensions.push(DimensionAnnotation {
            p1_2d: center,
            p2_2d: p2,
            offset_distance_pt: 0.0,
            label_text: label.into(),
            leader_angle_deg: Some(45.0),
        });
    }

    /// Renders red dimension callouts directly into the PDF vector stream.
    pub fn render_to_pdf(&self, pdf: &mut PdfStreamWriter) {
        pdf.set_stroke_color(self.style.line_color_rgb[0], self.style.line_color_rgb[1], self.style.line_color_rgb[2]);
        pdf.set_fill_color(self.style.line_color_rgb[0], self.style.line_color_rgb[1], self.style.line_color_rgb[2]);
        pdf.set_line_width(self.style.line_width_pt);

        for dim in &self.dimensions {
            let dx = dim.p2_2d[0] - dim.p1_2d[0];
            let dy = dim.p2_2d[1] - dim.p1_2d[1];
            let len = (dx * dx + dy * dy).sqrt().max(1e-6);

            // Unit perpendicular vector
            let nx = -dy / len;
            let ny = dx / len;

            let off = dim.offset_distance_pt;
            let over = self.style.extension_overshoot_pt as f64;

            // Offset dimension line coordinates
            let d1 = [dim.p1_2d[0] + nx * off, dim.p1_2d[1] + ny * off];
            let d2 = [dim.p2_2d[0] + nx * off, dim.p2_2d[1] + ny * off];

            // 1. Witness Extension Lines
            let sign = if off >= 0.0 { 1.0 } else { -1.0 };
            pdf.draw_line(dim.p1_2d[0], dim.p1_2d[1], d1[0] + nx * over * sign, d1[1] + ny * over * sign);
            pdf.draw_line(dim.p2_2d[0], dim.p2_2d[1], d2[0] + nx * over * sign, d2[1] + ny * over * sign);

            // 2. Main Dimension Line
            pdf.draw_line(d1[0], d1[1], d2[0], d2[1]);

            // 3. Arrowheads
            self.draw_arrow(pdf, d1, [d2[0] - d1[0], d2[1] - d1[1]]);
            self.draw_arrow(pdf, d2, [d1[0] - d2[0], d1[1] - d2[1]]);

            // 4. Centered Dimension Text Callout
            let mid_x = (d1[0] + d2[0]) * 0.5 + nx * 5.0;
            let mid_y = (d1[1] + d2[1]) * 0.5 + ny * 5.0;

            pdf.draw_text_centered(mid_x as f32, mid_y as f32, &dim.label_text, self.style.text_size_pt);
        }
    }

    fn draw_arrow(&self, pdf: &mut PdfStreamWriter, tip: [f64; 2], dir: [f64; 2]) {
        let len = (dir[0] * dir[0] + dir[1] * dir[1]).sqrt().max(1e-6);
        let ux = dir[0] / len;
        let uy = dir[1] / len;
        let nx = -uy;
        let ny = ux;

        let asz = self.style.arrow_size_pt as f64;
        let base = [tip[0] + ux * asz, tip[1] + uy * asz];
        let p_left = [base[0] + nx * (asz * 0.35), base[1] + ny * (asz * 0.35)];
        let p_right = [base[0] - nx * (asz * 0.35), base[1] - ny * (asz * 0.35)];

        pdf.fill_polygon(&[tip, p_left, p_right]);
    }
}
