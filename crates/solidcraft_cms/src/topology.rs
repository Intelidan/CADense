//! Automated Topological Shape Classifier and Port Mapping Engine.
//!
//! Analyzes closed boundary loops from 2D DXF templates, identifies un-notched open edge ports,
//! computes directional normal vectors, and automatically classifies the component into its
//! standard CMS product catalog type (Reducer, Unequal Tee, Cross, Standard Bend, or Riser).
//! Automatically binds dependent perimeter vertex indices and anchors coupler slots without manual mapping.

use crate::data::{
    CmsMetadata, CmsPortMetadata, CmsProductType, CmsSlotMetadata,
};
use crate::trace::{
    isolate_outer_perimeter, trace_dxf_boundaries, DiscoveredOpenEdge, RawDxfSegment,
    StructuralWireLoop,
};
use thiserror::Error;

/// Errors that can occur during topological shape classification.
#[derive(Debug, Error, PartialEq)]
pub enum TopologyError {
    #[error("No closed outer manufacturing boundary found in the DXF geometry")]
    NoClosedOuterBoundary,
    #[error("Insufficient ports detected: found {0}, expected at least 2 open ports")]
    InsufficientPorts(usize),
    #[error("Invalid geometry: {0}")]
    InvalidGeometry(String),
    #[error("Unrecognized CMS product shape: {0}")]
    UnrecognizedCmsShape(String),
}

/// Core topological classification subroutine.
///
/// Takes the extracted outer perimeter loop and interior cutout loops, discovers
/// open connection ports, analyzes vector orientations, and returns populated [`CmsMetadata`].
pub fn cms_identify_topology(
    outer_loop: &StructuralWireLoop,
    interior_holes: &[StructuralWireLoop],
    default_flange_height_mm: f64,
    thickness_mm: f64,
) -> Result<CmsMetadata, TopologyError> {
    // 1. Discover open perimeter edge candidates spanning >= 24mm (standard CMS lip threshold)
    let discovered_edges = outer_loop.find_open_perimeter_ports(24.0);

    if discovered_edges.len() < 2 {
        return Err(TopologyError::InsufficientPorts(discovered_edges.len()));
    }

    // 2. Filter and cluster port edges to isolate true open ports
    // (removes accidental collinear side notches)
    let open_ports = filter_and_cluster_ports(&discovered_edges, outer_loop);

    if open_ports.len() < 2 {
        return Err(TopologyError::InsufficientPorts(open_ports.len()));
    }

    // 3. Classify product type from port quantity and directional vectors
    let (product_type, product_name, sorted_ports) =
        classify_ports_and_vectors(&open_ports, default_flange_height_mm)?;

    // 4. Map dependent vertex indices to each port
    let cms_ports = bind_port_vertex_indices(sorted_ports, outer_loop);

    // 5. Anchor interior holes and coupler slots to nearest outer perimeter edges
    let rigid_slots = anchor_interior_cutouts(interior_holes, outer_loop);

    let mut meta = CmsMetadata::new(product_type, product_name, thickness_mm);
    meta.cms_ports = cms_ports;
    meta.rigid_slots = rigid_slots;

    Ok(meta)
}

/// Unified entry point: parses raw un-ordered DXF segment arrays, traces closed loops,
/// and classifies the product topology in one continuous pipeline.
pub fn cms_parse_and_classify_dxf(
    raw_segments: &[RawDxfSegment],
    tolerance: f64,
    default_flange_height_mm: f64,
    thickness_mm: f64,
) -> Result<CmsMetadata, TopologyError> {
    // 1. Trace closed loops with tolerance-based dead reckoning
    let loops = trace_dxf_boundaries(raw_segments, tolerance);

    // 2. Isolate outer perimeter from interior coupler/bolt punch holes
    let (outer_opt, interior_holes) = isolate_outer_perimeter(loops);
    let outer_loop = outer_opt.ok_or(TopologyError::NoClosedOuterBoundary)?;

    // 3. Run topological classification
    cms_identify_topology(&outer_loop, &interior_holes, default_flange_height_mm, thickness_mm)
}

/// Filters raw discovered edges to isolate distinct open connecting ports.
fn filter_and_cluster_ports(
    edges: &[DiscoveredOpenEdge],
    _outer_loop: &StructuralWireLoop,
) -> Vec<DiscoveredOpenEdge> {
    let mut filtered: Vec<DiscoveredOpenEdge> = Vec::new();

    for edge in edges {
        // If an edge is nearly identical or collinear with an existing one within 15mm, keep the larger span
        let mut duplicate = false;
        for existing in &mut filtered {
            let dist = (edge.center.0 - existing.center.0).hypot(edge.center.1 - existing.center.1);
            let dot = edge.outward_normal.0 * existing.outward_normal.0
                + edge.outward_normal.1 * existing.outward_normal.1;

            if dist < 15.0 && dot > 0.85 {
                if edge.span_mm > existing.span_mm {
                    *existing = edge.clone();
                }
                duplicate = true;
                break;
            }
        }
        if !duplicate {
            filtered.push(edge.clone());
        }
    }

    filtered
}

/// Classifies product type and organizes ports into canonical order (e.g. P1=In, P2=Out, P3=Branch)
/// by evaluating the geometric relationship between discovered candidate open edges.
fn classify_ports_and_vectors(
    candidate_edges: &[DiscoveredOpenEdge],
    default_height: f64,
) -> Result<(CmsProductType, String, Vec<CmsPortMetadata>), TopologyError> {
    let dot_prod = |a: &DiscoveredOpenEdge, b: &DiscoveredOpenEdge| -> f64 {
        a.outward_normal.0 * b.outward_normal.0 + a.outward_normal.1 * b.outward_normal.1
    };

    let n = candidate_edges.len();
    if n < 2 {
        return Err(TopologyError::InsufficientPorts(n));
    }

    // --- 1. Check for 4-Way Cross ---
    // Look for 4 edges forming two opposing pairs that are mutually orthogonal.
    // In a cross fitting, opposing ports must have matching widths within 18%.
    for i in 0..n {
        for j in (i + 1)..n {
            if dot_prod(&candidate_edges[i], &candidate_edges[j]) < -0.95 {
                let w_ij_ratio = candidate_edges[i].span_mm / candidate_edges[j].span_mm;
                if w_ij_ratio < 0.82 || w_ij_ratio > 1.22 {
                    continue;
                }
                for k in 0..n {
                    if k == i || k == j { continue; }
                    for l in (k + 1)..n {
                        if l == i || l == j { continue; }
                        if dot_prod(&candidate_edges[k], &candidate_edges[l]) < -0.95 {
                            let w_kl_ratio = candidate_edges[k].span_mm / candidate_edges[l].span_mm;
                            if w_kl_ratio < 0.82 || w_kl_ratio > 1.22 {
                                continue;
                            }
                            // Check orthogonality between pair 1 and pair 2
                            if dot_prod(&candidate_edges[i], &candidate_edges[k]).abs() < 0.15 {
                                let c1 = &candidate_edges[i];
                                let c2 = &candidate_edges[j];
                                let c3 = &candidate_edges[k];
                                let c4 = &candidate_edges[l];
                                let out_ports = vec![
                                    CmsPortMetadata::new(1, "P1 - Port 1", c1.span_mm, default_height, c1.outward_normal, c1.center),
                                    CmsPortMetadata::new(2, "P2 - Port 2", c2.span_mm, default_height, c2.outward_normal, c2.center),
                                    CmsPortMetadata::new(3, "P3 - Port 3", c3.span_mm, default_height, c3.outward_normal, c3.center),
                                    CmsPortMetadata::new(4, "P4 - Port 4", c4.span_mm, default_height, c4.outward_normal, c4.center),
                                ];
                                let name = format!("CMS-CROSS-4WAY-{:.0}", c1.span_mm);
                                return Ok((CmsProductType::UnequalCross, name, out_ports));
                            }
                        }
                    }
                }
            }
        }
    }

    // --- 2. Check for 3-Port Tee (Equal or Unequal) ---
    // Look for 2 opposing collinear run edges (dot < -0.95) + 1 orthogonal branch edge (|dot| < 0.15)
    let mut best_tee = None;
    for i in 0..n {
        for j in (i + 1)..n {
            if dot_prod(&candidate_edges[i], &candidate_edges[j]) < -0.95 {
                // (i, j) is the main run
                for k in 0..n {
                    if k == i || k == j { continue; }
                    let d1 = dot_prod(&candidate_edges[i], &candidate_edges[k]).abs();
                    let d2 = dot_prod(&candidate_edges[j], &candidate_edges[k]).abs();
                    if d1 < 0.15 && d2 < 0.15 {
                        best_tee = Some((i, j, k));
                        break;
                    }
                }
            }
            if best_tee.is_some() { break; }
        }
        if best_tee.is_some() { break; }
    }

    if let Some((r1, r2, br)) = best_tee {
        let p1 = &candidate_edges[r1];
        let p2 = &candidate_edges[r2];
        let p3 = &candidate_edges[br];
        let is_unequal = (p1.span_mm - p3.span_mm).abs() > 5.0;
        let name = if is_unequal {
            format!("CMS-TEE-UNEQUAL-{:.0}X{:.0}", p1.span_mm, p3.span_mm)
        } else {
            format!("CMS-TEE-EQUAL-{:.0}", p1.span_mm)
        };
        let out_ports = vec![
            CmsPortMetadata::new(1, "P1 - Main In", p1.span_mm, default_height, p1.outward_normal, p1.center),
            CmsPortMetadata::new(2, "P2 - Main Out", p2.span_mm, default_height, p2.outward_normal, p2.center),
            CmsPortMetadata::new(3, "P3 - Branch", p3.span_mm, default_height, p3.outward_normal, p3.center),
        ];
        return Ok((CmsProductType::UnequalTee, name, out_ports));
    }

    // --- 3. Check for 2-Port Reducer (Opposing parallel with unequal widths) ---
    // Two strictly parallel opposing edges with dot_prod < -0.95 and span ratio >= 1.08
    let mut best_reducer = None;
    for i in 0..n {
        for j in (i + 1)..n {
            let dot = dot_prod(&candidate_edges[i], &candidate_edges[j]);
            if dot < -0.95 {
                let w1 = candidate_edges[i].span_mm;
                let w2 = candidate_edges[j].span_mm;
                let ratio = if w1 > w2 { w1 / w2 } else { w2 / w1 };
                if ratio >= 1.08 {
                    best_reducer = Some((i, j));
                    break;
                }
            }
        }
        if best_reducer.is_some() { break; }
    }

    if let Some((r1, r2)) = best_reducer {
        let p_a = &candidate_edges[r1];
        let p_b = &candidate_edges[r2];
        let (p_large, p_small) = if p_a.span_mm >= p_b.span_mm { (p_a, p_b) } else { (p_b, p_a) };
        let out_ports = vec![
            CmsPortMetadata::new(1, "P1 - Large End", p_large.span_mm, default_height, p_large.outward_normal, p_large.center),
            CmsPortMetadata::new(2, "P2 - Small End", p_small.span_mm, default_height, p_small.outward_normal, p_small.center),
        ];
        let name = format!("CMS-REDUCER-{:.0}X{:.0}", p_large.span_mm, p_small.span_mm);
        return Ok((CmsProductType::Reducer, name, out_ports));
    }

    // --- 4. Check for Standard Bend / Elbow / Straight / Riser ---
    // Look for 2 orthogonal edges (|dot| < 0.15) or 45-degree edges (|dot - (-0.707)| < 0.15) or opposing equal-width edges (dot < -0.95)
    let mut best_bend = None;
    for i in 0..n {
        for j in (i + 1)..n {
            let dot = dot_prod(&candidate_edges[i], &candidate_edges[j]);
            let w1 = candidate_edges[i].span_mm;
            let w2 = candidate_edges[j].span_mm;
            let ratio = if w1 > w2 { w1 / w2 } else { w2 / w1 };

            // Orthogonal ports (90° bend) with similar width
            if dot.abs() < 0.15 && ratio < 1.35 {
                best_bend = Some((i, j, "90"));
                break;
            }
            // 45° bend (angle between outward normals is ~135°, dot ~ -0.707)
            if (dot - (-0.707)).abs() < 0.15 && ratio < 1.35 {
                best_bend = Some((i, j, "45"));
                break;
            }
            // Opposing equal-width straight section or riser
            if dot < -0.95 && ratio < 1.08 {
                best_bend = Some((i, j, "STRAIGHT"));
                break;
            }
        }
        if best_bend.is_some() { break; }
    }

    if let Some((b1, b2, kind)) = best_bend {
        let p1 = &candidate_edges[b1];
        let p2 = &candidate_edges[b2];
        let name = if kind == "STRAIGHT" {
            format!("CMS-STRAIGHT-{:.0}", p1.span_mm)
        } else {
            format!("CMS-BEND-{}DEG-{:.0}", kind, p1.span_mm)
        };
        let out_ports = vec![
            CmsPortMetadata::new(1, "P1 - Port 1", p1.span_mm, default_height, p1.outward_normal, p1.center),
            CmsPortMetadata::new(2, "P2 - Port 2", p2.span_mm, default_height, p2.outward_normal, p2.center),
        ];
        return Ok((CmsProductType::StandardBendOrRiser, name, out_ports));
    }

    // --- 5. Fallback: Custom Multi-Port Transition ---
    let mut out_ports = Vec::new();
    for (idx, p) in candidate_edges.iter().take(4).enumerate() {
        out_ports.push(CmsPortMetadata::new(
            idx + 1,
            format!("P{} - Port", idx + 1),
            p.span_mm,
            default_height,
            p.outward_normal,
            p.center,
        ));
    }
    let name = format!("CMS-TRANSITION-{}PORTS", candidate_edges.len());
    Ok((CmsProductType::CustomTransition, name, out_ports))
}

/// Identifies and binds perimeter loop vertex indices that displace when each port scales.
fn bind_port_vertex_indices(
    mut ports: Vec<CmsPortMetadata>,
    outer_loop: &StructuralWireLoop,
) -> Vec<CmsPortMetadata> {
    let n = outer_loop.vertices.len();
    if n < 3 {
        return ports;
    }

    for port in &mut ports {
        let mut bound_indices = Vec::new();
        let port_center = port.center;
        let half_width = port.base_width * 0.55; // 10% threshold envelope

        for i in 0..n {
            let v = outer_loop.vertices[i];
            let dist_to_center = (v.0 - port_center.0).hypot(v.1 - port_center.1);
            if dist_to_center <= half_width + 10.0 {
                bound_indices.push(i);
            }
        }

        port.vertex_indices = bound_indices;
    }

    ports
}

/// Anchors interior punch holes and coupler slot cutouts to their nearest outer perimeter edges.
fn anchor_interior_cutouts(
    holes: &[StructuralWireLoop],
    outer_loop: &StructuralWireLoop,
) -> Vec<CmsSlotMetadata> {
    let mut slots = Vec::new();
    let n_outer = outer_loop.vertices.len();
    if n_outer < 3 {
        return slots;
    }

    for (slot_idx, hole) in holes.iter().enumerate() {
        let hole_center = hole.centroid();
        let (min_x, min_y, max_x, max_y) = hole.bounding_box();
        let w = (max_x - min_x).abs();
        let h = (max_y - min_y).abs();
        let (major, minor) = if w >= h { (w, h) } else { (h, w) };

        // Identify standard coupler slot: aspect ratio >= 1.8 and major dimension 15-35mm
        let is_coupler = (major / minor.max(0.1) >= 1.8) && (major >= 12.0 && major <= 40.0);
        let pattern = if is_coupler {
            Some(format!("OBROUND_{:.0}X{:.0}", major, minor))
        } else if (major - minor).abs() < 2.0 {
            Some(format!("HOLE_DIA_{:.0}", (major + minor) * 0.5))
        } else {
            None
        };

        // Find nearest outer perimeter edge
        let mut best_edge_id = 0;
        let mut best_dist = f64::MAX;
        let mut best_edge_mid = (0.0, 0.0);

        for i in 0..n_outer {
            let p1 = outer_loop.vertices[i];
            let p2 = outer_loop.vertices[(i + 1) % n_outer];
            let mid = ((p1.0 + p2.0) * 0.5, (p1.1 + p2.1) * 0.5);
            let d = point_to_segment_distance(hole_center, p1, p2);

            if d < best_dist {
                best_dist = d;
                best_edge_id = i;
                best_edge_mid = mid;
            }
        }

        // Relative centroid offset from anchor edge midpoint
        let offset = (hole_center.0 - best_edge_mid.0, hole_center.1 - best_edge_mid.1);

        let mut slot = CmsSlotMetadata::new(
            slot_idx + 1,
            hole.vertices.clone(),
            hole_center,
            best_edge_id,
            offset,
        );
        slot.is_coupler_slot = is_coupler;
        slot.slot_pattern = pattern;

        slots.push(slot);
    }

    slots
}

/// Computes Euclidean distance from point `p` to segment `(a, b)`.
#[inline]
fn point_to_segment_distance(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let dx = b.0 - a.0;
    let dy = b.1 - a.1;
    let len_sq = dx * dx + dy * dy;
    if len_sq < 1e-12 {
        return (p.0 - a.0).hypot(p.1 - a.1);
    }
    let t = (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / len_sq).clamp(0.0, 1.0);
    let proj = (a.0 + t * dx, a.1 + t * dy);
    (p.0 - proj.0).hypot(p.1 - proj.1)
}
