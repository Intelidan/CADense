//! Data structures and parameter definitions for SolidCraft CMS truFold.
//!
//! Provides the core data architecture for relaxed, rule-free sheet metal modeling,
//! multi-port parametric scaling, and pin-rigid hole preservation for industrial
//! Cable Management System (CMS) products (Bends, Tees, Reducers, Crosses, and Risers).

use serde::{Deserialize, Serialize};

/// Enumeration identifying the structurally classified CMS product type.
/// Derived automatically via vector analysis of the open perimeter ports.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CmsProductType {
    /// Standard 45° or 90° horizontal bend or riser.
    StandardBendOrRiser,
    /// Vertical inside riser (flanges fold up along an inside curve).
    InsideRiser,
    /// Vertical outside riser (flanges fold down along an outside curve).
    OutsideRiser,
    /// Concentric or eccentric width reducer fitting.
    Reducer,
    /// Equal or unequal branch 3-port tee fitting.
    UnequalTee,
    /// Equal or unequal 4-port cross intersection.
    UnequalCross,
    /// Custom non-standard transition fitting.
    CustomTransition,
}

impl CmsProductType {
    /// Human-readable label for UI ribbon displays.
    pub fn display_label(&self) -> &'static str {
        match self {
            Self::StandardBendOrRiser => "Standard Bend / Riser",
            Self::InsideRiser => "Vertical Inside Riser",
            Self::OutsideRiser => "Vertical Outside Riser",
            Self::Reducer => "Tapered Reducer",
            Self::UnequalTee => "Tee Fitting (Equal/Unequal)",
            Self::UnequalCross => "Cross Fitting (4-Way)",
            Self::CustomTransition => "Custom Transition",
        }
    }
}

/// Bending direction relative to the sheet normal ($+Z$ or $-Z$).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BendDirection {
    /// Flange folds toward positive $Z$.
    Up,
    /// Flange folds toward negative $Z$.
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

/// Metadata definition for an open connecting port boundary.
/// Tracks variable widths, heights, outward directional vectors, and dependent vertex indices.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CmsPortMetadata {
    /// Unique sequential port identifier (P1, P2, P3, P4).
    pub port_id: usize,
    /// User-facing label (e.g., "P1 - Main In", "P2 - Branch Run").
    pub name: String,
    /// Baseline manufactured width in millimeters from the DXF template.
    pub base_width: f64,
    /// Current dynamically scaled width in millimeters.
    pub current_width: f64,
    /// Baseline side wall / flange height in millimeters.
    pub base_height: f64,
    /// Current dynamically scaled flange height in millimeters.
    pub current_height: f64,
    /// Normalized outward-pointing 2D normal vector $(N_x, N_y)$.
    pub normal: (f64, f64),
    /// Midpoint coordinate $(X, Y)$ along the un-notched open edge in millimeters.
    pub center: (f64, f64),
    /// Ordered indices of perimeter sketch vertices displaced when this port's width shifts.
    pub vertex_indices: Vec<usize>,
}

impl CmsPortMetadata {
    pub fn new(
        port_id: usize,
        name: impl Into<String>,
        base_width: f64,
        base_height: f64,
        normal: (f64, f64),
        center: (f64, f64),
    ) -> Self {
        Self {
            port_id,
            name: name.into(),
            base_width,
            current_width: base_width,
            base_height,
            current_height: base_height,
            normal,
            center,
            vertex_indices: Vec::new(),
        }
    }

    /// Scalar width delta between current and base template dimensions.
    #[inline]
    pub fn delta_width(&self) -> f64 {
        self.current_width - self.base_width
    }

    /// Scalar height delta between current and base template dimensions.
    #[inline]
    pub fn delta_height(&self) -> f64 {
        self.current_height - self.base_height
    }
}

/// Metadata definition for a pin-rigid internal feature (e.g. coupler slots or round mounting holes).
/// Anchored to the nearest outer perimeter edge to translate without oval distortion.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CmsSlotMetadata {
    /// Unique internal feature index.
    pub slot_id: usize,
    /// Closed 2D polygon vertex loop of the hole or obround cutout.
    pub contour: Vec<(f64, f64)>,
    /// Calculated centroid $(X, Y)$ of the cutout.
    pub center: (f64, f64),
    /// Index of the nearest outer perimeter edge this cutout is anchored to.
    pub anchor_edge_id: usize,
    /// Relative offset vector $(dX, dY)$ from the anchor edge midpoint to the hole centroid.
    pub centroid_offset: (f64, f64),
    /// Flag indicating whether this cutout is a standard coupler connection slot.
    pub is_coupler_slot: bool,
    /// Optional pattern identifier (e.g. "OBROUND 20X8", "M10_BOLT").
    pub slot_pattern: Option<String>,
}

impl CmsSlotMetadata {
    pub fn new(
        slot_id: usize,
        contour: Vec<(f64, f64)>,
        center: (f64, f64),
        anchor_edge_id: usize,
        centroid_offset: (f64, f64),
    ) -> Self {
        Self {
            slot_id,
            contour,
            center,
            anchor_edge_id,
            centroid_offset,
            is_coupler_slot: false,
            slot_pattern: None,
        }
    }
}

/// Parameters driving a relaxed B-Rep solid fold feature.
/// Defines finite hinge endpoints, bend radius, angle, and moving flange face indices.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CmsFoldParams {
    /// Unique feature timeline identifier (e.g. "cms_fold_1").
    pub feature_id: String,
    /// Sequential execution order badge (①, ②, ③, ...).
    pub sequence: usize,
    /// Start coordinate $(X_1, Y_1)$ of the finite hinge line segment.
    pub hinge_p1: (f64, f64),
    /// End coordinate $(X_2, Y_2)$ of the finite hinge line segment.
    pub hinge_p2: (f64, f64),
    /// Inner bending radius (fillet value) in millimeters.
    pub inner_radius: f64,
    /// Formed fold angle in degrees ($0^\circ$ = flat blank, $90^\circ$ = standard wall, up to $180^\circ$ = hem).
    pub angle_deg: f64,
    /// Upward or downward fold direction.
    pub direction: BendDirection,
    /// Indices of face nodes rotated by this fold.
    pub moving_face_ids: Vec<usize>,
}

impl CmsFoldParams {
    pub fn new(
        feature_id: impl Into<String>,
        sequence: usize,
        hinge_p1: (f64, f64),
        hinge_p2: (f64, f64),
        inner_radius: f64,
        angle_deg: f64,
        direction: BendDirection,
    ) -> Self {
        Self {
            feature_id: feature_id.into(),
            sequence,
            hinge_p1,
            hinge_p2,
            inner_radius,
            angle_deg,
            direction,
            moving_face_ids: Vec::new(),
        }
    }

    /// Length of the finite hinge line in millimeters.
    #[inline]
    pub fn length(&self) -> f64 {
        let dx = self.hinge_p2.0 - self.hinge_p1.0;
        let dy = self.hinge_p2.1 - self.hinge_p1.1;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Aggregated parametric metadata schema for a CMS sheet metal component.
/// Holds the classified shape, detected ports, rigid coupler slot bindings, and active bend features.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CmsMetadata {
    /// Structural product classification (Reducer, Unequal Tee, Standard Bend, etc.).
    pub product_classification: CmsProductType,
    /// Commercial catalog name or Bonn SKU code.
    pub product_name: String,
    /// Physical sheet metal gauge thickness in millimeters (e.g. 1.2, 1.5, 2.0 mm).
    pub material_thickness_mm: f64,
    /// Default inner bending radius in millimeters.
    pub default_inner_radius_mm: f64,
    /// Discovered or configured connection ports.
    pub cms_ports: Vec<CmsPortMetadata>,
    /// Discovered interior cutout holes and coupler slots.
    pub rigid_slots: Vec<CmsSlotMetadata>,
    /// Chronological list of active relaxed bend line features.
    pub active_bends: Vec<CmsFoldParams>,
}

impl Default for CmsMetadata {
    fn default() -> Self {
        Self {
            product_classification: CmsProductType::StandardBendOrRiser,
            product_name: "CMS-COMPONENT".to_string(),
            material_thickness_mm: 1.5,
            default_inner_radius_mm: 1.5,
            cms_ports: Vec::new(),
            rigid_slots: Vec::new(),
            active_bends: Vec::new(),
        }
    }
}

impl CmsMetadata {
    pub fn new(
        product_classification: CmsProductType,
        product_name: impl Into<String>,
        thickness_mm: f64,
    ) -> Self {
        Self {
            product_classification,
            product_name: product_name.into(),
            material_thickness_mm: thickness_mm,
            default_inner_radius_mm: thickness_mm,
            cms_ports: Vec::new(),
            rigid_slots: Vec::new(),
            active_bends: Vec::new(),
        }
    }

    /// Finds a mutable port by its sequential ID.
    pub fn port_mut(&mut self, port_id: usize) -> Option<&mut CmsPortMetadata> {
        self.cms_ports.iter_mut().find(|p| p.port_id == port_id)
    }

    /// Finds an immutable port by its sequential ID.
    pub fn port(&self, port_id: usize) -> Option<&CmsPortMetadata> {
        self.cms_ports.iter().find(|p| p.port_id == port_id)
    }
}
