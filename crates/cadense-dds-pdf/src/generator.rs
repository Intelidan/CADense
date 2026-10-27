use crate::dds::{DdsArchive, DdsItemRow};
use crate::dimensions::{DimensionStyle, DimensionSystem};
use crate::hlr::{HlrEngine, IsoCamera, LineKind, Mesh3D, Triangle3D};
use crate::insets::{InsetKind, SectionalInset};
use crate::pdf_writer::PdfStreamWriter;
use cadense_dsf::CmsCategory;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageMode {
    OnePagePerRow,
    GroupedSubmittal,
}

pub struct SubmittalPageSpec<'a> {
    pub item: &'a DdsItemRow,
    pub grouped_items: Vec<&'a DdsItemRow>,
    pub category: CmsCategory,
    pub is_grouped: bool,
}

pub struct BonnMetalSubmittalGenerator;

impl BonnMetalSubmittalGenerator {
    /// Generates a submittal PDF document according to the specified `PageMode`.
    pub fn generate_pdf(archive: &DdsArchive, mode: PageMode) -> Vec<u8> {
        let mut pdf = PdfStreamWriter::new();

        match mode {
            PageMode::OnePagePerRow => {
                for item in &archive.manifest.item_schedule {
                    let spec =
                        SubmittalPageSpec { item, grouped_items: vec![item], category: detect_category(&item.product_code), is_grouped: false };
                    Self::render_page(&mut pdf, archive, &spec);
                }
            }
            PageMode::GroupedSubmittal => {
                let mut groups: std::collections::BTreeMap<String, Vec<&DdsItemRow>> = std::collections::BTreeMap::new();
                for item in &archive.manifest.item_schedule {
                    groups.entry(item.product_code.clone()).or_default().push(item);
                }

                for (_, items) in groups {
                    if let Some(first) = items.first() {
                        let spec = SubmittalPageSpec {
                            item: first,
                            grouped_items: items.clone(),
                            category: detect_category(&first.product_code),
                            is_grouped: true,
                        };
                        Self::render_page(&mut pdf, archive, &spec);
                    }
                }
            }
        }

        pdf.finalize()
    }

    fn render_page(pdf: &mut PdfStreamWriter, archive: &DdsArchive, spec: &SubmittalPageSpec) {
        pdf.start_page();

        // 1. Sheet Border & Inner Margin (A4 Landscape: 595.28 x 420.94 pt)
        pdf.set_stroke_color(0.1, 0.1, 0.1);
        pdf.set_line_width(1.5);
        pdf.draw_rect(20.0, 20.0, 555.28, 380.94);
        pdf.set_line_width(0.75);
        pdf.draw_rect(24.0, 24.0, 547.28, 372.94);

        // 2. Title Block Frame
        Self::draw_bonn_title_block(pdf, archive, spec);

        // 3. Sectional View Inset
        if let Some(inset_kind) = resolve_inset_kind(spec.item, spec.category) {
            let inset = SectionalInset::new(inset_kind, [32.0, 390.0]);
            inset.render_to_pdf(pdf);
        }

        // 4. Generate 3D Mesh & HLR Vector Isometric Projection
        let mesh = build_synthetic_cms_mesh(spec.item);
        let mut hlr = HlrEngine::new(IsoCamera::default());
        // Viewport allocation: X: 190 -> 560, Y: 130 -> 380
        hlr.fit_bounds(&mesh, [190.0, 130.0, 560.0, 380.0]);

        let lines = hlr.render_vector_line_art(&mesh);
        for l in lines {
            match l.kind {
                LineKind::VisibleSharp | LineKind::VisibleSilhouette => {
                    pdf.set_stroke_color(0.05, 0.05, 0.05);
                    pdf.set_line_width(1.0);
                    pdf.draw_line(l.start[0], l.start[1], l.end[0], l.end[1]);
                }
                LineKind::BendLineDashed => {
                    pdf.set_stroke_color(0.3, 0.3, 0.3);
                    pdf.set_line_width(0.75);
                    pdf.draw_line(l.start[0], l.start[1], l.end[0], l.end[1]);
                }
                LineKind::Hidden => {}
            }
        }

        // 5. Red Dimension Annotations
        let mut dims = DimensionSystem::new(DimensionStyle::default());
        Self::populate_dimensions(&mut dims, &hlr, spec);
        dims.render_to_pdf(pdf);

        // 6. Specification Schedule Table
        Self::draw_schedule_table(pdf, spec);

        pdf.finish_page();
    }

    fn draw_bonn_title_block(pdf: &mut PdfStreamWriter, archive: &DdsArchive, spec: &SubmittalPageSpec) {
        let bx = 24.0;
        let by = 24.0;
        let bw = 547.28;
        let bh = 75.0;

        pdf.set_stroke_color(0.1, 0.1, 0.1);
        pdf.set_line_width(1.0);
        pdf.draw_rect(bx, by, bw, bh);

        // Vertical dividers
        pdf.draw_line(bx + 180.0, by, bx + 180.0, by + bh);
        pdf.draw_line(bx + 380.0, by, bx + 380.0, by + bh);

        // Box 1: Company Branding
        pdf.set_fill_color(0.08, 0.22, 0.42); // Bonn Navy Blue
        pdf.draw_text_bold((bx + 10.0) as f32, (by + 52.0) as f32, "BONN METAL CONS. IND.", 11.0);
        pdf.set_fill_color(0.3, 0.3, 0.3);
        pdf.draw_text((bx + 10.0) as f32, (by + 38.0) as f32, "BONN GROUP OF COMPANIES — UAE", 8.0);
        pdf.draw_text((bx + 10.0) as f32, (by + 26.0) as f32, "TECHNICAL DATA SHEET / SUBMITTAL", 8.0);
        pdf.draw_text((bx + 10.0) as f32, (by + 12.0) as f32, "BS EN 61537 / NEMA VE-1 COMPLIANT", 7.0);

        // Box 2: Product Title & Description
        pdf.set_fill_color(0.0, 0.0, 0.0);
        pdf.draw_text_bold((bx + 190.0) as f32, (by + 55.0) as f32, &spec.item.product_code, 12.0);
        pdf.draw_text((bx + 190.0) as f32, (by + 38.0) as f32, &spec.item.description, 8.5);
        pdf.set_fill_color(0.85, 0.1, 0.1);
        pdf.draw_text_bold((bx + 190.0) as f32, (by + 15.0) as f32, &format!("NOTE:- MATERIAL FINISH {}", spec.item.finish), 8.5);

        // Box 3: Project Meta & Signoff
        pdf.set_fill_color(0.2, 0.2, 0.2);
        pdf.draw_text((bx + 390.0) as f32, (by + 58.0) as f32, &format!("PROJECT: {}", archive.manifest.meta.project_name), 7.5);
        pdf.draw_text((bx + 390.0) as f32, (by + 46.0) as f32, &format!("CLIENT: {}", archive.manifest.meta.customer_name), 7.5);
        pdf.draw_text((bx + 390.0) as f32, (by + 34.0) as f32, &format!("ORDER REF: {}", archive.manifest.meta.order_reference), 7.5);
        pdf.draw_text((bx + 390.0) as f32, (by + 22.0) as f32, &format!("DATE: {}", archive.manifest.meta.date_created), 7.5);
        pdf.draw_text((bx + 390.0) as f32, (by + 10.0) as f32, &format!("SN: {:02} | QTY: {}", spec.item.sn, spec.item.qty), 7.5);
    }

    fn draw_schedule_table(pdf: &mut PdfStreamWriter, spec: &SubmittalPageSpec) {
        let tx = 32.0;
        let ty = 105.0;
        let tw = 280.0;
        let th = 55.0;

        pdf.set_stroke_color(0.2, 0.2, 0.2);
        pdf.set_line_width(0.65);
        pdf.draw_rect(tx, ty, tw, th);

        // Headers
        pdf.set_fill_color(0.92, 0.94, 0.96);
        pdf.fill_polygon(&[[tx, ty + th], [tx + tw, ty + th], [tx + tw, ty + th - 14.0], [tx, ty + th - 14.0]]);
        pdf.set_fill_color(0.0, 0.0, 0.0);
        pdf.draw_text_bold((tx + 6.0) as f32, (ty + th - 10.0) as f32, "SN", 7.0);
        pdf.draw_text_bold((tx + 28.0) as f32, (ty + th - 10.0) as f32, "W1 (mm)", 7.0);
        pdf.draw_text_bold((tx + 75.0) as f32, (ty + th - 10.0) as f32, "W2 (mm)", 7.0);
        pdf.draw_text_bold((tx + 120.0) as f32, (ty + th - 10.0) as f32, "H (mm)", 7.0);
        pdf.draw_text_bold((tx + 165.0) as f32, (ty + th - 10.0) as f32, "T (mm)", 7.0);
        pdf.draw_text_bold((tx + 215.0) as f32, (ty + th - 10.0) as f32, "QTY", 7.0);

        let mut row_y = ty + th - 24.0;
        for it in spec.grouped_items.iter().take(4) {
            let w1 = it.dimensions.get("W1").or_else(|| it.dimensions.get("W")).copied().unwrap_or(0.0);
            let w2 = it.dimensions.get("W2").copied().unwrap_or(w1);
            let h = it.dimensions.get("H").copied().unwrap_or(50.0);

            pdf.draw_text((tx + 6.0) as f32, row_y as f32, &format!("{:02}", it.sn), 7.0);
            pdf.draw_text((tx + 28.0) as f32, row_y as f32, &format!("{:.0}", w1), 7.0);
            pdf.draw_text((tx + 75.0) as f32, row_y as f32, &format!("{:.0}", w2), 7.0);
            pdf.draw_text((tx + 120.0) as f32, row_y as f32, &format!("{:.0}", h), 7.0);
            pdf.draw_text((tx + 165.0) as f32, row_y as f32, &format!("{:.1}", it.thickness_mm), 7.0);
            pdf.draw_text((tx + 215.0) as f32, row_y as f32, &format!("{}", it.qty), 7.0);

            row_y -= 10.0;
        }
    }

    fn populate_dimensions(dims: &mut DimensionSystem, hlr: &HlrEngine, spec: &SubmittalPageSpec) {
        let item = spec.item;
        let w1 = item.dimensions.get("W1").or_else(|| item.dimensions.get("W")).copied().unwrap_or(300.0);
        let h = item.dimensions.get("H").copied().unwrap_or(50.0);

        // Primary Width Dimension (Port 1 mouth)
        let (p1, _) = hlr.camera.project_point([-w1 * 0.5, 0.0, 0.0]);
        let (p2, _) = hlr.camera.project_point([w1 * 0.5, 0.0, 0.0]);
        let label = if spec.is_grouped { "WIDTH-1".to_string() } else { format!("{:.0} mm", w1) };
        dims.add_linear(p1, p2, -25.0, label);

        // Height Dimension
        let (hp1, _) = hlr.camera.project_point([w1 * 0.5, 0.0, 0.0]);
        let (hp2, _) = hlr.camera.project_point([w1 * 0.5, 0.0, h]);
        let h_label = if spec.is_grouped { "HEIGHT".to_string() } else { format!("{:.0} mm", h) };
        dims.add_linear(hp1, hp2, 20.0, h_label);

        // Obround Coupler Slot Callout
        let (sp, _) = hlr.camera.project_point([w1 * 0.5, 25.0, h * 0.5]);
        dims.add_radial_or_slot(sp, [35.0, 25.0], "OBROUND 20X8");
    }
}

fn detect_category(code: &str) -> CmsCategory {
    if code.starts_with("CT") {
        CmsCategory::CableTray
    } else if code.starts_with("TRK") {
        CmsCategory::CableTrunking
    } else if code.starts_with("UNI") {
        CmsCategory::UnistrutChannel
    } else {
        CmsCategory::AccessoryCouplerOrEndCap
    }
}

fn resolve_inset_kind(item: &DdsItemRow, cat: CmsCategory) -> Option<InsetKind> {
    match cat {
        CmsCategory::CableTray => Some(InsetKind::TrayInsideReturnFlange11mm { thickness_mm: item.thickness_mm }),
        CmsCategory::CableTrunking => Some(InsetKind::CoverLip10mm { thickness_mm: item.thickness_mm }),
        CmsCategory::UnistrutChannel => Some(InsetKind::UnistrutChannel41x41 { thickness_mm: item.thickness_mm }),
        CmsCategory::AccessoryCouplerOrEndCap => None,
    }
}

fn build_synthetic_cms_mesh(item: &DdsItemRow) -> Mesh3D {
    let mut mesh = Mesh3D::default();
    let w1 = item.dimensions.get("W1").or_else(|| item.dimensions.get("W")).copied().unwrap_or(300.0);
    let h = item.dimensions.get("H").copied().unwrap_or(50.0);
    let l = item.dimensions.get("L").copied().unwrap_or(250.0);

    let hw = w1 * 0.5;
    let hl = l * 0.5;

    // Corner vertices
    let v0 = [-hw, -hl, 0.0];
    let v1 = [hw, -hl, 0.0];
    let v2 = [hw, hl, 0.0];
    let v3 = [-hw, hl, 0.0];

    let v4 = [-hw, -hl, h];
    let v5 = [hw, -hl, h];
    let v6 = [hw, hl, h];
    let v7 = [-hw, hl, h];

    // Base plate triangles
    mesh.triangles.push(Triangle3D::new(v0, v1, v2));
    mesh.triangles.push(Triangle3D::new(v0, v2, v3));

    // Side wall triangles
    mesh.triangles.push(Triangle3D::new(v1, v5, v6));
    mesh.triangles.push(Triangle3D::new(v1, v6, v2));

    mesh.triangles.push(Triangle3D::new(v0, v3, v7));
    mesh.triangles.push(Triangle3D::new(v0, v7, v4));

    // Outer boundary edges
    mesh.edges.push((v0, v1));
    mesh.edges.push((v1, v2));
    mesh.edges.push((v2, v3));
    mesh.edges.push((v3, v0));

    mesh.edges.push((v4, v5));
    mesh.edges.push((v5, v6));
    mesh.edges.push((v6, v7));
    mesh.edges.push((v7, v4));

    mesh.edges.push((v0, v4));
    mesh.edges.push((v1, v5));
    mesh.edges.push((v2, v6));
    mesh.edges.push((v3, v7));

    mesh
}
