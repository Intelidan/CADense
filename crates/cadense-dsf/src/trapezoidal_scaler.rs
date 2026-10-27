//! Independent Port Width (`W1..W4`) and Trapezoidal Mixed-Height (`H1..H4`) Parametric Solver.
//!
//! Ensures that when any single port's Width or Height changes on an Unequal Tee,
//! Unequal Cross, or Reducer:
//! 1. All other ports retain their exact independent `W_i` and `H_i` dimensions.
//! 2. Floor transition edges slant in 2D (`X-Y`) between `W_start` and `W_end`.
//! 3. Transition sidewalls unfold as 2D trapezoids (`H_start` -> `H_end`) that fold into
//!    continuous 3D slanted walls without steps or gaps.
//! 4. Rigid coupler obround holes (`OBROUND 20X8`, `OBROUND 30X7`) stay locked at their
//!    port mouth offsets, and discrete slot counts (`1` vs `2` slots) are computed per port.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Independent dimensional state of a single physical port `i` (`1..=4`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct IndependentPortSpec {
    pub port_id: usize,
    pub width_mm: f64,
    pub height_mm: f64,
}

/// 2D/3D geometry of a slanted/tapered transition wall connecting `from_port` to `to_port`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrapezoidalWallSegment {
    pub wall_id: String,
    pub from_port_id: usize,
    pub to_port_id: usize,
    /// Horizontal offset in X-Y caused by unequal port widths `(W_from - W_to) / 2` or branch offset.
    pub plan_offset_x_mm: f64,
    /// Axial run length along the fitting transition zone.
    pub plan_run_y_mm: f64,
    /// True 2D bend-line length of the slanted wall (`hypot(plan_offset_x_mm, plan_run_y_mm)`).
    pub bend_line_length_mm: f64,
    /// Plan-view miter/taper angle in degrees in the X-Y base floor plane.
    pub plan_taper_angle_deg: f64,
    /// Unfolded flange height at the `from_port` end (`H_from`).
    pub start_height_mm: f64,
    /// Unfolded flange height at the `to_port` end (`H_to`).
    pub end_height_mm: f64,
    /// 3D top-edge ramp angle in degrees (`atan2(end_height - start_height, bend_line_length)`).
    pub top_edge_slope_angle_deg: f64,
    /// True length of the slanted top edge in 3D/2D (`hypot(bend_line_length, H_to - H_from)`).
    pub slanted_top_edge_length_mm: f64,
    /// Local 2D trapezoid vertices `[(0, 0), (L_bend, 0), (L_bend, H_end), (0, H_start)]`.
    pub unfolded_trapezoid_2d: [(f64, f64); 4],
    /// Discrete coupler slot count at `from_port` end (`1` if `H_from <= 75`, `2` if `H_from >= 100`).
    pub start_port_slot_count: u32,
    /// Discrete coupler slot count at `to_port` end (`1` if `H_to <= 75`, `2` if `H_to >= 100`).
    pub end_port_slot_count: u32,
}

/// Complete multi-port unequal solution for 2D unfolded flat patterns, 3D meshes, and SolidCraft parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiPortParametricSolution {
    pub ports: BTreeMap<usize, IndependentPortSpec>,
    pub transition_walls: Vec<TrapezoidalWallSegment>,
    pub per_port_slot_counts: BTreeMap<usize, u32>,
    /// Cover slope angle in degrees when `H_i != H_j`.
    pub max_cover_slope_deg: f64,
    /// Parameter map emitted to SolidCraft `parameters.set`.
    pub solidcraft_parameters: BTreeMap<String, f64>,
}

pub struct TrapezoidalMultiPortScaler;

impl TrapezoidalMultiPortScaler {
    /// Evaluates independent port widths (`W1..W4`) and heights (`H1..H4`) and computes
    /// all slanted width tapers and trapezoidal height walls between connected ports.
    pub fn solve(ports: &BTreeMap<usize, (f64, f64)>, transition_axial_length_mm: f64) -> MultiPortParametricSolution {
        let mut port_specs = BTreeMap::new();
        let mut per_port_slot_counts = BTreeMap::new();
        let mut solidcraft_parameters = BTreeMap::new();

        for (&port_id, &(w, h)) in ports {
            let clean_w = w.max(25.0);
            let clean_h = h.max(20.0);
            port_specs.insert(port_id, IndependentPortSpec { port_id, width_mm: clean_w, height_mm: clean_h });

            // Per-port discrete slot rule: 1 slot when H_i <= 75mm, 2 slots when H_i >= 100mm
            let slots = if clean_h >= 100.0 { 2 } else { 1 };
            per_port_slot_counts.insert(port_id, slots);

            solidcraft_parameters.insert(format!("#W{}", port_id), clean_w);
            solidcraft_parameters.insert(format!("#H{}", port_id), clean_h);
            solidcraft_parameters.insert(format!("#PORT{}_SLOTS", port_id), slots as f64);
        }

        // Default primary #W and #H to Port 1 for backwards compatibility with single-value expressions
        if let Some(p1) = port_specs.get(&1) {
            solidcraft_parameters.insert("#W".to_string(), p1.width_mm);
            solidcraft_parameters.insert("#H".to_string(), p1.height_mm);
        }

        let run_len = transition_axial_length_mm.max(50.0);
        let mut transition_walls = Vec::new();
        let mut max_cover_slope_deg: f64 = 0.0;

        // Build transition walls depending on the number of ports (2 = Reducer/Bend, 3 = Tee, 4 = Cross)
        let pairs: Vec<(&str, usize, usize)> = match port_specs.len() {
            2 => vec![("LEFT_WALL_P1_P2", 1, 2), ("RIGHT_WALL_P1_P2", 1, 2)],
            3 => vec![("MAIN_WALL_BACK_P1_P3", 1, 3), ("BRANCH_LEFT_P1_P2", 1, 2), ("BRANCH_RIGHT_P2_P3", 2, 3)],
            4 => vec![("CORNER_NW_P1_P2", 1, 2), ("CORNER_NE_P2_P3", 2, 3), ("CORNER_SE_P3_P4", 3, 4), ("CORNER_SW_P4_P1", 4, 1)],
            _ => Vec::new(),
        };

        for (wall_label, from_id, to_id) in pairs {
            if let (Some(p_from), Some(p_to)) = (port_specs.get(&from_id), port_specs.get(&to_id)) {
                let seg = Self::build_wall_segment(wall_label, *p_from, *p_to, run_len);
                if seg.top_edge_slope_angle_deg.abs() > max_cover_slope_deg.abs() {
                    max_cover_slope_deg = seg.top_edge_slope_angle_deg;
                }

                solidcraft_parameters.insert(format!("#{}_BEND_LEN", wall_label), seg.bend_line_length_mm);
                solidcraft_parameters.insert(format!("#{}_PLAN_TAPER_DEG", wall_label), seg.plan_taper_angle_deg);
                solidcraft_parameters.insert(format!("#{}_H_START", wall_label), seg.start_height_mm);
                solidcraft_parameters.insert(format!("#{}_H_END", wall_label), seg.end_height_mm);
                solidcraft_parameters.insert(format!("#{}_SLOPE_DEG", wall_label), seg.top_edge_slope_angle_deg);

                transition_walls.push(seg);
            }
        }

        solidcraft_parameters.insert("#COVER_SLOPE_DEG".to_string(), max_cover_slope_deg);

        MultiPortParametricSolution { ports: port_specs, transition_walls, per_port_slot_counts, max_cover_slope_deg, solidcraft_parameters }
    }

    fn build_wall_segment(wall_id: &str, p_from: IndependentPortSpec, p_to: IndependentPortSpec, run_len_mm: f64) -> TrapezoidalWallSegment {
        // Width difference creates a horizontal taper offset in X-Y
        let plan_offset_x_mm = (p_from.width_mm - p_to.width_mm).abs() * 0.5;
        let bend_line_length_mm = plan_offset_x_mm.hypot(run_len_mm);
        let plan_taper_angle_deg = plan_offset_x_mm.atan2(run_len_mm).to_degrees();

        // Height difference creates a trapezoidal unfolded flange that folds into a 3D slanted wall
        let delta_h = p_to.height_mm - p_from.height_mm;
        let top_edge_slope_angle_deg = delta_h.atan2(bend_line_length_mm).to_degrees();
        let slanted_top_edge_length_mm = bend_line_length_mm.hypot(delta_h);

        let unfolded_trapezoid_2d = [(0.0, 0.0), (bend_line_length_mm, 0.0), (bend_line_length_mm, p_to.height_mm), (0.0, p_from.height_mm)];

        let start_port_slot_count = if p_from.height_mm >= 100.0 { 2 } else { 1 };
        let end_port_slot_count = if p_to.height_mm >= 100.0 { 2 } else { 1 };

        TrapezoidalWallSegment {
            wall_id: wall_id.to_string(),
            from_port_id: p_from.port_id,
            to_port_id: p_to.port_id,
            plan_offset_x_mm,
            plan_run_y_mm: run_len_mm,
            bend_line_length_mm,
            plan_taper_angle_deg,
            start_height_mm: p_from.height_mm,
            end_height_mm: p_to.height_mm,
            top_edge_slope_angle_deg,
            slanted_top_edge_length_mm,
            unfolded_trapezoid_2d,
            start_port_slot_count,
            end_port_slot_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unequal_tee_independent_width_and_slanted_height() {
        let mut ports = BTreeMap::new();
        // Port 1: W1=600, H1=100 (changed independently!)
        ports.insert(1, (600.0, 100.0));
        // Port 2 (branch): W2=300, H2=50 (remains at 50!)
        ports.insert(2, (300.0, 50.0));
        // Port 3 (main run): W3=450, H3=50 (remains at 50!)
        ports.insert(3, (450.0, 50.0));

        let sol = TrapezoidalMultiPortScaler::solve(&ports, 150.0);

        // 1. Verify each port kept its exact independent Width and Height
        assert_eq!(sol.ports[&1].width_mm, 600.0);
        assert_eq!(sol.ports[&1].height_mm, 100.0);
        assert_eq!(sol.ports[&2].width_mm, 300.0);
        assert_eq!(sol.ports[&2].height_mm, 50.0);
        assert_eq!(sol.ports[&3].width_mm, 450.0);
        assert_eq!(sol.ports[&3].height_mm, 50.0);

        // 2. Verify per-port slot multiplication: Port 1 (H1=100) gets 2 slots; Ports 2 & 3 (H=50) get 1 slot
        assert_eq!(sol.per_port_slot_counts[&1], 2);
        assert_eq!(sol.per_port_slot_counts[&2], 1);
        assert_eq!(sol.per_port_slot_counts[&3], 1);

        // 3. Verify transition wall between Port 1 (H1=100) and Port 2 (H2=50) is a slanted trapezoid
        let p1_p2_wall = &sol.transition_walls[1];
        assert_eq!(p1_p2_wall.start_height_mm, 100.0);
        assert_eq!(p1_p2_wall.end_height_mm, 50.0);
        assert!(p1_p2_wall.top_edge_slope_angle_deg < 0.0); // Slants downward from 100mm to 50mm
        assert!(p1_p2_wall.plan_taper_angle_deg > 0.0); // Tapers in X-Y from W1=600 to W2=300
    }
}
