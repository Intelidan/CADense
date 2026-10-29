//! `crates/cadense-dsf/src/bending.rs`
//!
//! Step 1 & 2: Relaxed 2D Polygon Slicer, Kinematic Fold Tree, and SolidCraft v0.25.1 (Format 41)
//! Primitive Timeline Emitter for CMS Sheet Metal Products (Bends, Tees, Crosses, Reducers, Risers).

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};
use std::f64::consts::PI;

use crate::schema::{CmsCategory, SolidCraftCommand};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BendDirection {
    Up,
    Down,
}

impl BendDirection {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Up => "UP",
            Self::Down => "DOWN",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SheetMaterialRule {
    pub name: String,
    pub thickness_mm: f64,
    pub inner_radius_mm: f64,
    pub default_inner_radius_mm: f64,
    pub k_factor: f64,
}

impl Default for SheetMaterialRule {
    fn default() -> Self {
        Self::bonn_metal_1_5mm()
    }
}

impl SheetMaterialRule {
    pub fn bonn_metal_1_2mm() -> Self {
        Self { name: "1.2mm".into(), thickness_mm: 1.2, inner_radius_mm: 1.2, default_inner_radius_mm: 1.2, k_factor: 0.42 }
    }
    pub fn bonn_metal_1_5mm() -> Self {
        Self { name: "1.5mm".into(), thickness_mm: 1.5, inner_radius_mm: 1.5, default_inner_radius_mm: 1.5, k_factor: 0.42 }
    }
    pub fn bonn_metal_2_0mm() -> Self {
        Self { name: "2.0mm".into(), thickness_mm: 2.0, inner_radius_mm: 2.0, default_inner_radius_mm: 2.0, k_factor: 0.44 }
    }
    /// Computes flat bend allowance (mm) for `angle_deg` using this rule's thickness, radius, and K-factor.
    pub fn bend_allowance(&self, angle_deg: f64) -> f64 {
        calc_bend_allowance(self.thickness_mm, self.inner_radius_mm, angle_deg, self.k_factor)
    }

    /// Computes bend deduction (mm) for `angle_deg` using this rule's thickness, radius, and K-factor.
    pub fn bend_deduction(&self, angle_deg: f64) -> f64 {
        calc_bend_deduction(self.thickness_mm, self.inner_radius_mm, angle_deg, self.k_factor)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ToolingSpec {
    pub punch_radius_mm: f64,
    pub v_die_width_mm: f64,
    pub v_die_angle_deg: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BackgaugeSpec {
    pub stop_distance_mm: f64,
    pub finger_span_mm: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BendRole {
    BaseToWall,
    WallToFlange,
    WallToBoltingTab,
    RiserFloorJoint,
    Arbitrary,
}

impl Default for BendRole {
    fn default() -> Self {
        Self::BaseToWall
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BendLine {
    pub id: usize,
    pub sequence: usize,
    pub label: String,
    pub start_mm: [f64; 2],
    pub end_mm: [f64; 2],
    pub fixed_side_point: Option<[f64; 2]>,
    pub angle_deg: f64,
    pub inner_radius_mm: f64,
    pub direction: BendDirection,
    pub role: BendRole,
    pub bound_port_id: Option<usize>,
    pub tooling: Option<ToolingSpec>,
    pub backgauge: Option<BackgaugeSpec>,
}

impl BendLine {
    pub fn length_mm(&self) -> f64 {
        let dx = self.end_mm[0] - self.start_mm[0];
        let dy = self.end_mm[1] - self.start_mm[1];
        (dx * dx + dy * dy).sqrt()
    }
}

#[inline]
pub fn calc_bend_allowance(thickness_mm: f64, inner_radius_mm: f64, angle_deg: f64, k_factor: f64) -> f64 {
    let angle_rad = angle_deg.abs() * (PI / 180.0);
    angle_rad * (inner_radius_mm + k_factor * thickness_mm)
}

#[inline]
pub fn calc_bend_deduction(thickness_mm: f64, inner_radius_mm: f64, angle_deg: f64, k_factor: f64) -> f64 {
    let angle_rad = angle_deg.abs() * (PI / 180.0);
    let outside_setback = (inner_radius_mm + thickness_mm) * (0.5 * angle_rad).tan();
    let ba = calc_bend_allowance(thickness_mm, inner_radius_mm, angle_deg, k_factor);
    2.0 * outside_setback - ba
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Loop2D {
    pub vertices: Vec<[f64; 2]>,
}

impl Loop2D {
    pub fn new(vertices: Vec<[f64; 2]>) -> Self {
        Self { vertices }
    }

    pub fn rectangular_blank(width_mm: f64, length_mm: f64) -> Self {
        let hw = 0.5 * width_mm;
        let hl = 0.5 * length_mm;
        Self { vertices: vec![[-hw, -hl], [hw, -hl], [hw, hl], [-hw, hl]] }
    }

    pub fn centroid(&self) -> [f64; 2] {
        if self.vertices.is_empty() {
            return [0.0, 0.0];
        }
        let n = self.vertices.len() as f64;
        let (sx, sy) = self.vertices.iter().fold((0.0, 0.0), |(ax, ay), pt| (ax + pt[0], ay + pt[1]));
        [sx / n, sy / n]
    }

    pub fn contains_point(&self, pt: [f64; 2]) -> bool {
        let n = self.vertices.len();
        if n < 3 {
            return false;
        }
        let mut inside = false;
        let mut j = n - 1;
        for i in 0..n {
            let vi = self.vertices[i];
            let vj = self.vertices[j];
            let intersect = ((vi[1] > pt[1]) != (vj[1] > pt[1])) && (pt[0] < (vj[0] - vi[0]) * (pt[1] - vi[1]) / ((vj[1] - vi[1]) + 1e-15) + vi[0]);
            if intersect {
                inside = !inside;
            }
            j = i;
        }
        inside
    }

    pub fn signed_area(&self) -> f64 {
        let n = self.vertices.len();
        if n < 3 {
            return 0.0;
        }
        let mut area = 0.0;
        for i in 0..n {
            let p0 = self.vertices[i];
            let p1 = self.vertices[(i + 1) % n];
            area += p0[0] * p1[1] - p1[0] * p0[1];
        }
        0.5 * area
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HoleCutout2D {
    pub hole_id: usize,
    pub contour: Loop2D,
    pub center_2d: [f64; 2],
    pub is_coupler_slot: bool,
    pub slot_pattern: Option<String>,
    pub bound_port_id: Option<usize>,
    pub port_edge_setback_mm: f64,
    pub bend_line_offset_mm: f64,
}

pub fn split_loop_by_half_plane(poly: &Loop2D, line_start: [f64; 2], line_end: [f64; 2], keep_left: bool) -> Loop2D {
    let dx = line_end[0] - line_start[0];
    let dy = line_end[1] - line_start[1];
    let sign = if keep_left { 1.0 } else { -1.0 };

    let cross_side = |p: [f64; 2]| -> f64 { sign * (dx * (p[1] - line_start[1]) - dy * (p[0] - line_start[0])) };

    let mut out = Vec::new();
    let n = poly.vertices.len();
    if n < 3 {
        return Loop2D::new(out);
    }

    for i in 0..n {
        let curr = poly.vertices[i];
        let prev = poly.vertices[(i + n - 1) % n];
        let curr_s = cross_side(curr);
        let prev_s = cross_side(prev);

        if curr_s >= -1e-7 {
            if prev_s < -1e-7 {
                let t = prev_s / (prev_s - curr_s);
                out.push([prev[0] + t * (curr[0] - prev[0]), prev[1] + t * (curr[1] - prev[1])]);
            }
            out.push(curr);
        } else if prev_s >= -1e-7 {
            let t = prev_s / (prev_s - curr_s);
            out.push([prev[0] + t * (curr[0] - prev[0]), prev[1] + t * (curr[1] - prev[1])]);
        }
    }

    Loop2D::new(out)
}

pub fn bend_segment_intersects_loop(poly: &Loop2D, a: [f64; 2], b: [f64; 2]) -> bool {
    let mid = [0.5 * (a[0] + b[0]), 0.5 * (a[1] + b[1])];
    if poly.contains_point(mid) || poly.contains_point(a) || poly.contains_point(b) {
        return true;
    }
    let n = poly.vertices.len();
    for i in 0..n {
        let p1 = poly.vertices[i];
        let p2 = poly.vertices[(i + 1) % n];
        if segments_intersect_2d(a, b, p1, p2) {
            return true;
        }
    }
    false
}

fn segments_intersect_2d(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2]) -> bool {
    let ccw = |p1: [f64; 2], p2: [f64; 2], p3: [f64; 2]| -> f64 { (p2[0] - p1[0]) * (p3[1] - p1[1]) - (p2[1] - p1[1]) * (p3[0] - p1[0]) };
    let d1 = ccw(a, b, c);
    let d2 = ccw(a, b, d);
    let d3 = ccw(c, d, a);
    let d4 = ccw(c, d, b);
    ((d1 > 1e-7 && d2 < -1e-7) || (d1 < -1e-7 && d2 > 1e-7)) && ((d3 > 1e-7 && d4 < -1e-7) || (d3 < -1e-7 && d4 > 1e-7))
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transform3D {
    pub matrix: [[f64; 4]; 4],
}

impl Default for Transform3D {
    fn default() -> Self {
        Self::identity()
    }
}

impl Transform3D {
    pub fn identity() -> Self {
        Self { matrix: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]] }
    }

    pub fn translation(tx: f64, ty: f64, tz: f64) -> Self {
        Self { matrix: [[1.0, 0.0, 0.0, tx], [0.0, 1.0, 0.0, ty], [0.0, 0.0, 1.0, tz], [0.0, 0.0, 0.0, 1.0]] }
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
            matrix: [
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
                out[r][c] = self.matrix[r][0] * rhs.matrix[0][c]
                    + self.matrix[r][1] * rhs.matrix[1][c]
                    + self.matrix[r][2] * rhs.matrix[2][c]
                    + self.matrix[r][3] * rhs.matrix[3][c];
            }
        }
        Self { matrix: out }
    }

    pub fn transform_point3(&self, p: [f64; 3]) -> [f64; 3] {
        let m = &self.matrix;
        [
            m[0][0] * p[0] + m[0][1] * p[1] + m[0][2] * p[2] + m[0][3],
            m[1][0] * p[0] + m[1][1] * p[1] + m[1][2] * p[2] + m[1][3],
            m[2][0] * p[0] + m[2][1] * p[1] + m[2][2] * p[2] + m[2][3],
        ]
    }

    pub fn transform_point2(&self, p: [f64; 2]) -> [f64; 3] {
        self.transform_point3([p[0], p[1], 0.0])
    }

    pub fn transform_vector3(&self, v: [f64; 3]) -> [f64; 3] {
        let m = &self.matrix;
        [
            m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
            m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
            m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
        ]
    }

    pub fn plane_frame(&self) -> ([f64; 3], [f64; 3], [f64; 3], [f64; 3]) {
        let m = &self.matrix;
        let origin = [m[0][3], m[1][3], m[2][3]];
        let x_axis = [m[0][0], m[1][0], m[2][0]];
        let y_axis = [m[0][1], m[1][1], m[2][1]];
        let z_normal = [m[0][2], m[1][2], m[2][2]];
        (origin, x_axis, y_axis, z_normal)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CylindricalBendArc3D {
    pub bend_id: usize,
    pub label: String,
    pub axis_start_3d: [f64; 3],
    pub axis_end_3d: [f64; 3],
    pub axis_dir_3d: [f64; 3],
    pub bend_length_mm: f64,
    pub inner_radius_mm: f64,
    pub outer_radius_mm: f64,
    pub sweep_angle_deg: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FoldFaceNode {
    pub face_id: usize,
    pub name: String,
    pub parent_face_id: Option<usize>,
    pub hinge_bend_id: Option<usize>,
    pub children: Vec<(usize, usize)>,
    pub flat_contour_2d: Loop2D,
    pub holes_2d: Vec<HoleCutout2D>,
    pub world_transform_3d: Transform3D,
    pub folded_vertices_3d: Vec<[f64; 3]>,
}

pub type FoldedFaceNode = FoldFaceNode;
pub type Point2D = [f64; 2];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimelineFeatureBundle {
    pub feature_group_id: String,
    pub display_name: String,
    pub commands: Vec<SolidCraftCommand>,
}

/// Canonical Stage 1 & Stage 2 Sheet Metal Model for CADense CMS Products.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DsfRiggedSheet {
    pub category: CmsCategory,
    pub product_code: String,
    pub product_name: String,
    pub material_rule: SheetMaterialRule,
    pub outer_contour_2d: Loop2D,
    pub holes_2d: Vec<HoleCutout2D>,
    pub bends: Vec<BendLine>,
    pub face_nodes: BTreeMap<usize, FoldFaceNode>,
    pub bend_arcs_3d: Vec<CylindricalBendArc3D>,
    pub timeline_bundles: Vec<TimelineFeatureBundle>,
    pub preview_flattened: bool,
    pub root_seed_2d: [f64; 2],
}

impl Default for DsfRiggedSheet {
    fn default() -> Self {
        Self::new(CmsCategory::CableTray, "CT-STR", "CABLE TRAY STRAIGHT", SheetMaterialRule::bonn_metal_1_5mm())
    }
}

impl DsfRiggedSheet {
    /// Creates a new sheet metal model initialized with a default 600x300mm flat blank
    /// until a specific 2D DXF contour is loaded via `set_dxf_loops`.
    pub fn new(category: CmsCategory, product_code: impl Into<String>, product_name: impl Into<String>, material_rule: SheetMaterialRule) -> Self {
        let default_blank = Loop2D::rectangular_blank(500.0, 300.0);
        Self::from_dxf_loops_with_meta(category, product_code, product_name, default_blank, Vec::new(), material_rule)
    }

    /// Initializes a sheet metal blank directly from 2D DXF outer and inner hole loops.
    pub fn from_dxf_loops(outer_contour_2d: Loop2D, raw_hole_loops: Vec<Loop2D>, material_rule: SheetMaterialRule) -> Self {
        Self::from_dxf_loops_with_meta(
            CmsCategory::CableTray,
            "CT-CUSTOM",
            "CMS Custom DXF Template",
            outer_contour_2d,
            raw_hole_loops,
            material_rule,
        )
    }

    pub fn from_dxf_loops_with_meta(
        category: CmsCategory,
        product_code: impl Into<String>,
        product_name: impl Into<String>,
        outer_contour_2d: Loop2D,
        raw_hole_loops: Vec<Loop2D>,
        material_rule: SheetMaterialRule,
    ) -> Self {
        let holes_2d = raw_hole_loops
            .into_iter()
            .enumerate()
            .map(|(idx, loop_2d)| {
                let center_2d = loop_2d.centroid();
                HoleCutout2D {
                    hole_id: idx + 1,
                    contour: loop_2d,
                    center_2d,
                    is_coupler_slot: false,
                    slot_pattern: None,
                    bound_port_id: None,
                    port_edge_setback_mm: 0.0,
                    bend_line_offset_mm: 0.0,
                }
            })
            .collect();

        let root_seed_2d = outer_contour_2d.centroid();
        let mut sheet = Self {
            category,
            product_code: product_code.into(),
            product_name: product_name.into(),
            material_rule,
            outer_contour_2d,
            holes_2d,
            bends: Vec::new(),
            face_nodes: BTreeMap::new(),
            bend_arcs_3d: Vec::new(),
            timeline_bundles: Vec::new(),
            preview_flattened: false,
            root_seed_2d,
        };
        sheet.rebuild_fold_tree();
        sheet
    }

    /// Replaces the 2D DXF contour and hole loops and immediately rebuilds the 3D fold tree.
    pub fn set_dxf_loops(&mut self, outer_contour_2d: Loop2D, raw_hole_loops: Vec<Loop2D>) {
        self.root_seed_2d = outer_contour_2d.centroid();
        self.outer_contour_2d = outer_contour_2d;
        self.holes_2d = raw_hole_loops
            .into_iter()
            .enumerate()
            .map(|(idx, loop_2d)| {
                let center_2d = loop_2d.centroid();
                HoleCutout2D {
                    hole_id: idx + 1,
                    contour: loop_2d,
                    center_2d,
                    is_coupler_slot: false,
                    slot_pattern: None,
                    bound_port_id: None,
                    port_edge_setback_mm: 0.0,
                    bend_line_offset_mm: 0.0,
                }
            })
            .collect();
        self.rebuild_fold_tree();
    }

    /// Adds a bend line using `(start_mm, end_mm, angle_deg, direction)` and rebuilds the 3D fold tree.
    pub fn add_bend(&mut self, start_mm: [f64; 2], end_mm: [f64; 2], angle_deg: f64, direction: BendDirection) -> usize {
        let next_id = self.bends.len() + 1;
        let bend = BendLine {
            id: next_id,
            sequence: next_id,
            label: format!("Bend_{}", next_id),
            start_mm,
            end_mm,
            angle_deg,
            inner_radius_mm: self.material_rule.inner_radius_mm,
            direction,
            role: BendRole::BaseToWall,
            bound_port_id: None,
            tooling: None,
            backgauge: None,
            fixed_side_point: Some(self.root_seed_2d),
        };
        self.add_bend_line(bend)
    }

    /// Adds a full `BendLine` struct and rebuilds the 3D fold tree.
    pub fn add_bend_line(&mut self, mut bend: BendLine) -> usize {
        if bend.id == 0 {
            bend.id = self.bends.len() + 1;
        }
        if bend.inner_radius_mm <= 0.0 {
            bend.inner_radius_mm = self.material_rule.inner_radius_mm;
        }
        let assigned_id = bend.id;
        self.bends.push(bend);
        self.rebuild_fold_tree();
        assigned_id
    }
    pub fn remove_bend(&mut self, bend_id: &usize) {
        self.bends.retain(|b| b.id != *bend_id);
        self.rebuild_fold_tree();
    }

    pub fn toggle_bend_direction(&mut self, bend_id: &usize) {
        for b in &mut self.bends {
            if b.id == *bend_id {
                b.direction = match b.direction {
                    BendDirection::Up => BendDirection::Down,
                    BendDirection::Down => BendDirection::Up,
                };
            }
        }
        self.rebuild_fold_tree();
    }

    pub fn tag_coupler_slot(
        &mut self,
        hole_id: usize,
        slot_pattern: impl Into<String>,
        bound_port_id: usize,
        port_edge_setback_mm: f64,
        bend_line_offset_mm: f64,
    ) -> bool {
        if let Some(hole) = self.holes_2d.iter_mut().find(|h| h.hole_id == hole_id) {
            hole.is_coupler_slot = true;
            hole.slot_pattern = Some(slot_pattern.into());
            hole.bound_port_id = Some(bound_port_id);
            hole.port_edge_setback_mm = port_edge_setback_mm;
            hole.bend_line_offset_mm = bend_line_offset_mm;
            return true;
        }
        false
    }

    /// Partitions the 2D flat contour across all bend lines, builds the parent-child
    /// Kinematic Fold Tree, evaluates 3D transforms, and builds SolidCraft Format 41 timeline bundles.
    pub fn rebuild_fold_tree(&mut self) {
        self.face_nodes.clear();
        self.bend_arcs_3d.clear();
        self.timeline_bundles.clear();

        if self.outer_contour_2d.vertices.len() < 3 {
            return;
        }

        let mut regions: Vec<Loop2D> = vec![self.outer_contour_2d.clone()];

        for bend in &self.bends {
            let mut next_regions = Vec::new();
            for reg in regions {
                if bend_segment_intersects_loop(&reg, bend.start_mm, bend.end_mm) {
                    let left = split_loop_by_half_plane(&reg, bend.start_mm, bend.end_mm, true);
                    let right = split_loop_by_half_plane(&reg, bend.start_mm, bend.end_mm, false);
                    if left.vertices.len() >= 3 && right.vertices.len() >= 3 {
                        next_regions.push(left);
                        next_regions.push(right);
                        continue;
                    }
                }
                next_regions.push(reg);
            }
            regions = next_regions;
        }

        let mut root_idx = regions.iter().position(|r| r.contains_point(self.root_seed_2d)).unwrap_or_else(|| {
            regions
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.signed_area().abs().partial_cmp(&b.signed_area().abs()).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(i, _)| i)
                .unwrap_or(0)
        });

        if root_idx >= regions.len() {
            root_idx = 0;
        }
        regions.swap(0, root_idx);

        for (idx, loop_2d) in regions.into_iter().enumerate() {
            let mut assigned_holes = Vec::new();
            for hole in &self.holes_2d {
                if loop_2d.contains_point(hole.center_2d) {
                    assigned_holes.push(hole.clone());
                }
            }
            let name = if idx == 0 { "Face_0_Base_Floor".to_string() } else { format!("Face_{}_Flange", idx) };
            self.face_nodes.insert(
                idx,
                FoldFaceNode {
                    face_id: idx,
                    name,
                    parent_face_id: None,
                    hinge_bend_id: None,
                    children: Vec::new(),
                    flat_contour_2d: loop_2d,
                    holes_2d: assigned_holes,
                    world_transform_3d: Transform3D::identity(),
                    folded_vertices_3d: Vec::new(),
                },
            );
        }

        let face_ids: Vec<usize> = self.face_nodes.keys().copied().collect();
        let mut visited = vec![false; face_ids.len()];
        if visited.is_empty() {
            return;
        }
        visited[0] = true;

        // Pass 1 uses strict adjacency only (both faces touch the bend segment), so a return
        // flange is attached to its sidewall instead of jumping straight to the base.
        // Pass 2 (centroid-crossing fallback) only attaches faces that are still orphaned.
        for strict in [true, false] {
            let mut queue: VecDeque<usize> = face_ids.iter().copied().filter(|id| visited.get(*id).copied().unwrap_or(false)).collect();
            while let Some(parent_id) = queue.pop_front() {
                let parent_loop = self.face_nodes[&parent_id].flat_contour_2d.clone();
                for &candidate_id in &face_ids {
                    if visited[candidate_id] {
                        continue;
                    }
                    let cand_loop = self.face_nodes[&candidate_id].flat_contour_2d.clone();
                    let connecting_bend_info = self.find_connecting_bend(&parent_loop, &cand_loop, strict).map(|b| (b.id, b.label.clone()));

                    if let Some((bend_id, bend_label)) = connecting_bend_info {
                        visited[candidate_id] = true;
                        if let Some(parent_node) = self.face_nodes.get_mut(&parent_id) {
                            parent_node.children.push((bend_id, candidate_id));
                        }
                        if let Some(child_node) = self.face_nodes.get_mut(&candidate_id) {
                            child_node.parent_face_id = Some(parent_id);
                            child_node.hinge_bend_id = Some(bend_id);
                            child_node.name = format!("Face_{}_{}", candidate_id, bend_label);
                        }
                        queue.push_back(candidate_id);
                    }
                }
            }
        }

        self.evaluate_forward_kinematics();
        self.build_timeline_bundles();
    }

    fn find_connecting_bend(&self, loop_a: &Loop2D, loop_b: &Loop2D, strict: bool) -> Option<&BendLine> {
        let ca = loop_a.centroid();
        let cb = loop_b.centroid();
        self.bends.iter().find(|b| {
            let mid = [0.5 * (b.start_mm[0] + b.end_mm[0]), 0.5 * (b.start_mm[1] + b.end_mm[1])];
            let d_a = Self::min_distance_to_loop(mid, loop_a);
            let d_b = Self::min_distance_to_loop(mid, loop_b);
            let adjacent = d_a < 2.0 && d_b < 2.0;
            if strict {
                adjacent
            } else {
                adjacent || segments_intersect_2d(ca, cb, b.start_mm, b.end_mm)
            }
        })
    }

    /// Local hinge transform (in the parent face's flat frame) for `bend` folding `child_id`,
    /// scaled by `angle_ratio` (0 = flat, 1 = full bend angle).
    ///
    /// The axis is oriented so the child face lies on its left; a positive right-hand rotation
    /// then lifts the child toward +Z, so `BendDirection::Up` really folds up.
    /// Returns `(local_hinge, pivot_start_local, pivot_end_local, signed_angle_deg)`.
    fn local_hinge(&self, bend: &BendLine, child_id: usize, angle_ratio: f64) -> (Transform3D, [f64; 3], [f64; 3], f64) {
        let child_centroid = self.face_nodes.get(&child_id).map(|n| n.flat_contour_2d.centroid()).unwrap_or(bend.end_mm);
        let (mut p0, mut p1) = (bend.start_mm, bend.end_mm);
        let side_child = (p1[0] - p0[0]) * (child_centroid[1] - p0[1]) - (p1[1] - p0[1]) * (child_centroid[0] - p0[0]);
        if side_child < 0.0 {
            std::mem::swap(&mut p0, &mut p1);
        }
        let axis_vec = [p1[0] - p0[0], p1[1] - p0[1], 0.0];

        let dir_sign = match bend.direction {
            BendDirection::Up => 1.0,
            BendDirection::Down => -1.0,
        };
        let ratio = if self.preview_flattened { 0.0 } else { angle_ratio.clamp(0.0, 1.0) };
        let signed_angle_deg = dir_sign * bend.angle_deg.abs() * ratio;

        let r_mid = bend.inner_radius_mm.max(0.0) + 0.5 * self.material_rule.thickness_mm;
        let pivot_start = [p0[0], p0[1], dir_sign * r_mid];
        let pivot_end = [p1[0], p1[1], dir_sign * r_mid];

        let t_to_origin = Transform3D::translation(-pivot_start[0], -pivot_start[1], -pivot_start[2]);
        let rot = Transform3D::rotation_axis_angle(axis_vec, signed_angle_deg * (PI / 180.0));
        let t_back = Transform3D::translation(pivot_start[0], pivot_start[1], pivot_start[2]);
        (t_back.mul(&rot).mul(&t_to_origin), pivot_start, pivot_end, signed_angle_deg)
    }

    /// World transforms of every face at a partial fold (`fold_ratio` 0 = flat, 1 = fully folded).
    /// Chains parent -> child hinges (Base -> Sidewall -> Return Flange).
    pub fn face_transforms_at_ratio(&self, fold_ratio: f64) -> BTreeMap<usize, Transform3D> {
        let mut out = BTreeMap::new();
        if !self.face_nodes.contains_key(&0) {
            return out;
        }
        out.insert(0usize, Transform3D::identity());
        let mut queue = VecDeque::new();
        queue.push_back(0usize);
        while let Some(parent_id) = queue.pop_front() {
            let parent_t = out.get(&parent_id).copied().unwrap_or_default();
            let children = self.face_nodes.get(&parent_id).map(|n| n.children.clone()).unwrap_or_default();
            for (bend_id, child_id) in children {
                if out.contains_key(&child_id) {
                    continue;
                }
                if let Some(bend) = self.bends.iter().find(|b| b.id == bend_id) {
                    let (hinge, _, _, _) = self.local_hinge(bend, child_id, fold_ratio);
                    out.insert(child_id, parent_t.mul(&hinge));
                    queue.push_back(child_id);
                }
            }
        }
        // Orphan faces (not reachable through any bend) stay flat.
        for id in self.face_nodes.keys() {
            out.entry(*id).or_insert_with(Transform3D::identity);
        }
        out
    }

    /// Face whose flat contour contains `pt`; falls back to the nearest face boundary.
    pub fn face_id_for_point(&self, pt: [f64; 2]) -> Option<usize> {
        if let Some((id, _)) = self.face_nodes.iter().find(|(_, n)| n.flat_contour_2d.contains_point(pt)) {
            return Some(*id);
        }
        self.face_nodes
            .iter()
            .map(|(id, n)| (*id, Self::min_distance_to_loop(pt, &n.flat_contour_2d)))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(id, _)| id)
    }

    /// Folds a flat point. The owning face is chosen from `ref_pt` (e.g. a slot or sub-edge midpoint)
    /// so every vertex of one rigid feature moves together. Returns `(point_3d, top_normal_3d)`.
    pub fn fold_point_with(&self, transforms: &BTreeMap<usize, Transform3D>, pt: [f64; 2], ref_pt: [f64; 2]) -> ([f64; 3], [f64; 3]) {
        let t = self.face_id_for_point(ref_pt).and_then(|id| transforms.get(&id).copied()).unwrap_or_default();
        (t.transform_point2(pt), t.transform_vector3([0.0, 0.0, 1.0]))
    }

    /// World transforms sweeping a bend's fillet from 0 to its current angle (`steps + 1` samples).
    /// Applying them to the flat bend-line endpoints traces the inner bend arc.
    pub fn bend_fillet_transforms(&self, transforms: &BTreeMap<usize, Transform3D>, bend_id: usize, fold_ratio: f64, steps: usize) -> Vec<Transform3D> {
        let Some((child_id, parent_id)) = self
            .face_nodes
            .iter()
            .find(|(_, n)| n.hinge_bend_id == Some(bend_id))
            .map(|(id, n)| (*id, n.parent_face_id.unwrap_or(0)))
        else {
            return Vec::new();
        };
        let Some(bend) = self.bends.iter().find(|b| b.id == bend_id) else {
            return Vec::new();
        };
        let parent_t = transforms.get(&parent_id).copied().unwrap_or_default();
        let steps = steps.max(1);
        (0..=steps)
            .map(|k| {
                let (hinge, _, _, _) = self.local_hinge(bend, child_id, fold_ratio * k as f64 / steps as f64);
                parent_t.mul(&hinge)
            })
            .collect()
    }

    /// World transform of the parent (stationary-side) face of `bend_id`.
    pub fn bend_parent_transform(&self, transforms: &BTreeMap<usize, Transform3D>, bend_id: usize) -> Transform3D {
        self.face_nodes
            .values()
            .find(|n| n.hinge_bend_id == Some(bend_id))
            .and_then(|n| transforms.get(&n.parent_face_id.unwrap_or(0)).copied())
            .unwrap_or_default()
    }

    fn min_distance_to_loop(pt: [f64; 2], poly: &Loop2D) -> f64 {
        let n = poly.vertices.len();
        if n == 0 {
            return f64::MAX;
        }
        let mut best = f64::MAX;
        for i in 0..n {
            let p0 = poly.vertices[i];
            let p1 = poly.vertices[(i + 1) % n];
            let d = point_to_segment_dist_2d(pt, p0, p1);
            if d < best {
                best = d;
            }
        }
        best
    }

    pub fn evaluate_forward_kinematics(&mut self) {
        self.bend_arcs_3d.clear();
        let t = self.material_rule.thickness_mm;
        if !self.face_nodes.contains_key(&0) {
            return;
        }

        let transforms = self.face_transforms_at_ratio(1.0);

        // Bend arc metadata, in BFS order from the base face.
        let mut arcs = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(0usize);
        let mut seen = std::collections::BTreeSet::new();
        seen.insert(0usize);
        while let Some(parent_id) = queue.pop_front() {
            let parent_t = transforms.get(&parent_id).copied().unwrap_or_default();
            let children = self.face_nodes.get(&parent_id).map(|n| n.children.clone()).unwrap_or_default();
            for (bend_id, child_id) in children {
                if !seen.insert(child_id) {
                    continue;
                }
                if let Some(bend) = self.bends.iter().find(|b| b.id == bend_id) {
                    let (_, pivot_start, pivot_end, signed_deg) = self.local_hinge(bend, child_id, 1.0);
                    let axis = [pivot_end[0] - pivot_start[0], pivot_end[1] - pivot_start[1]];
                    let bend_len = (axis[0] * axis[0] + axis[1] * axis[1]).sqrt();
                    arcs.push(CylindricalBendArc3D {
                        bend_id: bend.id,
                        label: bend.label.clone(),
                        axis_start_3d: parent_t.transform_point3(pivot_start),
                        axis_end_3d: parent_t.transform_point3(pivot_end),
                        axis_dir_3d: parent_t.transform_vector3([axis[0] / bend_len.max(1e-9), axis[1] / bend_len.max(1e-9), 0.0]),
                        bend_length_mm: bend_len,
                        inner_radius_mm: bend.inner_radius_mm,
                        outer_radius_mm: bend.inner_radius_mm + t,
                        sweep_angle_deg: signed_deg,
                    });
                }
                queue.push_back(child_id);
            }
        }
        self.bend_arcs_3d = arcs;

        for (id, node) in self.face_nodes.iter_mut() {
            let world = transforms.get(id).copied().unwrap_or_default();
            node.world_transform_3d = world;
            node.folded_vertices_3d = node.flat_contour_2d.vertices.iter().map(|&v| world.transform_point2(v)).collect();
        }
    }

    fn build_timeline_bundles(&mut self) {
        self.timeline_bundles.clear();
        let thickness = self.material_rule.thickness_mm;

        for (&face_id, node) in &self.face_nodes {
            let (origin, x_axis, y_axis, z_normal) = node.world_transform_3d.plane_frame();
            let plane_id = format!("plane_face_{}", face_id);
            let sketch_id = format!("sketch_face_{}", face_id);
            let extrude_id = format!("extrude_face_{}", face_id);

            let commands = vec![
                SolidCraftCommand {
                    command: "construct.plane.from_frame".to_string(),
                    args: serde_json::json!({
                        "id": plane_id,
                        "name": format!("{}_Plane", node.name),
                        "origin": origin,
                        "x_axis": x_axis,
                        "y_axis": y_axis,
                        "normal": z_normal,
                    }),
                },
                SolidCraftCommand {
                    command: "sketch.create_closed_loops".to_string(),
                    args: serde_json::json!({
                        "id": sketch_id,
                        "plane": plane_id,
                        "outer_loop": node.flat_contour_2d.vertices,
                        "hole_loops": node.holes_2d.iter().map(|h| &h.contour.vertices).collect::<Vec<_>>(),
                    }),
                },
                SolidCraftCommand {
                    command: "extrude".to_string(),
                    args: serde_json::json!({
                        "id": extrude_id,
                        "sketch": sketch_id,
                        "distance_mm": thickness,
                        "operation": "new_body",
                    }),
                },
            ];

            self.timeline_bundles.push(TimelineFeatureBundle {
                feature_group_id: format!("cms_face_group_{}", face_id),
                display_name: node.name.clone(),
                commands,
            });
        }

        for arc in &self.bend_arcs_3d {
            if arc.sweep_angle_deg.abs() < 1e-4 {
                continue;
            }
            let axis_id = format!("axis_bend_{}", arc.bend_id);
            let sketch_id = format!("sketch_bend_arc_{}", arc.bend_id);
            let revolve_id = format!("revolve_bend_arc_{}", arc.bend_id);

            let commands = vec![
                SolidCraftCommand {
                    command: "construct.axis.two_points".to_string(),
                    args: serde_json::json!({
                        "id": axis_id,
                        "name": format!("Bend_{}_Axis", arc.bend_id),
                        "start": arc.axis_start_3d,
                        "end": arc.axis_end_3d,
                    }),
                },
                SolidCraftCommand {
                    command: "revolve".to_string(),
                    args: serde_json::json!({
                        "id": revolve_id,
                        "sketch": sketch_id,
                        "axis": axis_id,
                        "angle_deg": arc.sweep_angle_deg,
                        "inner_radius_mm": arc.inner_radius_mm,
                        "outer_radius_mm": arc.outer_radius_mm,
                        "length_mm": arc.bend_length_mm,
                        "operation": "new_body",
                    }),
                },
            ];

            self.timeline_bundles.push(TimelineFeatureBundle {
                feature_group_id: format!("cms_bend_group_{}", arc.bend_id),
                display_name: format!("Bend_{}_Radius_{}", arc.bend_id, arc.label),
                commands,
            });
        }
    }

    pub fn emit_all_solidcraft_commands(&self) -> Vec<SolidCraftCommand> {
        self.timeline_bundles.iter().flat_map(|b| b.commands.clone()).collect()
    }

    pub fn compile_solidcraft_fold_commands(&self) -> Vec<SolidCraftCommand> {
        self.emit_all_solidcraft_commands()
    }
}

fn point_to_segment_dist_2d(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let len_sq = dx * dx + dy * dy;
    if len_sq < 1e-12 {
        return ((p[0] - a[0]).powi(2) + (p[1] - a[1]).powi(2)).sqrt();
    }
    let t = (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / len_sq).clamp(0.0, 1.0);
    let proj = [a[0] + t * dx, a[1] + t * dy];
    ((p[0] - proj[0]).powi(2) + (p[1] - proj[1]).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_two_stage_sidewall_and_edge_flange_fold_tree() {
        let outer = Loop2D::new(vec![[-100.0, 0.0], [100.0, 0.0], [100.0, 300.0], [-100.0, 300.0]]);
        let slot = Loop2D::new(vec![[60.0, 20.0], [68.0, 20.0], [68.0, 40.0], [60.0, 40.0]]);

        let mut sheet = DsfRiggedSheet::from_dxf_loops(outer, vec![slot], SheetMaterialRule::default());
        sheet.tag_coupler_slot(1, "OBROUND 20X8", 1, 20.0, 10.0);

        sheet.add_bend([50.0, 0.0], [50.0, 300.0], 90.0, BendDirection::Up);
        sheet.add_bend([85.0, 0.0], [85.0, 300.0], 90.0, BendDirection::Up);

        assert_eq!(sheet.face_nodes.len(), 3);
        assert_eq!(sheet.bend_arcs_3d.len(), 2);
        assert_eq!(sheet.timeline_bundles.len(), 5);
    }

    fn max_z_of_face_containing(sheet: &DsfRiggedSheet, pt: [f64; 2], ratio: f64) -> f64 {
        let transforms = sheet.face_transforms_at_ratio(ratio);
        let id = sheet.face_id_for_point(pt).expect("face for point");
        let node = &sheet.face_nodes[&id];
        let t = transforms[&id];
        node.flat_contour_2d.vertices.iter().map(|&v| t.transform_point2(v)[2]).fold(f64::MIN, f64::max)
    }

    #[test]
    fn test_up_bend_folds_toward_positive_z_and_down_toward_negative_z() {
        for flip_line in [false, true] {
            let outer = Loop2D::rectangular_blank(200.0, 100.0);
            let mut sheet = DsfRiggedSheet::from_dxf_loops(outer, Vec::new(), SheetMaterialRule::default());
            let (a, b) = if flip_line { ([60.0, 50.0], [60.0, -50.0]) } else { ([60.0, -50.0], [60.0, 50.0]) };
            sheet.add_bend(a, b, 90.0, BendDirection::Up);

            // Base stays flat, flange tip rises to ~ its 40 mm length.
            assert!(max_z_of_face_containing(&sheet, [0.0, 0.0], 1.0).abs() < 1e-6);
            let up_z = max_z_of_face_containing(&sheet, [90.0, 0.0], 1.0);
            assert!(up_z > 35.0, "UP flange must rise, got z = {up_z}");

            let id = sheet.bends[0].id;
            sheet.toggle_bend_direction(&id);
            let transforms = sheet.face_transforms_at_ratio(1.0);
            let fid = sheet.face_id_for_point([90.0, 0.0]).expect("flange");
            let min_z = sheet.face_nodes[&fid].flat_contour_2d.vertices.iter().map(|&v| transforms[&fid].transform_point2(v)[2]).fold(f64::MAX, f64::min);
            assert!(min_z < -35.0, "DOWN flange must drop, got z = {min_z}");
        }
    }

    #[test]
    fn test_chained_return_flange_forms_u_channel_lip() {
        // Base [-100, 50], sidewall [50, 85], return flange [85, 100].
        let outer = Loop2D::new(vec![[-100.0, 0.0], [100.0, 0.0], [100.0, 300.0], [-100.0, 300.0]]);
        let mut sheet = DsfRiggedSheet::from_dxf_loops(outer, Vec::new(), SheetMaterialRule::default());
        sheet.add_bend([50.0, 0.0], [50.0, 300.0], 90.0, BendDirection::Up);
        sheet.add_bend([85.0, 0.0], [85.0, 300.0], 90.0, BendDirection::Up);

        let wall = sheet.face_id_for_point([70.0, 150.0]).expect("wall");
        let lip = sheet.face_id_for_point([95.0, 150.0]).expect("lip");
        assert_eq!(sheet.face_nodes[&lip].parent_face_id, Some(wall), "lip must hang off the sidewall, not the base");

        let transforms = sheet.face_transforms_at_ratio(1.0);
        let lip_n = transforms[&lip].transform_vector3([0.0, 0.0, 1.0]);
        // Two 90 deg UP folds: lip normal points down (-Z), lip sits high above the base.
        assert!(lip_n[2] < -0.99, "lip normal {lip_n:?}");
        let lip_pt = transforms[&lip].transform_point2([95.0, 150.0]);
        assert!(lip_pt[2] > 30.0, "lip height {lip_pt:?}");
        assert!(lip_pt[0] < 60.0, "lip must fold back inward over the base, got x = {}", lip_pt[0]);

        // Half-way fold is between flat and full.
        let half = sheet.face_transforms_at_ratio(0.5)[&lip].transform_point2([95.0, 150.0]);
        assert!(half[2] > 1.0 && half[2] < lip_pt[2] + 30.0);
    }
}
