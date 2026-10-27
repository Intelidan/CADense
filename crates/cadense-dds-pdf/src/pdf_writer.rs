use std::fmt::Write as FmtWrite;
use std::io::Write;

/// Lightweight native PDF 1.4 vector stream generator.
pub struct PdfStreamWriter {
    pub buffer: Vec<u8>,
    pub current_page_stream: String,
    pub page_object_ids: Vec<usize>,
    pub object_offsets: Vec<usize>,
}

impl Default for PdfStreamWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl PdfStreamWriter {
    pub fn new() -> Self {
        let mut w = Self {
            buffer: Vec::new(),
            current_page_stream: String::new(),
            page_object_ids: Vec::new(),
            object_offsets: Vec::new(),
        };
        w.write_header();
        w
    }

    fn write_header(&mut self) {
        self.buffer.extend_from_slice(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");
    }

    pub fn start_page(&mut self) {
        self.current_page_stream.clear();
    }

    pub fn set_line_width(&mut self, w: f32) {
        let _ = write!(self.current_page_stream, "{:.2} w\n", w);
    }

    pub fn set_stroke_color(&mut self, r: f32, g: f32, b: f32) {
        let _ = write!(self.current_page_stream, "{:.3} {:.3} {:.3} RG\n", r, g, b);
    }

    pub fn set_fill_color(&mut self, r: f32, g: f32, b: f32) {
        let _ = write!(self.current_page_stream, "{:.3} {:.3} {:.3} rg\n", r, g, b);
    }

    pub fn draw_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        let _ = write!(
            self.current_page_stream,
            "{:.2} {:.2} m {:.2} {:.2} l S\n",
            x1, y1, x2, y2
        );
    }

    pub fn draw_rect(&mut self, x: f64, y: f64, w: f64, h: f64) {
        let _ = write!(
            self.current_page_stream,
            "{:.2} {:.2} {:.2} {:.2} re S\n",
            x, y, w, h
        );
    }

    pub fn fill_polygon(&mut self, pts: &[[f64; 2]]) {
        if pts.is_empty() { return; }
        let _ = write!(self.current_page_stream, "{:.2} {:.2} m\n", pts[0][0], pts[0][1]);
        for p in &pts[1..] {
            let _ = write!(self.current_page_stream, "{:.2} {:.2} l\n", p[0], p[1]);
        }
        self.current_page_stream.push_str("f\n");
    }

    pub fn draw_text(&mut self, x: f32, y: f32, text: &str, size_pt: f32) {
        let sanitized = text.replace('\\', "\\\\").replace('(', "\\(").replace(')', "\\)");
        let _ = write!(
            self.current_page_stream,
            "BT /F1 {:.1} Tf {:.2} {:.2} Td ({}) Tj ET\n",
            size_pt, x, y, sanitized
        );
    }

    pub fn draw_text_bold(&mut self, x: f32, y: f32, text: &str, size_pt: f32) {
        let sanitized = text.replace('\\', "\\\\").replace('(', "\\(").replace(')', "\\)");
        let _ = write!(
            self.current_page_stream,
            "BT /F2 {:.1} Tf {:.2} {:.2} Td ({}) Tj ET\n",
            size_pt, x, y, sanitized
        );
    }

    pub fn draw_text_centered(&mut self, cx: f32, cy: f32, text: &str, size_pt: f32) {
        // Approximate character width for standard Helvetica (0.52 * font size)
        let est_w = text.len() as f32 * size_pt * 0.52;
        self.draw_text(cx - est_w * 0.5, cy, text, size_pt);
    }

    pub fn finish_page(&mut self) {
        let content_id = self.allocate_object_id();
        let content_bytes = self.current_page_stream.as_bytes().to_vec();

        self.start_object(content_id);
        let _ = write!(
            self.buffer,
            "<< /Length {} >>\nstream\n",
            content_bytes.len()
        );
        self.buffer.extend_from_slice(&content_bytes);
        self.buffer.extend_from_slice(b"\nendstream\nendobj\n");

        let page_id = self.allocate_object_id();
        self.page_object_ids.push(page_id);

        self.start_object(page_id);
        let _ = write!(
            self.buffer,
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595.28 420.94] /Contents {} 0 R /Resources << /Font << /F1 3 0 R /F2 4 0 R >> >> >>\nendobj\n",
            content_id
        );
    }

    fn allocate_object_id(&self) -> usize {
        self.object_offsets.len() + 1
    }

    fn start_object(&mut self, id: usize) {
        while self.object_offsets.len() < id {
            self.object_offsets.push(0);
        }
        self.object_offsets[id - 1] = self.buffer.len();
        let _ = write!(self.buffer, "{} 0 obj\n", id);
    }

    pub fn finalize(mut self) -> Vec<u8> {
        let catalog_id = self.allocate_object_id();
        let pages_id = self.allocate_object_id();
        let font_f1 = self.allocate_object_id();
        let font_f2 = self.allocate_object_id();

        // 1. Catalog Object
        self.start_object(catalog_id);
        let _ = write!(
            self.buffer,
            "<< /Type /Catalog /Pages {} 0 R >>\nendobj\n",
            pages_id
        );

        // 2. Pages Root Object
        self.start_object(pages_id);
        let kids_str = self
            .page_object_ids
            .iter()
            .map(|id| format!("{} 0 R", id))
            .collect::<Vec<_>>()
            .join(" ");
        let _ = write!(
            self.buffer,
            "<< /Type /Pages /Kids [{}] /Count {} >>\nendobj\n",
            kids_str,
            self.page_object_ids.len()
        );

        // 3. Fonts (Helvetica Standard + Bold)
        self.start_object(font_f1);
        self.buffer.extend_from_slice(
            b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n",
        );
        self.start_object(font_f2);
        self.buffer.extend_from_slice(
            b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold >>\nendobj\n",
        );

        // 4. Cross-Reference Table
        let xref_offset = self.buffer.len();
        let total_objects = self.object_offsets.len() + 1;
        let _ = write!(self.buffer, "xref\n0 {}\n0000000000 65535 f \n", total_objects);
        for offset in &self.object_offsets {
            let _ = write!(self.buffer, "{:010} 00000 n \n", offset);
        }

        // 5. Trailer
        let _ = write!(
            self.buffer,
            "trailer\n<< /Size {} /Root {} 0 R >>\nstartxref\n{}\n%%EOF\n",
            total_objects, catalog_id, xref_offset
        );

        self.buffer
    }
}