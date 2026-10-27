use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex3D {
    pub pos: [f64; 3],
}

impl Vertex3D {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { pos: [x, y, z] }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Triangle3D {
    pub a: [f64; 3],
    pub b: [f64; 3],
    pub c: [f64; 3],
    pub normal: [f64; 3],
}

impl Triangle3D {
    pub fn new(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> Self {
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let n = [ab[1] * ac[2] - ab[2] * ac[1], ab[2] * ac[0] - ab[0] * ac[2], ab[0] * ac[1] - ab[1] * ac[0]];
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-9);
        let normal = [n[0] / len, n[1] / len, n[2] / len];
        Self { a, b, c, normal }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Mesh3D {
    pub vertices: Vec<Vertex3D>,
    pub triangles: Vec<Triangle3D>,
    pub edges: Vec<([f64; 3], [f64; 3])>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    VisibleSharp,
    VisibleSilhouette,
    BendLineDashed,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line2D {
    pub start: [f64; 2],
    pub end: [f64; 2],
    pub kind: LineKind,
    pub depth: f64,
}

/// Standard 30° / 35.264° isometric view camera.
#[derive(Debug, Clone, Copy)]
pub struct IsoCamera {
    pub pitch_deg: f64,
    pub yaw_deg: f64,
    pub scale: f64,
    pub center_2d: [f64; 2],
}

impl Default for IsoCamera {
    fn default() -> Self {
        Self {
            pitch_deg: 35.264, // atan(1 / sqrt(2)) standard engineering isometric angle
            yaw_deg: -45.0,
            scale: 1.0,
            center_2d: [297.0, 210.0],
        }
    }
}

impl IsoCamera {
    pub fn project_point(&self, p: [f64; 3]) -> ([f64; 2], f64) {
        let yaw = self.yaw_deg.to_radians();
        let pitch = self.pitch_deg.to_radians();

        // 1. Rotate around Z (Yaw)
        let x1 = p[0] * yaw.cos() - p[1] * yaw.sin();
        let y1 = p[0] * yaw.sin() + p[1] * yaw.cos();
        let z1 = p[2];

        // 2. Rotate around X (Pitch)
        let x2 = x1;
        let y2 = y1 * pitch.cos() - z1 * pitch.sin();
        let z2 = y1 * pitch.sin() + z1 * pitch.cos();

        // Orthographic projection to 2D + depth
        let sx = self.center_2d[0] + x2 * self.scale;
        let sy = self.center_2d[1] - y2 * self.scale;

        ([sx, sy], z2)
    }
}

/// Vector Hidden-Line Removal (HLR) processor.
pub struct HlrEngine {
    pub camera: IsoCamera,
}

impl HlrEngine {
    pub fn new(camera: IsoCamera) -> Self {
        Self { camera }
    }

    /// Automatically scales and centers the 3D model to fit the target 2D viewport bounds.
    pub fn fit_bounds(&mut self, mesh: &Mesh3D, target_rect: [f64; 4]) {
        if mesh.triangles.is_empty() && mesh.edges.is_empty() {
            return;
        }

        let mut min_x = f64::MAX;
        let mut max_x = f64::MIN;
        let mut min_y = f64::MAX;
        let mut max_y = f64::MIN;

        let base_cam = IsoCamera { scale: 1.0, center_2d: [0.0, 0.0], ..self.camera };

        for tri in &mesh.triangles {
            for pt in &[tri.a, tri.b, tri.c] {
                let (p2d, _) = base_cam.project_point(*pt);
                min_x = min_x.min(p2d[0]);
                max_x = max_x.max(p2d[0]);
                min_y = min_y.min(p2d[1]);
                max_y = max_y.max(p2d[1]);
            }
        }
        for (p1, p2) in &mesh.edges {
            for pt in &[p1, p2] {
                let (p2d, _) = base_cam.project_point(**pt);
                min_x = min_x.min(p2d[0]);
                max_x = max_x.max(p2d[0]);
                min_y = min_y.min(p2d[1]);
                max_y = max_y.max(p2d[1]);
            }
        }

        let w = (max_x - min_x).max(1e-3);
        let h = (max_y - min_y).max(1e-3);

        let target_w = target_rect[2] - target_rect[0];
        let target_h = target_rect[3] - target_rect[1];

        let scale = (target_w / w).min(target_h / h) * 0.85;
        self.camera.scale = scale;

        let mid_x = (min_x + max_x) * 0.5 * scale;
        let mid_y = (min_y + max_y) * 0.5 * scale;

        self.camera.center_2d = [(target_rect[0] + target_rect[2]) * 0.5 - mid_x, (target_rect[1] + target_rect[3]) * 0.5 - mid_y];
    }

    /// Evaluates the mesh geometry and returns clean, classified 2D vector lines.
    pub fn render_vector_line_art(&self, mesh: &Mesh3D) -> Vec<Line2D> {
        let mut visible_lines = Vec::new();
        let samples = 8;

        for &(p1, p2) in &mesh.edges {
            let (start_2d, z1) = self.camera.project_point(p1);
            let (end_2d, z2) = self.camera.project_point(p2);
            let avg_z = (z1 + z2) * 0.5;

            // Edge ray sample testing against front-facing triangles
            let mut occluded_points = 0;
            for i in 1..samples {
                let t = i as f64 / samples as f64;
                let test_pt = [p1[0] + (p2[0] - p1[0]) * t, p1[1] + (p2[1] - p1[1]) * t, p1[2] + (p2[2] - p1[2]) * t];
                let (pt_2d, pt_z) = self.camera.project_point(test_pt);

                for tri in &mesh.triangles {
                    // Backface culling
                    let (_tn_2d, tz) = self.camera.project_point(tri.normal);
                    if tz < 0.0 {
                        continue;
                    }

                    let (t_a, za) = self.camera.project_point(tri.a);
                    let (t_b, zb) = self.camera.project_point(tri.b);
                    let (t_c, zc) = self.camera.project_point(tri.c);
                    let tri_z = (za + zb + zc) / 3.0;

                    if pt_z < tri_z - 1.0 && point_in_triangle_2d(pt_2d, t_a, t_b, t_c) {
                        occluded_points += 1;
                        break;
                    }
                }
            }

            let visibility = 1.0 - (occluded_points as f64 / (samples - 1) as f64);
            if visibility >= 0.5 {
                visible_lines.push(Line2D { start: start_2d, end: end_2d, kind: LineKind::VisibleSharp, depth: avg_z });
            }
        }

        // Sort lines back-to-front for clean PDF vector pathing
        visible_lines.sort_by(|a, b| a.depth.partial_cmp(&b.depth).unwrap_or(Ordering::Equal));
        visible_lines
    }
}

fn point_in_triangle_2d(pt: [f64; 2], v1: [f64; 2], v2: [f64; 2], v3: [f64; 2]) -> bool {
    let d1 = sign(pt, v1, v2);
    let d2 = sign(pt, v2, v3);
    let d3 = sign(pt, v3, v1);
    let has_neg = (d1 < 0.0) || (d2 < 0.0) || (d3 < 0.0);
    let has_pos = (d1 > 0.0) || (d2 > 0.0) || (d3 > 0.0);
    !(has_neg && has_pos)
}

fn sign(p1: [f64; 2], p2: [f64; 2], p3: [f64; 2]) -> f64 {
    (p1[0] - p3[0]) * (p2[1] - p3[1]) - (p2[0] - p3[0]) * (p1[1] - p3[1])
}
