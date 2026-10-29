//! `cadense-dsf` — Danish Sheet Fold (`.dsf`) Parametric Port & Template Engine
//!
//! Built additively for SolidCraft v0.25.1 (Format 41).

pub mod bending;
pub mod catalog;
pub mod container;
pub mod schema;
pub mod trapezoidal_scaler;

pub use bending::{
    bend_segment_intersects_loop, calc_bend_allowance, calc_bend_deduction, split_loop_by_half_plane, BackgaugeSpec, BendDirection, BendLine,
    BendRole, CylindricalBendArc3D, DsfRiggedSheet, FoldFaceNode, FoldedFaceNode, HoleCutout2D, Loop2D, Point2D, SheetMaterialRule,
    TimelineFeatureBundle, ToolingSpec, Transform3D,
};
pub use catalog::bonn_metal_catalog;
pub use container::{DsfContainer, DsfError, SolidCraftLiveSessionBridge, SOLIDCRAFT_FORMAT_VERSION_V0_25_1};
pub use schema::{
    CmsCategory, CoverStyle, DerivedFormula, DsfMeta, MaterialFinish, PortDefinition, PortDimensionOverrides, PortEqualityMode, RigidFeatureRule,
    SlotCountCondition, SolidCraftCommand, TdsViewConfig,
};
pub use trapezoidal_scaler::{IndependentPortSpec, MultiPortParametricSolution, TrapezoidalMultiPortScaler, TrapezoidalWallSegment};
