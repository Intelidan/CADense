//! Industrial 3D Solid Sheet Metal Rendering Engine for CADense Studio.
//! Reuses SolidCraft's core 3D shading formulas, metallic material lighting,
//! physical thickness volumetric edge extrusion, punched slot depth,
//! multi-stage chained kinematic folding (Base -> Sidewall -> Edge Return Flange),
//! cylindrical bend fillets, floor blueprint projection, ground grid, and orientation triad.

use cadense_dsf::{BendLine, DsfRiggedSheet};
use cadense_ui::ThemeColors;
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use solidcraft_geom::Vec3;

/// Core metallic lighting equation matching `solidcraft_render::raster::shade`.
/// Computes ambient, key directional light, eye fill light, and specular highlights.
pub fn solidcraft_shade(normal: [f32; 3], view_dir: [f32; 3], base: [f32; 3]) -> Color32 {
    let mut n = Vec3::new(normal[0] as f64, normal[1] as f64, normal[2] as f64);
    let vd = Vec3::new(view_dir[0] as f64, view_dir[1] as f64, view_dir[2] as f64);
    if n.dot(vd) < 0.0 {
        n = -n;
    }
    let key = (vd * 0.8 + Vec3::new(-0.3, 0.2, 0.6)).normalized().unwrap_or(vd);
    let diff = n.dot(key).max(0.0);
    let fill = n.dot(vd).max(0.0);
    let h = (key + vd).normalized().unwrap_or(vd);
    let spec = n.dot(h).max(0.0).powi(40) * 0.25;
    let k = 0.42 + 0.38 * diff + 0.25 * fill;

    let r = ((base[0] as f64 * k + spec).clamp(0.0, 1.0) * 255.0).round() as u8;
    let g = ((base[1] as f64 * k + spec).clamp(0.0, 1.0) * 255.0).round() as u8;
    let b = ((base[2] as f64 * k + spec).clamp(0.0, 1.0) * 255.0).round() as u8;
    Color32::from_rgb(r, g, b)
}

/// Evaluates cumulative multi-stage 3D forward kinematics (`Base -> Sidewall -> Edge Return Flange`)
/// for any 2D flat-pattern point `orig`.
pub fn evaluate_chained_fold_point_and_normal(
    sheet: &DsfRiggedSheet,
    fold_slider_deg: f32,
    orig: [f32; 2],
    eval_ref_override: Option<[f32; 2]>,
) -> ([f32; 3], [f32; 3]) {
    if sheet.bends.is_empty() || fold_slider_deg <= 0.001 {
        return ([orig[0], orig[1], 0.0], [0.0, 0.0, 1.0]);
    }

    let fold_ratio = (fold_slider_deg as f64 / 90.0).clamp(0.0, 1.0);
    let transforms = sheet.face_transforms_at_ratio(fold_ratio);
    let ref_pt = eval_ref_override.unwrap_or(orig);
    let (pt, norm) = sheet.fold_point_with(
        &transforms,
        [orig[0] as f64, orig[1] as f64],
        [ref_pt[0] as f64, ref_pt[1] as f64],
    );
    ([pt[0] as f32, pt[1] as f32, pt[2] as f32], [norm[0] as f32, norm[1] as f32, norm[2] as f32])
}

/// A 3D polygon with precomputed depth and color for painter's algorithm rendering.
#[derive(Clone)]
pub struct ShadedPoly3D {
    pub vertices: Vec<[f32; 3]>,
    pub color: Color32,
    pub outline_stroke: Option<Stroke>,
    pub avg_depth: f32,
}

/// A 3D line segment with depth for rendering.
#[derive(Clone)]
pub struct ShadedLine3D {
    pub p1: [f32; 3],
    pub p2: [f32; 3],
    pub stroke: Stroke,
    pub avg_depth: f32,
    pub dashed: bool,
}

/// A dimension callout for the floor blueprint.
#[derive(Clone)]
pub struct BlueprintDimension {
    pub p1: [f32; 2],
    pub p2: [f32; 2],
    pub text: String,
    pub offset_dist: f32,
}

/// Complete compiled 3D solid model of the folded sheet metal part.
pub struct CompiledSolidSheetMetal {
    pub polygons: Vec<ShadedPoly3D>,
    pub lines: Vec<ShadedLine3D>,
    pub blueprint_floor_polys: Vec<Vec<[f32; 2]>>,
    pub blueprint_floor_lines: Vec<([f32; 2], [f32; 2], bool)>,
    pub dimensions: Vec<BlueprintDimension>,
    pub bend_badges: Vec<([f32; 3], String)>,
}

fn line_intersection_2d(a: [f32; 2], b: [f32; 2], c: [f32; 2], d: [f32; 2]) -> Option<[f32; 2]> {
    let dx1 = b[0] - a[0];
    let dy1 = b[1] - a[1];
    let dx2 = d[0] - c[0];
    let dy2 = d[1] - c[1];
    let denom = dx1 * dy2 - dy1 * dx2;
    if denom.abs() < 1e-6 {
        return None;
    }
    let t = ((c[0] - a[0]) * dy2 - (c[1] - a[1]) * dx2) / denom;
    let u = ((c[0] - a[0]) * dy1 - (c[1] - a[1]) * dx1) / denom;
    if t >= 0.0 && t <= 1.0 && u >= 0.0 && u <= 1.0 {
        Some([a[0] + t * dx1, a[1] + t * dy1])
    } else {
        None
    }
}

fn split_segment_by_bends(p1: [f32; 2], p2: [f32; 2], bends: &[BendLine]) -> Vec<([f32; 2], [f32; 2])> {
    let mut segments = vec![(p1, p2)];
    for bend in bends {
        let mut next_segs = Vec::new();
        let b1 = [bend.start_mm[0] as f32, bend.start_mm[1] as f32];
        let b2 = [bend.end_mm[0] as f32, bend.end_mm[1] as f32];
        for (s1, s2) in segments {
            if let Some(inter) = line_intersection_2d(s1, s2, b1, b2) {
                let d1 = (inter[0] - s1[0]).hypot(inter[1] - s1[1]);
                let d2 = (inter[0] - s2[0]).hypot(inter[1] - s2[1]);
                if d1 > 0.5 && d2 > 0.5 {
                    next_segs.push((s1, inter));
                    next_segs.push((inter, s2));
                    continue;
                }
            }
            next_segs.push((s1, s2));
        }
        segments = next_segs;
    }
    segments
}

impl CompiledSolidSheetMetal {
    /// Compiles the complete 3D solid model from DXF geometry, rigged bends, and fold angle.
    pub fn build(geom: &crate::landing::DxfGeometry2D, sheet: &DsfRiggedSheet, fold_slider_deg: f32, view_dir: [f32; 3]) -> Self {
        let sheet_t = sheet.material_rule.thickness_mm as f32;
        let base_metal = [0.76_f32, 0.73_f32, 0.70_f32];
        let edge_metal = [0.71_f32, 0.68_f32, 0.65_f32];
        let hole_metal = [0.60_f32, 0.57_f32, 0.54_f32];

        let mut polygons = Vec::new();
        let mut lines = Vec::new();
        let mut blueprint_floor_polys = Vec::new();
        let mut blueprint_floor_lines = Vec::new();
        let mut dimensions = Vec::new();
        let mut bend_badges = Vec::new();

        let fold_ratio = (fold_slider_deg as f64 / 90.0).clamp(0.0, 1.0);
        let transforms = sheet.face_transforms_at_ratio(fold_ratio);

        let fold_vertex = |orig: [f32; 2], is_slot: bool, center: Option<[f32; 2]>| -> ([f32; 3], [f32; 3]) {
            let ref_pt = if is_slot { center.unwrap_or(orig) } else { orig };
            let (pt, norm) = sheet.fold_point_with(
                &transforms,
                [orig[0] as f64, orig[1] as f64],
                [ref_pt[0] as f64, ref_pt[1] as f64],
            );
            ([pt[0] as f32, pt[1] as f32, pt[2] as f32], [norm[0] as f32, norm[1] as f32, norm[2] as f32])
        };

        // 1. FLOOR BLUEPRINT EXTRACTION (Unfolded flat pattern at Z = 0)
        let mut raw_outline_segments = Vec::new();
        for (p1, p2, kind) in &geom.lines {
            if *kind == crate::landing::DxfEntityKind::Outline || *kind == crate::landing::DxfEntityKind::ReliefCut {
                raw_outline_segments.push((*p1, *p2));
                blueprint_floor_lines.push((*p1, *p2, false));
            }
        }
        for (arc_c, r, a_start, a_end, kind) in &geom.arcs {
            if *kind == crate::landing::DxfEntityKind::Outline || *kind == crate::landing::DxfEntityKind::ReliefCut {
                let steps = 14;
                let mut sweep = a_end - a_start;
                if sweep <= 0.0 {
                    sweep += 360.0;
                }
                let mut prev = None;
                for s in 0..=steps {
                    let t = s as f32 / steps as f32;
                    let ang = (a_start + t * sweep).to_radians();
                    let pt = [arc_c[0] + r * ang.cos(), arc_c[1] + r * ang.sin()];
                    if let Some(p_prev) = prev {
                        raw_outline_segments.push((p_prev, pt));
                        blueprint_floor_lines.push((p_prev, pt, false));
                    }
                    prev = Some(pt);
                }
            }
        }

        for (p1, p2, kind) in &geom.lines {
            if *kind == crate::landing::DxfEntityKind::SlotOrPerforation {
                blueprint_floor_lines.push((*p1, *p2, false));
            }
        }
        for (arc_c, r, a_start, a_end, kind) in &geom.arcs {
            if *kind == crate::landing::DxfEntityKind::SlotOrPerforation {
                let steps = 10;
                let mut sweep = a_end - a_start;
                if sweep <= 0.0 {
                    sweep += 360.0;
                }
                let mut prev = None;
                for s in 0..=steps {
                    let t = s as f32 / steps as f32;
                    let ang = (a_start + t * sweep).to_radians();
                    let pt = [arc_c[0] + r * ang.cos(), arc_c[1] + r * ang.sin()];
                    if let Some(p_prev) = prev {
                        blueprint_floor_lines.push((p_prev, pt, false));
                    }
                    prev = Some(pt);
                }
            }
        }

        for bend in &sheet.bends {
            let bp1 = [bend.start_mm[0] as f32, bend.start_mm[1] as f32];
            let bp2 = [bend.end_mm[0] as f32, bend.end_mm[1] as f32];
            blueprint_floor_lines.push((bp1, bp2, true));
        }

        let w = (geom.max_x - geom.min_x).abs();
        let h = (geom.max_y - geom.min_y).abs();
        if w > 10.0 && h > 10.0 {
            dimensions.push(BlueprintDimension {
                p1: [geom.min_x, geom.min_y],
                p2: [geom.max_x, geom.min_y],
                text: format!("({:.2} mm)", w),
                offset_dist: -20.0,
            });
            dimensions.push(BlueprintDimension {
                p1: [geom.max_x, geom.min_y],
                p2: [geom.max_x, geom.max_y],
                text: format!("({:.2} mm)", h),
                offset_dist: 20.0,
            });
            if let Some(first_bend) = sheet.bends.first() {
                let b_len = first_bend.length_mm() as f32;
                dimensions.push(BlueprintDimension {
                    p1: [first_bend.start_mm[0] as f32, first_bend.start_mm[1] as f32],
                    p2: [first_bend.end_mm[0] as f32, first_bend.end_mm[1] as f32],
                    text: format!("{:.2} mm", b_len),
                    offset_dist: 16.0,
                });
            }
        }

        // 2. CHAIN OUTLINE INTO CLOSED POLYGON & SUBDIVIDE ACROSS BEND LINES
        let mut ordered_contour: Vec<[f32; 2]> = Vec::new();
        if !raw_outline_segments.is_empty() {
            let mut remaining = raw_outline_segments.clone();
            let mut cur_pt = remaining[0].0;
            ordered_contour.push(cur_pt);

            while !remaining.is_empty() {
                let mut found_idx = None;
                for (i, (p1, p2)) in remaining.iter().enumerate() {
                    let d1 = (cur_pt[0] - p1[0]).hypot(cur_pt[1] - p1[1]);
                    let d2 = (cur_pt[0] - p2[0]).hypot(cur_pt[1] - p2[1]);
                    if d1 < 2.0 {
                        found_idx = Some((i, *p2));
                        break;
                    } else if d2 < 2.0 {
                        found_idx = Some((i, *p1));
                        break;
                    }
                }

                if let Some((idx, next_pt)) = found_idx {
                    remaining.swap_remove(idx);
                    ordered_contour.push(next_pt);
                    cur_pt = next_pt;
                } else {
                    let next_seg = remaining.swap_remove(0);
                    cur_pt = next_seg.1;
                    ordered_contour.push(next_seg.0);
                    ordered_contour.push(cur_pt);
                }
            }
        }

        if ordered_contour.len() > 3 {
            blueprint_floor_polys.push(ordered_contour.clone());

            // Slice the 2D polygon across all active bend lines so triangles never bridge across a fold hinge!
            let mut sub_loops = vec![cadense_dsf::Loop2D::new(ordered_contour.iter().map(|p| [p[0] as f64, p[1] as f64]).collect())];

            for bend in &sheet.bends {
                let mut next_loops = Vec::new();
                for lp in sub_loops {
                    if cadense_dsf::bend_segment_intersects_loop(&lp, bend.start_mm, bend.end_mm) {
                        let left = cadense_dsf::split_loop_by_half_plane(&lp, bend.start_mm, bend.end_mm, true);
                        let right = cadense_dsf::split_loop_by_half_plane(&lp, bend.start_mm, bend.end_mm, false);
                        if left.vertices.len() >= 3 && right.vertices.len() >= 3 {
                            next_loops.push(left);
                            next_loops.push(right);
                            continue;
                        }
                    }
                    next_loops.push(lp);
                }
                sub_loops = next_loops;
            }

            for lp in sub_loops {
                let sub_pts_f32: Vec<[f32; 2]> = lp.vertices.iter().map(|v| [v[0] as f32, v[1] as f32]).collect();
                let geom_pts: Vec<solidcraft_geom::Vec2> = lp.vertices.iter().map(|p| solidcraft_geom::Vec2::new(p[0], p[1])).collect();
                let loop_outer = solidcraft_geom::Loop2::polygon(&geom_pts);
                let region = solidcraft_geom::Region2 { outer: loop_outer, holes: Vec::new() };
                let triangles: Vec<[[f32; 2]; 3]> = {
                    let tris = region.triangulate(1.5);
                    if !tris.is_empty() {
                        tris.into_iter()
                            .map(|tri| [[tri[0].x as f32, tri[0].y as f32], [tri[1].x as f32, tri[1].y as f32], [tri[2].x as f32, tri[2].y as f32]])
                            .collect()
                    } else {
                        ear_clip_polygon(&sub_pts_f32)
                    }
                };

                let centroid_f32 = {
                    let c = lp.centroid();
                    [c[0] as f32, c[1] as f32]
                };

                for tri in triangles {
                    let p0 = tri[0];
                    let p1 = tri[1];
                    let p2 = tri[2];

                    // Evaluate using the sub-face's centroid so the entire rigid face transforms together
                    let (v0, n0) = fold_vertex(p0, true, Some(centroid_f32));
                    let (v1, n1) = fold_vertex(p1, true, Some(centroid_f32));
                    let (v2, n2) = fold_vertex(p2, true, Some(centroid_f32));

                    let avg_n = [(n0[0] + n1[0] + n2[0]) / 3.0, (n0[1] + n1[1] + n2[1]) / 3.0, (n0[2] + n1[2] + n2[2]) / 3.0];

                    let top_color = solidcraft_shade(avg_n, view_dir, base_metal);
                    let depth_top = (v0[1] + v1[1] + v2[1]) / 3.0;
                    polygons.push(ShadedPoly3D { vertices: vec![v0, v1, v2], color: top_color, outline_stroke: None, avg_depth: depth_top });

                    let v0_b = [v0[0] - sheet_t * n0[0], v0[1] - sheet_t * n0[1], v0[2] - sheet_t * n0[2]];
                    let v1_b = [v1[0] - sheet_t * n1[0], v1[1] - sheet_t * n1[1], v1[2] - sheet_t * n1[2]];
                    let v2_b = [v2[0] - sheet_t * n2[0], v2[1] - sheet_t * n2[1], v2[2] - sheet_t * n2[2]];

                    let inv_n = [-avg_n[0], -avg_n[1], -avg_n[2]];
                    let bot_color = solidcraft_shade(inv_n, view_dir, base_metal);
                    let depth_bot = (v0_b[1] + v1_b[1] + v2_b[1]) / 3.0;
                    polygons.push(ShadedPoly3D { vertices: vec![v0_b, v2_b, v1_b], color: bot_color, outline_stroke: None, avg_depth: depth_bot });
                }
            }
        }

        // 3. PHYSICAL THICKNESS SIDE WALLS
        for (p1, p2, kind) in &geom.lines {
            if *kind == crate::landing::DxfEntityKind::Outline || *kind == crate::landing::DxfEntityKind::ReliefCut {
                let sub_segs = split_segment_by_bends(*p1, *p2, &sheet.bends);
                for (s1, s2) in sub_segs {
                    let d = (s2[0] - s1[0]).hypot(s2[1] - s1[1]);
                    if d < 0.2 {
                        continue;
                    }
                    let mid = [(s1[0] + s2[0]) * 0.5, (s1[1] + s2[1]) * 0.5];
                    let (v1, n1) = fold_vertex(s1, true, Some(mid));
                    let (v2, n2) = fold_vertex(s2, true, Some(mid));

                    let v1_b = [v1[0] - sheet_t * n1[0], v1[1] - sheet_t * n1[1], v1[2] - sheet_t * n1[2]];
                    let v2_b = [v2[0] - sheet_t * n2[0], v2[1] - sheet_t * n2[1], v2[2] - sheet_t * n2[2]];

                    let edge_v = [v2[0] - v1[0], v2[1] - v1[1], v2[2] - v1[2]];
                    let wall_n = [edge_v[1] * n1[2] - edge_v[2] * n1[1], edge_v[2] * n1[0] - edge_v[0] * n1[2], edge_v[0] * n1[1] - edge_v[1] * n1[0]];
                    let len = (wall_n[0] * wall_n[0] + wall_n[1] * wall_n[1] + wall_n[2] * wall_n[2]).sqrt().max(1e-4);
                    let unit_wall_n = [wall_n[0] / len, wall_n[1] / len, wall_n[2] / len];

                    let wall_color = solidcraft_shade(unit_wall_n, view_dir, edge_metal);
                    let depth = (v1[1] + v2[1] + v1_b[1] + v2_b[1]) * 0.25;

                    polygons.push(ShadedPoly3D {
                        vertices: vec![v1, v2, v2_b, v1_b],
                        color: wall_color,
                        outline_stroke: Some(Stroke::new(1.0, Color32::from_rgba_unmultiplied(65, 70, 80, 180))),
                        avg_depth: depth,
                    });

                    let line_stroke = Stroke::new(1.3, Color32::from_rgb(50, 55, 65));
                    lines.push(ShadedLine3D { p1: v1, p2: v2, stroke: line_stroke, avg_depth: (v1[1] + v2[1]) * 0.5, dashed: false });
                    lines.push(ShadedLine3D {
                        p1: v1_b,
                        p2: v2_b,
                        stroke: Stroke::new(1.0, Color32::from_rgba_unmultiplied(60, 65, 75, 150)),
                        avg_depth: (v1_b[1] + v2_b[1]) * 0.5,
                        dashed: false,
                    });
                }
            }
        }

        // Arcs on perimeter
        for (arc_c, r, a_start, a_end, kind) in &geom.arcs {
            if *kind == crate::landing::DxfEntityKind::Outline || *kind == crate::landing::DxfEntityKind::ReliefCut {
                let steps = 14;
                let mut sweep = a_end - a_start;
                if sweep <= 0.0 {
                    sweep += 360.0;
                }
                let mut prev: Option<([f32; 3], [f32; 3], [f32; 3])> = None;
                for s in 0..=steps {
                    let t = s as f32 / steps as f32;
                    let ang = (a_start + t * sweep).to_radians();
                    let raw_pt = [arc_c[0] + r * ang.cos(), arc_c[1] + r * ang.sin()];
                    let (v, n) = fold_vertex(raw_pt, false, None);
                    let v_b = [v[0] - sheet_t * n[0], v[1] - sheet_t * n[1], v[2] - sheet_t * n[2]];

                    if let Some((v_prev, vb_prev, n_prev)) = prev {
                        let edge_v = [v[0] - v_prev[0], v[1] - v_prev[1], v[2] - v_prev[2]];
                        let wall_n = [
                            edge_v[1] * n_prev[2] - edge_v[2] * n_prev[1],
                            edge_v[2] * n_prev[0] - edge_v[0] * n_prev[2],
                            edge_v[0] * n_prev[1] - edge_v[1] * n_prev[0],
                        ];
                        let len = (wall_n[0] * wall_n[0] + wall_n[1] * wall_n[1] + wall_n[2] * wall_n[2]).sqrt().max(1e-4);
                        let unit_wall_n = [wall_n[0] / len, wall_n[1] / len, wall_n[2] / len];

                        let wall_color = solidcraft_shade(unit_wall_n, view_dir, edge_metal);
                        let depth = (v[1] + v_prev[1] + v_b[1] + vb_prev[1]) * 0.25;

                        polygons.push(ShadedPoly3D {
                            vertices: vec![v_prev, v, v_b, vb_prev],
                            color: wall_color,
                            outline_stroke: None,
                            avg_depth: depth,
                        });

                        lines.push(ShadedLine3D {
                            p1: v_prev,
                            p2: v,
                            stroke: Stroke::new(1.3, Color32::from_rgb(50, 55, 65)),
                            avg_depth: (v[1] + v_prev[1]) * 0.5,
                            dashed: false,
                        });
                    }
                    prev = Some((v, v_b, n));
                }
            }
        }

        // 4. SOLID PUNCHED SLOTS / HOLES WITH INTERIOR HOLE RIM SHADING
        for (p1, p2, kind) in &geom.lines {
            if *kind == crate::landing::DxfEntityKind::SlotOrPerforation {
                let sc = [(p1[0] + p2[0]) * 0.5, (p1[1] + p2[1]) * 0.5];
                let (v1, n1) = fold_vertex(*p1, true, Some(sc));
                let (v2, n2) = fold_vertex(*p2, true, Some(sc));

                let v1_b = [v1[0] - sheet_t * n1[0], v1[1] - sheet_t * n1[1], v1[2] - sheet_t * n1[2]];
                let v2_b = [v2[0] - sheet_t * n2[0], v2[1] - sheet_t * n2[1], v2[2] - sheet_t * n2[2]];

                let edge_v = [v1[0] - v2[0], v1[1] - v2[1], v1[2] - v2[2]];
                let wall_n = [edge_v[1] * n1[2] - edge_v[2] * n1[1], edge_v[2] * n1[0] - edge_v[0] * n1[2], edge_v[0] * n1[1] - edge_v[1] * n1[0]];
                let len = (wall_n[0] * wall_n[0] + wall_n[1] * wall_n[1] + wall_n[2] * wall_n[2]).sqrt().max(1e-4);
                let unit_wall_n = [wall_n[0] / len, wall_n[1] / len, wall_n[2] / len];

                let hole_color = solidcraft_shade(unit_wall_n, view_dir, hole_metal);
                let depth = (v1[1] + v2[1] + v1_b[1] + v2_b[1]) * 0.25;

                polygons.push(ShadedPoly3D { vertices: vec![v1, v1_b, v2_b, v2], color: hole_color, outline_stroke: None, avg_depth: depth });

                lines.push(ShadedLine3D {
                    p1: v1,
                    p2: v2,
                    stroke: Stroke::new(1.4, Color32::from_rgb(16, 185, 129)),
                    avg_depth: (v1[1] + v2[1]) * 0.5,
                    dashed: false,
                });
            }
        }

        for (arc_c, r, a_start, a_end, kind) in &geom.arcs {
            if *kind == crate::landing::DxfEntityKind::SlotOrPerforation {
                let steps = 12;
                let mut sweep = a_end - a_start;
                if sweep <= 0.0 {
                    sweep += 360.0;
                }
                let mut prev: Option<([f32; 3], [f32; 3], [f32; 3])> = None;
                for s in 0..=steps {
                    let t = s as f32 / steps as f32;
                    let ang = (a_start + t * sweep).to_radians();
                    let raw_pt = [arc_c[0] + r * ang.cos(), arc_c[1] + r * ang.sin()];
                    let (v, n) = fold_vertex(raw_pt, true, Some(*arc_c));
                    let v_b = [v[0] - sheet_t * n[0], v[1] - sheet_t * n[1], v[2] - sheet_t * n[2]];

                    if let Some((v_prev, vb_prev, n_prev)) = prev {
                        let edge_v = [v_prev[0] - v[0], v_prev[1] - v[1], v_prev[2] - v[2]];
                        let wall_n = [
                            edge_v[1] * n_prev[2] - edge_v[2] * n_prev[1],
                            edge_v[2] * n_prev[0] - edge_v[0] * n_prev[2],
                            edge_v[0] * n_prev[1] - edge_v[1] * n_prev[0],
                        ];
                        let len = (wall_n[0] * wall_n[0] + wall_n[1] * wall_n[1] + wall_n[2] * wall_n[2]).sqrt().max(1e-4);
                        let unit_wall_n = [wall_n[0] / len, wall_n[1] / len, wall_n[2] / len];

                        let hole_color = solidcraft_shade(unit_wall_n, view_dir, hole_metal);
                        let depth = (v[1] + v_prev[1] + v_b[1] + vb_prev[1]) * 0.25;

                        polygons.push(ShadedPoly3D {
                            vertices: vec![v_prev, vb_prev, v_b, v],
                            color: hole_color,
                            outline_stroke: None,
                            avg_depth: depth,
                        });

                        lines.push(ShadedLine3D {
                            p1: v_prev,
                            p2: v,
                            stroke: Stroke::new(1.4, Color32::from_rgb(16, 185, 129)),
                            avg_depth: (v[1] + v_prev[1]) * 0.5,
                            dashed: false,
                        });
                    }
                    prev = Some((v, v_b, n));
                }
            }
        }

        // 5. CYLINDRICAL BEND FILLETS & DASHED AMBER TANGENT BEND LINES
        for bend in &sheet.bends {
            let parent_t = sheet.bend_parent_transform(&transforms, bend.id);
            let v1_start_f64 = parent_t.transform_point2(bend.start_mm);
            let v2_start_f64 = parent_t.transform_point2(bend.end_mm);
            let v1_start = [v1_start_f64[0] as f32, v1_start_f64[1] as f32, v1_start_f64[2] as f32];
            let v2_start = [v2_start_f64[0] as f32, v2_start_f64[1] as f32, v2_start_f64[2] as f32];

            let amber_stroke = Stroke::new(1.8, Color32::from_rgb(217, 119, 6));
            lines.push(ShadedLine3D { p1: v1_start, p2: v2_start, stroke: amber_stroke, avg_depth: (v1_start[1] + v2_start[1]) * 0.5, dashed: true });

            if fold_ratio > 0.05 {
                let bend_steps = 6;
                let fillet_ts = sheet.bend_fillet_transforms(&transforms, bend.id, fold_ratio, bend_steps);
                for k in 1..fillet_ts.len() {
                    let t_prev = &fillet_ts[k - 1];
                    let t_curr = &fillet_ts[k];

                    let p1_prev = t_prev.transform_point2(bend.start_mm);
                    let p2_prev = t_prev.transform_point2(bend.end_mm);
                    let p1_curr = t_curr.transform_point2(bend.start_mm);
                    let p2_curr = t_curr.transform_point2(bend.end_mm);

                    let n_prev = t_prev.transform_vector3([0.0, 0.0, 1.0]);
                    let n_curr = t_curr.transform_vector3([0.0, 0.0, 1.0]);
                    let avg_n = [
                        ((n_prev[0] + n_curr[0]) * 0.5) as f32,
                        ((n_prev[1] + n_curr[1]) * 0.5) as f32,
                        ((n_prev[2] + n_curr[2]) * 0.5) as f32,
                    ];

                    let fillet_color = solidcraft_shade(avg_n, view_dir, [0.78, 0.75, 0.72]);
                    let pt1_p = [p1_prev[0] as f32, p1_prev[1] as f32, p1_prev[2] as f32];
                    let pt2_p = [p2_prev[0] as f32, p2_prev[1] as f32, p2_prev[2] as f32];
                    let pt1_c = [p1_curr[0] as f32, p1_curr[1] as f32, p1_curr[2] as f32];
                    let pt2_c = [p2_curr[0] as f32, p2_curr[1] as f32, p2_curr[2] as f32];

                    let depth = (pt1_p[1] + pt2_p[1] + pt1_c[1] + pt2_c[1]) * 0.25;
                    polygons.push(ShadedPoly3D {
                        vertices: vec![pt1_p, pt1_c, pt2_c, pt2_p],
                        color: fillet_color,
                        outline_stroke: None,
                        avg_depth: depth,
                    });
                }
            }

            let mid_pt = [(v1_start[0] + v2_start[0]) * 0.5, (v1_start[1] + v2_start[1]) * 0.5, (v1_start[2] + v2_start[2]) * 0.5 + 4.0];
            let badge_lbl = format!("#{} {} {:.0}° R{:.1}", bend.sequence, bend.direction.label(), bend.angle_deg, bend.inner_radius_mm);
            bend_badges.push((mid_pt, badge_lbl));
        }

        polygons.sort_by(|a, b| b.avg_depth.total_cmp(&a.avg_depth));
        lines.sort_by(|a, b| b.avg_depth.total_cmp(&a.avg_depth));

        Self { polygons, lines, blueprint_floor_polys, blueprint_floor_lines, dimensions, bend_badges }
    }

    /// Renders the compiled solid sheet metal model into the egui painter.
    pub fn paint(
        &self,
        painter: &egui::Painter,
        center: Pos2,
        pan: Vec2,
        zoom: f32,
        orbit_yaw: f32,
        orbit_pitch: f32,
        show_floor_blueprint: bool,
        _colors: &ThemeColors,
    ) {
        let cos_y = orbit_yaw.cos();
        let sin_y = orbit_yaw.sin();
        let cos_p = orbit_pitch.cos();
        let sin_p = orbit_pitch.sin();

        let project_pt = |x: f32, y: f32, z: f32| -> Pos2 {
            let x1 = x * cos_y - y * sin_y;
            let y1 = x * sin_y + y * cos_y;
            let z1 = z;

            let y2 = y1 * cos_p - z1 * sin_p;
            let z2 = y1 * sin_p + z1 * cos_p;

            let d = 950.0_f32;
            let persp = d / (d + y2);
            Pos2::new(center.x + (x1 * persp * zoom) + pan.x, center.y - (z2 * persp * zoom) + pan.y)
        };

        // 1. GROUND PLANE PERSPECTIVE GRID (Z = 0)
        let grid_extent = 250.0_f32;
        let grid_step = 25.0_f32;
        let mut x = -grid_extent;
        while x <= grid_extent {
            let p_start = project_pt(x, -grid_extent, 0.0);
            let p_end = project_pt(x, grid_extent, 0.0);
            let stroke = if x.abs() < 1e-3 {
                Stroke::new(1.2, Color32::from_rgba_unmultiplied(16, 185, 129, 90))
            } else {
                Stroke::new(0.8, Color32::from_rgba_unmultiplied(210, 205, 195, 35))
            };
            painter.line_segment([p_start, p_end], stroke);
            x += grid_step;
        }

        let mut y = -grid_extent;
        while y <= grid_extent {
            let p_start = project_pt(-grid_extent, y, 0.0);
            let p_end = project_pt(grid_extent, y, 0.0);
            let stroke = if y.abs() < 1e-3 {
                Stroke::new(1.2, Color32::from_rgba_unmultiplied(239, 68, 68, 90))
            } else {
                Stroke::new(0.8, Color32::from_rgba_unmultiplied(210, 205, 195, 35))
            };
            painter.line_segment([p_start, p_end], stroke);
            y += grid_step;
        }

        // 2. TRANSLUCENT FLOOR BLUEPRINT PROJECTION (Z = 0)
        if show_floor_blueprint {
            for contour in &self.blueprint_floor_polys {
                let screen_pts: Vec<Pos2> = contour.iter().map(|p| project_pt(p[0], p[1], 0.0)).collect();
                if screen_pts.len() >= 3 {
                    let parchment_fill = Color32::from_rgba_unmultiplied(254, 240, 138, 48);
                    painter.add(egui::Shape::convex_polygon(screen_pts, parchment_fill, Stroke::NONE));
                }
            }

            for (p1, p2, is_bend) in &self.blueprint_floor_lines {
                let s1 = project_pt(p1[0], p1[1], 0.0);
                let s2 = project_pt(p2[0], p2[1], 0.0);
                if *is_bend {
                    painter.line_segment([s1, s2], Stroke::new(1.6, Color32::from_rgba_unmultiplied(217, 119, 6, 200)));
                } else {
                    painter.line_segment([s1, s2], Stroke::new(1.2, Color32::from_rgba_unmultiplied(180, 140, 60, 170)));
                }
            }

            for dim in &self.dimensions {
                let dx = dim.p2[0] - dim.p1[0];
                let dy = dim.p2[1] - dim.p1[1];
                let len = dx.hypot(dy).max(1e-4);
                let u = [dx / len, dy / len];
                let perp = [-u[1] * dim.offset_dist, u[0] * dim.offset_dist];

                let d1 = [dim.p1[0] + perp[0], dim.p1[1] + perp[1]];
                let d2 = [dim.p2[0] + perp[0], dim.p2[1] + perp[1]];

                let s_p1 = project_pt(dim.p1[0], dim.p1[1], 0.0);
                let s_p2 = project_pt(dim.p2[0], dim.p2[1], 0.0);
                let s_d1 = project_pt(d1[0], d1[1], 0.0);
                let s_d2 = project_pt(d2[0], d2[1], 0.0);

                let dim_stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(100, 95, 90, 180));
                painter.line_segment([s_p1, s_d1], dim_stroke);
                painter.line_segment([s_p2, s_d2], dim_stroke);
                painter.line_segment([s_d1, s_d2], dim_stroke);

                let mid_s = Pos2::new((s_d1.x + s_d2.x) * 0.5, (s_d1.y + s_d2.y) * 0.5);
                let txt_rect = Rect::from_center_size(mid_s, Vec2::new(70.0, 14.0));
                painter.rect_filled(txt_rect, 2.0, Color32::from_rgba_unmultiplied(255, 255, 250, 210));
                painter.text(mid_s, egui::Align2::CENTER_CENTER, &dim.text, egui::FontId::monospace(10.5), Color32::from_rgb(60, 55, 50));
            }
        }

        // 3. SOLID SHADED 3D POLYGONS
        for poly in &self.polygons {
            let screen_pts: Vec<Pos2> = poly.vertices.iter().map(|v| project_pt(v[0], v[1], v[2])).collect();
            if screen_pts.len() >= 3 {
                let stroke = poly.outline_stroke.unwrap_or(Stroke::NONE);
                painter.add(egui::Shape::convex_polygon(screen_pts, poly.color, stroke));
            }
        }

        // 4. CRISP FEATURE EDGES & DASHED TANGENT BEND LINES
        for line in &self.lines {
            let s1 = project_pt(line.p1[0], line.p1[1], line.p1[2]);
            let s2 = project_pt(line.p2[0], line.p2[1], line.p2[2]);
            if line.dashed {
                let d = s1.distance(s2);
                let dash_len = 6.0;
                let gap_len = 4.0;
                let steps = (d / (dash_len + gap_len)).floor() as usize;
                for i in 0..steps {
                    let t1 = (i as f32 * (dash_len + gap_len)) / d;
                    let t2 = ((i as f32 * (dash_len + gap_len)) + dash_len) / d;
                    let p_a = s1.lerp(s2, t1);
                    let p_b = s1.lerp(s2, t2.min(1.0));
                    painter.line_segment([p_a, p_b], line.stroke);
                }
            } else {
                painter.line_segment([s1, s2], line.stroke);
            }
        }

        // 5. BEND SEQUENCE BADGES
        for (pos_3d, badge_lbl) in &self.bend_badges {
            let badge_pos = project_pt(pos_3d[0], pos_3d[1], pos_3d[2]);
            let badge_rect = Rect::from_center_size(badge_pos, Vec2::new(92.0, 18.0));
            painter.rect_filled(badge_rect, 4.0, Color32::from_rgba_unmultiplied(245, 158, 11, 235));
            painter.text(badge_pos, egui::Align2::CENTER_CENTER, badge_lbl, egui::FontId::monospace(10.5), Color32::BLACK);
        }

        // 6. 3D ORIENTATION GIZMO / TRIAD (Top-Right Corner)
        let gizmo_center = Pos2::new(center.x + 120.0, center.y - 100.0);
        let g_len = 28.0_f32;
        let g_proj = |x: f32, y: f32, z: f32| -> Pos2 {
            let x1 = x * cos_y - y * sin_y;
            let y1 = x * sin_y + y * cos_y;
            let z1 = z;
            let _y2 = y1 * cos_p - z1 * sin_p;
            let z2 = y1 * sin_p + z1 * cos_p;
            Pos2::new(gizmo_center.x + x1, gizmo_center.y - z2)
        };

        let go = g_proj(0.0, 0.0, 0.0);
        let gx = g_proj(g_len, 0.0, 0.0);
        let gy = g_proj(0.0, g_len, 0.0);
        let gz = g_proj(0.0, 0.0, g_len);

        painter.line_segment([go, gx], Stroke::new(2.5, Color32::from_rgb(239, 68, 68)));
        painter.line_segment([go, gy], Stroke::new(2.5, Color32::from_rgb(16, 185, 129)));
        painter.line_segment([go, gz], Stroke::new(2.5, Color32::from_rgb(59, 130, 246)));

        painter.text(gx, egui::Align2::CENTER_CENTER, "X", egui::FontId::monospace(10.0), Color32::from_rgb(239, 68, 68));
        painter.text(gy, egui::Align2::CENTER_CENTER, "Y", egui::FontId::monospace(10.0), Color32::from_rgb(16, 185, 129));
        painter.text(gz, egui::Align2::CENTER_CENTER, "Z", egui::FontId::monospace(10.0), Color32::from_rgb(59, 130, 246));
    }
}

/// Fallback ear-clipping triangulation for arbitrary 2D polygon loops.
fn ear_clip_polygon(pts: &[[f32; 2]]) -> Vec<[[f32; 2]; 3]> {
    let n = pts.len();
    if n < 3 {
        return Vec::new();
    }
    if n == 3 {
        return vec![[pts[0], pts[1], pts[2]]];
    }
    let mut indices: Vec<usize> = (0..n).collect();
    let mut triangles = Vec::with_capacity(n - 2);

    let is_convex = |a: [f32; 2], b: [f32; 2], c: [f32; 2]| -> bool { (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]) > 0.0 };

    let pt_in_tri = |p: [f32; 2], a: [f32; 2], b: [f32; 2], c: [f32; 2]| -> bool {
        let cp1 = (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
        let cp2 = (c[0] - b[0]) * (p[1] - b[1]) - (c[1] - b[1]) * (p[0] - b[0]);
        let cp3 = (a[0] - c[0]) * (p[1] - c[1]) - (a[1] - c[1]) * (p[0] - c[0]);
        (cp1 >= 0.0 && cp2 >= 0.0 && cp3 >= 0.0) || (cp1 <= 0.0 && cp2 <= 0.0 && cp3 <= 0.0)
    };

    let mut area = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        area += pts[i][0] * pts[j][1] - pts[j][0] * pts[i][1];
    }
    if area < 0.0 {
        indices.reverse();
    }

    let mut count = 0;
    while indices.len() > 2 && count < 2 * n {
        let mut ear_found = false;
        let len = indices.len();
        for i in 0..len {
            let prev = indices[(i + len - 1) % len];
            let curr = indices[i];
            let next = indices[(i + 1) % len];

            let a = pts[prev];
            let b = pts[curr];
            let c = pts[next];

            if is_convex(a, b, c) {
                let mut has_point_inside = false;
                for &idx in &indices {
                    if idx != prev && idx != curr && idx != next {
                        if pt_in_tri(pts[idx], a, b, c) {
                            has_point_inside = true;
                            break;
                        }
                    }
                }
                if !has_point_inside {
                    triangles.push([a, b, c]);
                    indices.remove(i);
                    ear_found = true;
                    break;
                }
            }
        }
        if !ear_found {
            let v0 = indices[0];
            for i in 1..indices.len() - 1 {
                triangles.push([pts[v0], pts[indices[i]], pts[indices[i + 1]]]);
            }
            break;
        }
        count += 1;
    }
    triangles
}
