//! `cadense-dsf` — Danish Sheet Fold (`.dsf`) Parametric Port & Template Engine
//!
//! Built additively for SolidCraft v0.23.1+ (Format 37).
//! Provides:
//! - `.dsf` container read/write (`design.solidcraft` + `dsf_meta.json`)
//! - Port Assignment (`P1`..`P4`) with `AllEqual`, `SymmetricOpposite`, and `Unequal` modes
//! - Rigid feature locking (`OBROUND 30X7`, `OBROUND 20X8`, `OBROUND 50X7`, `OBROUND 29X12`)
//! - Discrete conditional slot-count evaluation (e.g., `H >= 100 -> 2 slots`, `H <= 75 -> 1 slot`)
//! - Derived clearance offsets (e.g., `Actual_W = W - 5mm` for Trunking End Caps & Couplers)
//! - Engine command batch generation (`parameters.set`, `sheet.manage_rules`, `sheet.flat_pattern`)

pub mod catalog;
pub mod container;
pub mod schema;
pub mod trapezoidal_scaler;

pub use catalog::bonn_metal_catalog;
pub use container::{DsfContainer, DsfError};
pub use schema::{
    CmsCategory, CoverStyle, DerivedFormula, DsfMeta, MaterialFinish, PortDefinition,
    PortDimensionOverrides, PortEqualityMode, RigidFeatureRule, SlotCountCondition,
    SolidCraftCommand, TdsViewConfig,
};
pub use trapezoidal_scaler::{
    IndependentPortSpec, MultiPortParametricSolution, TrapezoidalMultiPortScaler,
    TrapezoidalWallSegment,
};

