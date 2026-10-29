//! Manufacturing Exporters: Layer-Segregated DXF Flat Patterns and 3D STEP Models.
//!
//! Produces machine-ready CNC laser, plasma, punch, and press brake output:
//! - 2D Flat DXF: Layer-segregated vectors into `CMS_FLAT_OUTLINE`, `CMS_PUNCH_HOLES`, and `CMS_BEND_LINES`.
//! - 3D STEP: ISO 10303-21 STEP faceted B-Rep solid model representing relaxed folded geometry.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use thiserror::Error;

use crate::data::{CmsFoldParams, CmsSlotMetadata};
use crate::eval::RelaxedFoldResult;
use crate::trace::StructuralWireLoop;

#[derive(Error, Debug)]
pub enum ExportError {
    #[error("I/O error during export: {0}")]
    Io(#[from] std::io::Error),
    #[error("Geometry error: {0}")]
    Geometry(String),
}

// =============================================================================
// 1. 2D FLAT DXF EXPORTER (LAYER-SEGREGATED FOR LASER & PUNCH CNC)
// =============================================================================

/// Configuration options for 2D Flat DXF CAM export.
#[derive(Clone, Debug)]
pub struct DxfExportOptions {
    /// Layer name for outer cut perimeter (default: "CMS_FLAT_OUTLINE").
    pub outline_layer: String,
    /// Color index for outer cut perimeter (AutoCAD color 7 = White/Black, 3 = Green).
    pub outline_color: i16,
    /// Layer name for interior punch cutouts and coupler slots (default: "CMS_PUNCH_HOLES").
    pub punch_layer: String,
    /// Color index for punch cutouts (AutoCAD color 1 = Red).
    pub punch_color: i16,
    /// Layer name for press brake bend lines (default: "CMS_BEND_LINES").
    pub bend_layer: String,
    /// Color index for bend lines (AutoCAD color 4 = Cyan).
    pub bend_color: i16,
}

impl Default for DxfExportOptions {
    fn default() -> Self {
        Self {
            outline_layer: "CMS_FLAT_OUTLINE".to_string(),
            outline_color: 7,
            punch_layer: "CMS_PUNCH_HOLES".to_string(),
            punch_color: 1,
            bend_layer: "CMS_BEND_LINES".to_string(),
            bend_color: 4,
        }
    }
}

/// Generates layer-segregated 2D DXF string content (AutoCAD R12 compatible).
pub fn generate_2d_flat_dxf_string(
    outer_loop: &StructuralWireLoop,
    slots: &[CmsSlotMetadata],
    bends: &[CmsFoldParams],
    options: &DxfExportOptions,
) -> String {
    let mut out = String::with_capacity(8192);

    // 1. Header Section
    out.push_str("0\nSECTION\n2\nHEADER\n");
    out.push_str("9\n$ACADVER\n1\nAC1009\n"); // AutoCAD R12 ASCII (universal CAM compatibility)
    out.push_str("9\n$INSUNITS\n70\n4\n");   // 4 = Millimeters
    out.push_str("0\nENDSEC\n");

    // 2. Tables Section (Layer definitions)
    out.push_str("0\nSECTION\n2\nTABLES\n");
    out.push_str("0\nTABLE\n2\nLAYER\n70\n3\n");

    // Layer 1: Outer Outline
    out.push_str("0\nLAYER\n2\n");
    out.push_str(&options.outline_layer);
    out.push_str("\n70\n0\n62\n");
    out.push_str(&options.outline_color.to_string());
    out.push_str("\n6\nCONTINUOUS\n");

    // Layer 2: Punch Holes
    out.push_str("0\nLAYER\n2\n");
    out.push_str(&options.punch_layer);
    out.push_str("\n70\n0\n62\n");
    out.push_str(&options.punch_color.to_string());
    out.push_str("\n6\nCONTINUOUS\n");

    // Layer 3: Bend Lines
    out.push_str("0\nLAYER\n2\n");
    out.push_str(&options.bend_layer);
    out.push_str("\n70\n0\n62\n");
    out.push_str(&options.bend_color.to_string());
    out.push_str("\n6\nDASHED\n");

    out.push_str("0\nENDTAB\n0\nENDSEC\n");

    // 3. Entities Section
    out.push_str("0\nSECTION\n2\nENTITIES\n");

    // A. Outer Perimeter Closed Loop
    let n_verts = outer_loop.vertices.len();
    if n_verts >= 3 {
        for i in 0..n_verts {
            let p1 = outer_loop.vertices[i];
            let p2 = outer_loop.vertices[(i + 1) % n_verts];
            append_dxf_line(&mut out, &options.outline_layer, p1.0, p1.1, p2.0, p2.1);
        }
    }

    // B. Interior Coupler Slots and Punch Holes
    for slot in slots {
        let n_slot = slot.contour.len();
        if n_slot >= 3 {
            for i in 0..n_slot {
                let p1 = slot.contour[i];
                let p2 = slot.contour[(i + 1) % n_slot];
                append_dxf_line(&mut out, &options.punch_layer, p1.0, p1.1, p2.0, p2.1);
            }
        }
    }

    // C. Hinge / Bend Lines
    for bend in bends {
        append_dxf_line(
            &mut out,
            &options.bend_layer,
            bend.hinge_p1.0,
            bend.hinge_p1.1,
            bend.hinge_p2.0,
            bend.hinge_p2.1,
        );
    }

    out.push_str("0\nENDSEC\n0\nEOF\n");
    out
}

#[inline]
fn append_dxf_line(out: &mut String, layer: &str, x1: f64, y1: f64, x2: f64, y2: f64) {
    out.push_str("0\nLINE\n8\n");
    out.push_str(layer);
    out.push_str("\n10\n");
    out.push_str(&format!("{:.4}\n20\n{:.4}\n30\n0.0\n", x1, y1));
    out.push_str("11\n");
    out.push_str(&format!("{:.4}\n21\n{:.4}\n31\n0.0\n", x2, y2));
}

/// Exports layer-segregated 2D flat DXF file to the given filesystem path.
pub fn export_2d_flat_dxf(
    outer_loop: &StructuralWireLoop,
    slots: &[CmsSlotMetadata],
    bends: &[CmsFoldParams],
    path: &Path,
) -> Result<(), ExportError> {
    let options = DxfExportOptions::default();
    let content = generate_2d_flat_dxf_string(outer_loop, slots, bends, &options);

    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    writer.write_all(content.as_bytes())?;
    writer.flush()?;
    Ok(())
}

// =============================================================================
// 2. 3D STEP EXPORTER (ISO 10303-21 FACETED B-REP)
// =============================================================================

/// Generates an ISO 10303-21 STEP Part 21 string from relaxed fold 3D mesh triangles.
pub fn generate_3d_step_string(result: &RelaxedFoldResult, part_name: &str) -> String {
    let triangles = &result.fused_solid_triangles;
    let mut out = String::with_capacity(32768);

    // STEP Header
    out.push_str("ISO-10303-21;\nHEADER;\n");
    out.push_str("FILE_DESCRIPTION(('SolidCraft CMS truFold 3D B-Rep Export'),'2;1');\n");
    out.push_str(&format!("FILE_NAME('{}.step','2026-10-29',('CADense CMS Engine'),('Intelidan'),'SolidCraft 0.25.1','SolidCraft CMS truFold','');\n", part_name));
    out.push_str("FILE_SCHEMA(('CONFIG_CONTROL_DESIGN'));\nENDSEC;\n");

    // DATA Section
    out.push_str("DATA;\n");
    out.push_str("#1=APPLICATION_CONTEXT('configuration controlled 3D design of mechanical parts');\n");
    out.push_str("#2=APPLICATION_PROTOCOL_DEFINITION('international standard','config_control_design',2026,#1);\n");
    out.push_str("#3=PRODUCT_CONTEXT('part definition',#1,'mechanical');\n");
    out.push_str(&format!("#4=PRODUCT('{0}','{0}','',(#3));\n", part_name));
    out.push_str("#5=PRODUCT_DEFINITION_FORMATION('1.0',$,#4);\n");
    out.push_str("#6=PRODUCT_DEFINITION('design',$,#5,#3);\n");
    out.push_str("#7=PRODUCT_DEFINITION_SHAPE('','',#6);\n");

    // Geometric Context
    out.push_str("#10=UNCERTAINTY_MEASURE_WITH_UNIT(LENGTH_MEASURE(0.01),#14,'distance_accuracy_value','confusion accuracy');\n");
    out.push_str("#11=(GEOMETRIC_REPRESENTATION_CONTEXT(3) GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((#10)) GLOBAL_UNIT_ASSIGNED_CONTEXT((#14,#15,#16)) REPRESENTATION_CONTEXT('','3D'));\n");
    out.push_str("#12=DIMENSIONAL_EXPONENTS(1.,0.,0.,0.,0.,0.,0.);\n");
    out.push_str("#13=SI_UNIT(*,.MILLI.,.METRE.);\n");
    out.push_str("#14=(LENGTH_UNIT() NAMED_UNIT(*) SI_UNIT(.MILLI.,.METRE.));\n");
    out.push_str("#15=(NAMED_UNIT(*) PLANE_ANGLE_UNIT() SI_UNIT($,.RADIAN.));\n");
    out.push_str("#16=(NAMED_UNIT(*) SOLID_ANGLE_UNIT() SI_UNIT($,.STERADIAN.));\n");

    let mut id_seq: usize = 20;

    // Output all Cartesian points and Poly loops for each triangle
    let mut face_ids = Vec::with_capacity(triangles.len());

    for tri in triangles {
        let p0_id = id_seq;
        let p1_id = id_seq + 1;
        let p2_id = id_seq + 2;
        let loop_id = id_seq + 3;
        let bound_id = id_seq + 4;
        let face_id = id_seq + 5;
        id_seq += 6;

        out.push_str(&format!("#{} = CARTESIAN_POINT('',({:.4},{:.4},{:.4}));\n", p0_id, tri[0][0], tri[0][1], tri[0][2]));
        out.push_str(&format!("#{} = CARTESIAN_POINT('',({:.4},{:.4},{:.4}));\n", p1_id, tri[1][0], tri[1][1], tri[1][2]));
        out.push_str(&format!("#{} = CARTESIAN_POINT('',({:.4},{:.4},{:.4}));\n", p2_id, tri[2][0], tri[2][1], tri[2][2]));

        out.push_str(&format!("#{} = POLY_LOOP('',(#{p0_id},#{p1_id},#{p2_id}));\n", loop_id));
        out.push_str(&format!("#{} = FACE_OUTER_BOUND('',#{},.T.);\n", bound_id, loop_id));
        out.push_str(&format!("#{} = FACE_SURFACE('',(#{}),#{},.T.);\n", face_id, bound_id, loop_id));

        face_ids.push(face_id);
    }

    // Closed Shell & Faceted B-Rep Solid
    let shell_id = id_seq;
    let brep_id = id_seq + 1;
    let shape_rep_id = id_seq + 2;

    let face_list = face_ids.iter().map(|id| format!("#{}", id)).collect::<Vec<_>>().join(",");
    out.push_str(&format!("#{} = CLOSED_SHELL('',({}));\n", shell_id, face_list));
    out.push_str(&format!("#{} = FACETED_BREP('{}',#{});\n", brep_id, part_name, shell_id));
    out.push_str(&format!("#{} = SHAPE_REPRESENTATION('{}',(#{},#11),#11);\n", shape_rep_id, part_name, brep_id));
    out.push_str(&format!("#{} = SHAPE_DEFINITION_REPRESENTATION(#7,#{});\n", shape_rep_id + 1, shape_rep_id));

    out.push_str("ENDSEC;\nEND-ISO-10303-21;\n");
    out
}

/// Exports relaxed fold 3D solid as ISO 10303-21 STEP file.
pub fn export_3d_step(
    result: &RelaxedFoldResult,
    part_name: &str,
    path: &Path,
) -> Result<(), ExportError> {
    let content = generate_3d_step_string(result, part_name);

    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    writer.write_all(content.as_bytes())?;
    writer.flush()?;
    Ok(())
}
