//! High-precision 2D boundary edge-tracing and loop assembly engine.
//!
//! Converts an unordered soup of raw lines and arcs from standard laser-cut DXFs into
//! topologically structured, watertight closed B-Rep loops. Uses dead-reckoning node continuity
//! with micro-gap tolerance snapping and Shoelace signed area calculations to separate the
//! outer manufacturing footprint from interior coupler slots and punch holes.

use std::collections::HashSet;

/// Raw un-ordered 2D line segment extracted from a DXF flat-pattern layout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RawDxfSegment {
    pub start: (f64, f64),
    pub end: (f64, f64),
}

impl RawDxfSegment {
    pub fn new(start: (f64, f64), end: (f64, f64)) -> Self {
        Self { start, end }
    }

    /// Length of the segment in millimeters.
    #[inline]
    pub fn length(&self) -> f64 {
        let dx = self.end.0 - self.start.0;
        let dy = self.end.1 - self.start.1;
        (dx * dx + dy * dy).sqrt()
    }

    /// Midpoint of the segment.
    #[inline]
    pub fn midpoint(&self) -> (f64, f64) {
        ((self.start.0 + self.end.0) * 0.5, (self.start.1 + self.end.1) * 0.5)
    }

    /// Normalized direction vector $(dx/L, dy/L)$.
    pub fn direction(&self) -> (f64, f64) {
        let len = self.length();
        if len < 1e-12 {
            (1.0, 0.0)
        } else {
            ((self.end.0 - self.start.0) / len, (self.end.1 - self.start.1) / len)
        }
    }
}

/// Discovered un-notched open edge spanning across a part boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct DiscoveredOpenEdge {
    pub edge_index: usize,
    pub p1: (f64, f64),
    pub p2: (f64, f64),
    pub span_mm: f64,
    pub center: (f64, f64),
    pub outward_normal: (f64, f64),
}

/// A structured, ordered sequence of continuous 2D sketch segments forming a closed boundary loop.
#[derive(Debug, Clone, PartialEq)]
pub struct StructuralWireLoop {
    /// Indices of raw input segments in dead-reckoning traversal order.
    pub edge_ids: Vec<usize>,
    /// Continuous ordered $(X, Y)$ vertices traversing the loop perimeter.
    pub vertices: Vec<(f64, f64)>,
}

impl StructuralWireLoop {
    pub fn new(edge_ids: Vec<usize>, vertices: Vec<(f64, f64)>) -> Self {
        Self { edge_ids, vertices }
    }

    /// Computes the signed two-dimensional surface area using the Shoelace formula.
    ///
    /// Positively oriented shapes (Clockwise or CCW depending on axis) define the Outer Boundary;
    /// negative orientations or smaller magnitudes isolate internal punch holes.
    pub fn signed_area(&self) -> f64 {
        let n = self.vertices.len();
        if n < 3 {
            return 0.0;
        }
        let mut area = 0.0;
        for i in 0..n {
            let j = (i + 1) % n;
            area += self.vertices[i].0 * self.vertices[j].1;
            area -= self.vertices[j].0 * self.vertices[i].1;
        }
        area * 0.5
    }

    /// Absolute enclosed area in square millimeters.
    #[inline]
    pub fn area(&self) -> f64 {
        self.signed_area().abs()
    }

    /// Geometric centroid $(\bar{X}, \bar{Y})$ of the closed vertex loop.
    pub fn centroid(&self) -> (f64, f64) {
        let n = self.vertices.len();
        if n == 0 {
            return (0.0, 0.0);
        }
        let (sx, sy) = self.vertices.iter().fold((0.0, 0.0), |(ax, ay), pt| (ax + pt.0, ay + pt.1));
        let nf = n as f64;
        (sx / nf, sy / nf)
    }

    /// Axis-aligned bounding box: $(min_x, min_y, max_x, max_y)$.
    pub fn bounding_box(&self) -> (f64, f64, f64, f64) {
        if self.vertices.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;
        for &(x, y) in &self.vertices {
            if x < min_x { min_x = x; }
            if y < min_y { min_y = y; }
            if x > max_x { max_x = x; }
            if y > max_y { max_y = y; }
        }
        (min_x, min_y, max_x, max_y)
    }

    /// Returns `true` if the 2D point is contained within this closed polygon loop.
    pub fn contains_point(&self, pt: (f64, f64)) -> bool {
        let n = self.vertices.len();
        if n < 3 {
            return false;
        }
        let mut inside = false;
        let mut j = n - 1;
        for i in 0..n {
            let vi = self.vertices[i];
            let vj = self.vertices[j];
            let intersect = ((vi.1 > pt.1) != (vj.1 > pt.1))
                && (pt.0 < (vj.0 - vi.0) * (pt.1 - vi.1) / ((vj.1 - vi.1) + 1e-15) + vi.0);
            if intersect {
                inside = !inside;
            }
            j = i;
        }
        inside
    }

    /// Scans perimeter segments for un-notched flat linear edges spanning
    /// greater than standard side lip boundaries (e.g. $\ge 24\text{ mm}$).
    /// Computes their span, midpoint, and outward-facing normal vectors.
    pub fn find_open_perimeter_ports(&self, min_span_mm: f64) -> Vec<DiscoveredOpenEdge> {
        let n = self.vertices.len();
        if n < 3 {
            return Vec::new();
        }

        let loop_centroid = self.centroid();
        let mut open_edges = Vec::new();

        for i in 0..n {
            let p1 = self.vertices[i];
            let p2 = self.vertices[(i + 1) % n];
            let dx = p2.0 - p1.0;
            let dy = p2.1 - p1.1;
            let span = (dx * dx + dy * dy).sqrt();

            if span >= min_span_mm {
                let mid = ((p1.0 + p2.0) * 0.5, (p1.1 + p2.1) * 0.5);
                // Tangent vector
                let tx = dx / span;
                let ty = dy / span;
                // Perpendicular normals: (+ty, -tx) or (-ty, +tx)
                let n1 = (ty, -tx);
                let n2 = (-ty, tx);

                // Choose the normal pointing AWAY from the loop centroid
                let d1 = (mid.0 + n1.0 - loop_centroid.0).powi(2) + (mid.1 + n1.1 - loop_centroid.1).powi(2);
                let d2 = (mid.0 + n2.0 - loop_centroid.0).powi(2) + (mid.1 + n2.1 - loop_centroid.1).powi(2);
                let outward = if d1 > d2 { n1 } else { n2 };

                open_edges.push(DiscoveredOpenEdge {
                    edge_index: i,
                    p1,
                    p2,
                    span_mm: span,
                    center: mid,
                    outward_normal: outward,
                });
            }
        }

        open_edges
    }
}

/// Boundary Loop Assembly Subroutine. Converts an unordered soup of raw lines
/// from a laser-cut DXF into structured, traceable B-Rep closed loops.
///
/// Employs dead-reckoning edge tracking with tolerance-based node snapping
/// to bridge small gaps (e.g. $0.05\text{ mm}$) between laser-cutter coordinate endpoints.
pub fn trace_dxf_boundaries(raw_segments: &[RawDxfSegment], tolerance: f64) -> Vec<StructuralWireLoop> {
    let mut structural_loops = Vec::new();
    let mut visited_segments = HashSet::new();

    // 1. Loop over every segment to find un-traced closed boundaries
    for (start_idx, base_seg) in raw_segments.iter().enumerate() {
        if visited_segments.contains(&start_idx) {
            continue;
        }

        let mut current_loop_vertices = vec![base_seg.start];
        let mut current_loop_edges = vec![start_idx];
        visited_segments.insert(start_idx);
        let mut current_endpoint = base_seg.end;
        let mut loop_closed = false;

        // 2. Dead Reckoning Edge Tracking: Search for adjacent connecting vertices
        loop {
            let mut found_next_segment = false;
            for (next_idx, candidate_seg) in raw_segments.iter().enumerate() {
                if visited_segments.contains(&next_idx) {
                    continue;
                }

                // Check for forward node continuity within specified tolerance snapping
                if (candidate_seg.start.0 - current_endpoint.0).abs() <= tolerance
                    && (candidate_seg.start.1 - current_endpoint.1).abs() <= tolerance
                {
                    current_loop_vertices.push(candidate_seg.start);
                    current_loop_edges.push(next_idx);
                    current_endpoint = candidate_seg.end;
                    visited_segments.insert(next_idx);
                    found_next_segment = true;
                    break;
                }
                // Check for reverse-oriented segment snaps (flip vector directions if necessary)
                else if (candidate_seg.end.0 - current_endpoint.0).abs() <= tolerance
                    && (candidate_seg.end.1 - current_endpoint.1).abs() <= tolerance
                {
                    current_loop_vertices.push(candidate_seg.end);
                    current_loop_edges.push(next_idx);
                    current_endpoint = candidate_seg.start;
                    visited_segments.insert(next_idx);
                    found_next_segment = true;
                    break;
                }
            }

            // 3. Evaluate closing criteria bounds
            let original_start = base_seg.start;
            if (current_endpoint.0 - original_start.0).abs() <= tolerance
                && (current_endpoint.1 - original_start.1).abs() <= tolerance
            {
                loop_closed = true;
                break;
            }

            if !found_next_segment {
                // Break open strings (ignores trailing lines or un-closed notation segments)
                break;
            }
        }

        if loop_closed && current_loop_vertices.len() >= 3 {
            structural_loops.push(StructuralWireLoop {
                edge_ids: current_loop_edges,
                vertices: current_loop_vertices,
            });
        }
    }

    structural_loops
}

/// Shoelace Area Formula: Calculates the signed two-dimensional surface space.
///
/// Positively oriented shapes (or highest magnitude area) define the Outer Manufacturing Boundary.
/// Smaller interior loops isolate Internal Punch Holes and Coupler Slots.
pub fn isolate_outer_perimeter(
    loops: Vec<StructuralWireLoop>,
) -> (Option<StructuralWireLoop>, Vec<StructuralWireLoop>) {
    let mut outer_perimeter: Option<StructuralWireLoop> = None;
    let mut interior_holes = Vec::new();
    let mut max_area = -f64::MAX;

    for wire_loop in loops {
        let area = wire_loop.signed_area().abs();
        // The absolute size determines which continuous track represents the outer sheet blank footprint
        if area > max_area {
            max_area = area;
            if let Some(old_outer) = outer_perimeter.take() {
                interior_holes.push(old_outer);
            }
            outer_perimeter = Some(wire_loop);
        } else {
            interior_holes.push(wire_loop);
        }
    }

    (outer_perimeter, interior_holes)
}
