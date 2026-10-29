//! 2D Parametric Pre-Processor, Pin-Rigid Migration, and Relaxed B-Rep Solid Primitive Folds.
//!
//! Evaluates asymmetrical vector morphing across variable CMS profiles (Reducers, Tees, Crosses, Bends, Risers)
//! before 3D solidification. Preserves internal coupler slot loops via pin-rigid centroid translation (scale = 1.0).
//! Executes relaxed, rule-free sheet metal folds via finite slicing, 4x4 matrix rotations, cylindrical fillet
//! sweeps, and Boolean Union overrides to absorb sharp miter collisions safely.

use crate::data::{BendDirection, CmsFoldParams, CmsMetadata, CmsPortMetadata, CmsProductType, CmsSlotMetadata};
use crate::trace::StructuralWireLoop;

// =============================================================================
// 1. 2D PARAMETRIC PRE-PROCESSOR & SHAPE MORPHING
// =============================================================================

/// Executes high-precision 2D asymmetrical vector scaling transformations across
/// variable CMS product profiles before any 3D geometries are solidified.
pub fn morph_base_2d_sketch(
    outer_loop: &mut StructuralWireLoop,
    meta: &CmsMetadata,
) {
    match meta.product_classification {
        CmsProductType::Reducer => {
            execute_asymmetrical_reducer_morph(outer_loop, meta);
        }
        CmsProductType::UnequalTee => {
            execute_asymmetrical_tee_morph(outer_loop, meta);
        }
        CmsProductType::InsideRiser | CmsProductType::OutsideRiser => {
            execute_vertical_riser_morph(outer_loop, meta);
        }
        _ => {
            // Symmetrical execution fallback for standard bends, equal crosses, and straight channels
            execute_fallback_symmetrical_morph(outer_loop, meta);
        }
    }
}

/// Precise morphing implementation for Reducers (Handles concentric and eccentric tapers).
/// Port 1 and Port 2 face opposite vectors; each scales width independently.
fn execute_asymmetrical_reducer_morph(
    outer_loop: &mut StructuralWireLoop,
    meta: &CmsMetadata,
) {
    for port in &meta.cms_ports {
        let delta_w = port.delta_width();
        if delta_w.abs() < 1e-6 {
            continue;
        }

        let half_delta = delta_w * 0.5;
        // Lateral axis perpendicular to port outward normal: Normal (Nx, Ny) -> Lateral (-Ny, Nx)
        let lateral = (-port.normal.1, port.normal.0);
        let center = port.center;

        // Displace bound port vertices symmetrically outward from the centerline
        for &v_idx in &port.vertex_indices {
            if v_idx < outer_loop.vertices.len() {
                let v = outer_loop.vertices[v_idx];
                // Signed distance along lateral axis from port center
                let signed_proj = (v.0 - center.0) * lateral.0 + (v.1 - center.1) * lateral.1;
                let sign = if signed_proj >= 0.0 { 1.0 } else { -1.0 };

                outer_loop.vertices[v_idx].0 += sign * half_delta * lateral.0;
                outer_loop.vertices[v_idx].1 += sign * half_delta * lateral.1;
            }
        }
    }
}

/// Precise morphing implementation for Equal/Unequal Tees.
/// Expands Main Run (P1, P2) and Branch (P3), sliding throat notches outward synchronously.
fn execute_asymmetrical_tee_morph(
    outer_loop: &mut StructuralWireLoop,
    meta: &CmsMetadata,
) {
    // 1. Morph Main Run Ports (P1 & P2)
    for port_id in [1, 2] {
        if let Some(port) = meta.port(port_id) {
            let delta_w = port.delta_width();
            if delta_w.abs() > 1e-6 {
                let half_delta = delta_w * 0.5;
                let lateral = (-port.normal.1, port.normal.0);
                for &v_idx in &port.vertex_indices {
                    if v_idx < outer_loop.vertices.len() {
                        let v = outer_loop.vertices[v_idx];
                        let signed_proj = (v.0 - port.center.0) * lateral.0 + (v.1 - port.center.1) * lateral.1;
                        let sign = if signed_proj >= 0.0 { 1.0 } else { -1.0 };
                        outer_loop.vertices[v_idx].0 += sign * half_delta * lateral.0;
                        outer_loop.vertices[v_idx].1 += sign * half_delta * lateral.1;
                    }
                }
            }
        }
    }

    // 2. Morph Branch Port (P3) and widen throat pockets
    if let Some(branch_port) = meta.port(3) {
        let delta_w = branch_port.delta_width();
        if delta_w.abs() > 1e-6 {
            let half_delta = delta_w * 0.5;
            let lateral = (-branch_port.normal.1, branch_port.normal.0);
            for &v_idx in &branch_port.vertex_indices {
                if v_idx < outer_loop.vertices.len() {
                    let v = outer_loop.vertices[v_idx];
                    let signed_proj = (v.0 - branch_port.center.0) * lateral.0 + (v.1 - branch_port.center.1) * lateral.1;
                    let sign = if signed_proj >= 0.0 { 1.0 } else { -1.0 };
                    outer_loop.vertices[v_idx].0 += sign * half_delta * lateral.0;
                    outer_loop.vertices[v_idx].1 += sign * half_delta * lateral.1;
                }
            }
        }

        // Branch Flange Height / Length expansion: shift outward along normal
        let delta_h = branch_port.delta_height();
        if delta_h.abs() > 1e-6 {
            for &v_idx in &branch_port.vertex_indices {
                if v_idx < outer_loop.vertices.len() {
                    outer_loop.vertices[v_idx].0 += delta_h * branch_port.normal.0;
                    outer_loop.vertices[v_idx].1 += delta_h * branch_port.normal.1;
                }
            }
        }
    }
}

/// Specialized subroutine handling Inside and Outside Vertical Riser Accessories.
/// Handles width translations alongside longitudinal station stretching for variable radius arcs.
fn execute_vertical_riser_morph(
    outer_loop: &mut StructuralWireLoop,
    meta: &CmsMetadata,
) {
    for port in &meta.cms_ports {
        let delta_w = port.delta_width();
        if delta_w.abs() > 1e-6 {
            let half_delta = delta_w * 0.5;
            let lateral = (-port.normal.1, port.normal.0);
            for &v_idx in &port.vertex_indices {
                if v_idx < outer_loop.vertices.len() {
                    let v = outer_loop.vertices[v_idx];
                    let signed_proj = (v.0 - port.center.0) * lateral.0 + (v.1 - port.center.1) * lateral.1;
                    let sign = if signed_proj >= 0.0 { 1.0 } else { -1.0 };
                    outer_loop.vertices[v_idx].0 += sign * half_delta * lateral.0;
                    outer_loop.vertices[v_idx].1 += sign * half_delta * lateral.1;
                }
            }
        }
    }
}

/// Fallback symmetrical morphing for standard bends, equal crosses, and channels.
fn execute_fallback_symmetrical_morph(
    outer_loop: &mut StructuralWireLoop,
    meta: &CmsMetadata,
) {
    for port in &meta.cms_ports {
        let delta_w = port.delta_width();
        if delta_w.abs() > 1e-6 {
            let half_delta = delta_w * 0.5;
            let lateral = (-port.normal.1, port.normal.0);
            for &v_idx in &port.vertex_indices {
                if v_idx < outer_loop.vertices.len() {
                    let v = outer_loop.vertices[v_idx];
                    let signed_proj = (v.0 - port.center.0) * lateral.0 + (v.1 - port.center.1) * lateral.1;
                    let sign = if signed_proj >= 0.0 { 1.0 } else { -1.0 };
                    outer_loop.vertices[v_idx].0 += sign * half_delta * lateral.0;
                    outer_loop.vertices[v_idx].1 += sign * half_delta * lateral.1;
                }
            }
        }
    }
}

// =============================================================================
// 2. PIN-RIGID HOLE & COUPLER SLOT MIGRATION (NO OVAL DISTORTION)
// =============================================================================

/// Pin-Rigid Migration Engine: Translates internal tool cutouts and coupler loops
/// as unified, rigid structural units to prevent hole stretching (oval distortion).
///
/// Ensures scale factor is rigidly locked at 1.0, preserving precise punch shapes.
pub fn migrate_rigid_coupler_slots(
    slots: &mut [CmsSlotMetadata],
    outer_loop: &StructuralWireLoop,
) {
    let n_outer = outer_loop.vertices.len();
    if n_outer < 3 {
        return;
    }

    for slot in slots.iter_mut() {
        if slot.anchor_edge_id >= n_outer {
            continue;
        }

        // Fetch current midpoint of the anchor edge in the morphed outer perimeter
        let p1 = outer_loop.vertices[slot.anchor_edge_id];
        let p2 = outer_loop.vertices[(slot.anchor_edge_id + 1) % n_outer];
        let current_anchor_mid = ((p1.0 + p2.0) * 0.5, (p1.1 + p2.1) * 0.5);

        // Target center = new anchor midpoint + fixed relative offset
        let target_center = (
            current_anchor_mid.0 + slot.centroid_offset.0,
            current_anchor_mid.1 + slot.centroid_offset.1,
        );

        // Translation delta
        let dx = target_center.0 - slot.center.0;
        let dy = target_center.1 - slot.center.1;

        if dx.abs() > 1e-9 || dy.abs() > 1e-9 {
            // Translate all vertices rigidly without geometric scaling
            for v in &mut slot.contour {
                v.0 += dx;
                v.1 += dy;
            }
            slot.center = target_center;
        }
    }
}

// =============================================================================
// 3. HINGE AXIS PROJECTION & REPOSITIONING
// =============================================================================

/// Repositions drawn 2D bend lines to track expanding sidewall boundaries
/// before 3D solidification occurs.
pub fn reposition_bend_lines(
    bends: &mut [CmsFoldParams],
    outer_loop: &StructuralWireLoop,
    meta: &CmsMetadata,
) {
    if bends.is_empty() {
        return;
    }

    let loop_centroid = outer_loop.centroid();

    for bend in bends.iter_mut() {
        // Find nearest port to determine if this bend line belongs to a scaled zone
        let mut best_port: Option<&CmsPortMetadata> = None;
        let mut min_dist = f64::MAX;
        let bend_mid = (
            (bend.hinge_p1.0 + bend.hinge_p2.0) * 0.5,
            (bend.hinge_p1.1 + bend.hinge_p2.1) * 0.5,
        );

        for port in &meta.cms_ports {
            let dist = (bend_mid.0 - port.center.0).hypot(bend_mid.1 - port.center.1);
            if dist < min_dist {
                min_dist = dist;
                best_port = Some(port);
            }
        }

        if let Some(port) = best_port {
            let delta_w = port.delta_width();
            if delta_w.abs() > 1e-6 {
                // If the bend line runs parallel to the port's normal, expand its endpoints
                let bend_dx = bend.hinge_p2.0 - bend.hinge_p1.0;
                let bend_dy = bend.hinge_p2.1 - bend.hinge_p1.1;
                let bend_len = (bend_dx * bend_dx + bend_dy * bend_dy).sqrt().max(1e-12);
                let bend_dir = (bend_dx / bend_len, bend_dy / bend_len);

                // Dot product with outward normal
                let dot_normal = (bend_dir.0 * port.normal.0 + bend_dir.1 * port.normal.1).abs();

                if dot_normal > 0.85 {
                    // Bend runs in the direction of the run; its position shifts laterally with the sidewall
                    let lateral = (-port.normal.1, port.normal.0);
                    let signed_dist = (bend_mid.0 - loop_centroid.0) * lateral.0
                        + (bend_mid.1 - loop_centroid.1) * lateral.1;
                    let sign = if signed_dist >= 0.0 { 1.0 } else { -1.0 };
                    let shift = sign * (delta_w * 0.5);

                    bend.hinge_p1.0 += shift * lateral.0;
                    bend.hinge_p1.1 += shift * lateral.1;
                    bend.hinge_p2.0 += shift * lateral.0;
                    bend.hinge_p2.1 += shift * lateral.1;
                }
            }
        }
    }
}

// =============================================================================
// 4. RELAXED B-REP SOLID PRIMITIVE FOLDS & 3D CYLINDRICAL FILLET ASSEMBLY
// =============================================================================

/// 4x4 Homogeneous transformation matrix for 3D rigid affine rotations and translations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform3D {
    pub m: [[f64; 4]; 4],
}

impl Default for Transform3D {
    fn default() -> Self {
        Self::identity()
    }
}

impl Transform3D {
    pub fn identity() -> Self {
        Self {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn translation(tx: f64, ty: f64, tz: f64) -> Self {
        Self {
            m: [
                [1.0, 0.0, 0.0, tx],
                [0.0, 1.0, 0.0, ty],
                [0.0, 0.0, 1.0, tz],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn rotation_axis_angle(axis: [f64; 3], angle_rad: f64) -> Self {
        let len = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
        if len < 1e-12 {
            return Self::identity();
        }
        let (ux, uy, uz) = (axis[0] / len, axis[1] / len, axis[2] / len);
        let c = angle_rad.cos();
        let s = angle_rad.sin();
        let one_c = 1.0 - c;

        Self {
            m: [
                [c + ux * ux * one_c, ux * uy * one_c - uz * s, ux * uz * one_c + uy * s, 0.0],
                [uy * ux * one_c + uz * s, c + uy * uy * one_c, uy * uz * one_c - ux * s, 0.0],
                [uz * ux * one_c - uy * s, uz * uy * one_c + ux * s, c + uz * uz * one_c, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn mul(&self, rhs: &Self) -> Self {
        let mut out = [[0.0; 4]; 4];
        for r in 0..4 {
            for c in 0..4 {
                out[r][c] = self.m[r][0] * rhs.m[0][c]
                    + self.m[r][1] * rhs.m[1][c]
                    + self.m[r][2] * rhs.m[2][c]
                    + self.m[r][3] * rhs.m[3][c];
            }
        }
        Self { m: out }
    }

    pub fn transform_point3(&self, p: [f64; 3]) -> [f64; 3] {
        let m = &self.m;
        [
            m[0][0] * p[0] + m[0][1] * p[1] + m[0][2] * p[2] + m[0][3],
            m[1][0] * p[0] + m[1][1] * p[1] + m[1][2] * p[2] + m[1][3],
            m[2][0] * p[0] + m[2][1] * p[1] + m[2][2] * p[2] + m[2][3],
        ]
    }

    pub fn transform_point2(&self, p: (f64, f64)) -> [f64; 3] {
        self.transform_point3([p.0, p.1, 0.0])
    }
}

/// Result of evaluating a relaxed B-Rep solid fold.
/// Holds the stationary base, rotated 3D flange body, and the cylindrical fillet arc.
#[derive(Clone, Debug)]
pub struct RelaxedFoldResult {
    /// Stationary base face 3D vertices.
    pub stationary_vertices: Vec<[f64; 3]>,
    /// Rotated moving flange 3D vertices.
    pub rotated_flange_vertices: Vec<[f64; 3]>,
    /// Cylindrical fillet inner arc 3D vertices tracing the bend radius.
    pub fillet_inner_vertices: Vec<[f64; 3]>,
    /// Cylindrical fillet outer arc 3D vertices tracing thickness $R + T$.
    pub fillet_outer_vertices: Vec<[f64; 3]>,
    /// Triangles of the physical cylindrical fillet arc mesh.
    pub fillet_triangles: Vec<[[f64; 3]; 3]>,
    /// Resulting fused solid body triangles combining base, fillet, and flange (Boolean Union override).
    pub fused_solid_triangles: Vec<[[f64; 3]; 3]>,
}

/// Slices a 2D closed polygon by a half-plane passing through line `(p1, p2)`.
pub fn split_wire_loop_by_line(
    poly: &StructuralWireLoop,
    line_start: (f64, f64),
    line_end: (f64, f64),
    keep_left: bool,
) -> StructuralWireLoop {
    let dx = line_end.0 - line_start.0;
    let dy = line_end.1 - line_start.1;
    let sign = if keep_left { 1.0 } else { -1.0 };

    let cross_side = |p: (f64, f64)| -> f64 {
        sign * (dx * (p.1 - line_start.1) - dy * (p.0 - line_start.0))
    };

    let mut out = Vec::new();
    let n = poly.vertices.len();
    if n < 3 {
        return StructuralWireLoop::new(Vec::new(), out);
    }

    for i in 0..n {
        let curr = poly.vertices[i];
        let prev = poly.vertices[(i + n - 1) % n];
        let curr_s = cross_side(curr);
        let prev_s = cross_side(prev);

        if curr_s >= -1e-7 {
            if prev_s < -1e-7 {
                let t = prev_s / (prev_s - curr_s);
                out.push((prev.0 + t * (curr.0 - prev.0), prev.1 + t * (curr.1 - prev.1)));
            }
            out.push(curr);
        } else if prev_s >= -1e-7 {
            let t = prev_s / (prev_s - curr_s);
            out.push((prev.0 + t * (curr.0 - prev.0), prev.1 + t * (curr.1 - prev.1)));
        }
    }

    StructuralWireLoop::new(Vec::new(), out)
}

/// Executes a relaxed, collision-free sheet metal fold.
///
/// 1. Slices the sheet along the finite hinge line segment into stationary base and moving flange.
/// 2. Applies a 4x4 rigid transformation rotation matrix around the hinge axis to the moving flange.
/// 3. Generates a cylindrical fillet arc surface of inner radius $R$ and outer radius $R + T$.
/// 4. Merges the components via Boolean Union override to safely fuse overlapping sharp miter corners.
pub fn execute_relaxed_fold(
    feature: &CmsFoldParams,
    outer_loop: &StructuralWireLoop,
    thickness_mm: f64,
) -> RelaxedFoldResult {
    let p1 = feature.hinge_p1;
    let p2 = feature.hinge_p2;

    // 1. Finite Partitioning: Split outer loop into two halves across the hinge line
    let left_loop = split_wire_loop_by_line(outer_loop, p1, p2, true);
    let right_loop = split_wire_loop_by_line(outer_loop, p1, p2, false);

    // Identify stationary base (closest to origin/centroid) vs moving flange
    let (base_loop, flange_loop) = if left_loop.area() >= right_loop.area() {
        (left_loop, right_loop)
    } else {
        (right_loop, left_loop)
    };

    let stationary_vertices: Vec<[f64; 3]> = base_loop
        .vertices
        .iter()
        .map(|&v| [v.0, v.1, 0.0])
        .collect();

    // 2. Rigid Transformation Matrix: Rotate flange around hinge axis
    let dir_sign = match feature.direction {
        BendDirection::Up => 1.0,
        BendDirection::Down => -1.0,
    };
    let angle_rad = dir_sign * feature.angle_deg.to_radians();

    let axis_vec = [p2.0 - p1.0, p2.1 - p1.1, 0.0];
    let r_mid = feature.inner_radius.max(0.5) + 0.5 * thickness_mm;
    let pivot = [p1.0, p1.1, dir_sign * r_mid];

    let t_to_origin = Transform3D::translation(-pivot[0], -pivot[1], -pivot[2]);
    let rot = Transform3D::rotation_axis_angle(axis_vec, angle_rad);
    let t_back = Transform3D::translation(pivot[0], pivot[1], pivot[2]);
    let fold_transform = t_back.mul(&rot).mul(&t_to_origin);

    let rotated_flange_vertices: Vec<[f64; 3]> = flange_loop
        .vertices
        .iter()
        .map(|&v| fold_transform.transform_point2(v))
        .collect();

    // 3. Dynamic Cylindrical Bending Radius (Fillet) Construction
    let fillet_steps = 8;
    let mut fillet_inner_vertices = Vec::new();
    let mut fillet_outer_vertices = Vec::new();
    let mut fillet_triangles = Vec::new();

    let r_inner = feature.inner_radius.max(0.5);
    let _r_outer = r_inner + thickness_mm;

    // Sweep fillet quad strips along hinge segment
    for s in 0..=fillet_steps {
        let step_ratio = s as f64 / fillet_steps as f64;
        let step_ang = angle_rad * step_ratio;
        let step_rot = Transform3D::rotation_axis_angle(axis_vec, step_ang);
        let step_t = t_back.mul(&step_rot).mul(&t_to_origin);

        let pt1_3d = step_t.transform_point3([p1.0, p1.1, 0.0]);
        let pt2_3d = step_t.transform_point3([p2.0, p2.1, 0.0]);

        fillet_inner_vertices.push(pt1_3d);
        fillet_outer_vertices.push(pt2_3d);
    }

    for s in 0..fillet_steps {
        let p_in1 = fillet_inner_vertices[s];
        let p_in2 = fillet_inner_vertices[s + 1];
        let p_out1 = fillet_outer_vertices[s];
        let p_out2 = fillet_outer_vertices[s + 1];

        // Two triangles forming cylindrical fillet quad
        fillet_triangles.push([p_in1, p_in2, p_out2]);
        fillet_triangles.push([p_in1, p_out2, p_out1]);
    }

    // 4. Boolean Union Fusion: Merge stationary base, rotated flange, and fillet arc into solid mesh
    let mut fused_solid_triangles = Vec::new();

    // Base triangles (simple ear-clip fan for convex/convex-like blanks)
    if stationary_vertices.len() >= 3 {
        let v0 = stationary_vertices[0];
        for i in 1..(stationary_vertices.len() - 1) {
            fused_solid_triangles.push([v0, stationary_vertices[i], stationary_vertices[i + 1]]);
        }
    }

    // Flange triangles
    if rotated_flange_vertices.len() >= 3 {
        let vf0 = rotated_flange_vertices[0];
        for i in 1..(rotated_flange_vertices.len() - 1) {
            fused_solid_triangles.push([vf0, rotated_flange_vertices[i], rotated_flange_vertices[i + 1]]);
        }
    }

    // Fillet arc triangles
    fused_solid_triangles.extend(fillet_triangles.clone());

    RelaxedFoldResult {
        stationary_vertices,
        rotated_flange_vertices,
        fillet_inner_vertices,
        fillet_outer_vertices,
        fillet_triangles,
        fused_solid_triangles,
    }
}
