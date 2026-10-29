//! Workspace schema & tool definitions for `apps/cadense-studio`.
//! Bridges `cadense_dsf` (Step 1 & 2) with the Studio UI and `landing.rs`.

use serde::{Deserialize, Serialize};
pub use cadense_dsf::{
    BendDirection, BendLine, CylindricalBendArc3D, DsfContainer, DsfMeta, DsfRiggedSheet,
    FoldedFaceNode, HoleCutout2D, Loop2D, Point2D, PortDefinition, SheetMaterialRule,
    SolidCraftLiveSessionBridge, TimelineFeatureBundle, Transform3D,
};

/// Active interactive tool in the 3D Viewport & CMS Ribbon Toolbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CmsViewportTool {
    /// Standard SolidCraft 3D Orbit / Pan / Zoom navigation
    Navigate3D,
    /// Interactive 2-point bend line drawing on the 2D sheet or pre-drawn return flange
    DrawBendLine,
    /// Interactive tagging of rigid obround coupler mounting slots (`OBROUND 20X8`, `OBROUND 30X7`, etc.)
    TagCouplerSlot,
    /// Interactive port edge binding (`W1..W4`, `H1..H4`)
    BindPortEdge,
}

impl Default for CmsViewportTool {
    fn default() -> Self {
        Self::Navigate3D
    }
}

/// Coupler slot standard presets used by Bonn Metal CMS catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CouplerSlotPreset {
    Obround20x8,
    Obround30x7,
    Obround50x7,
    Obround29x12,
}

impl CouplerSlotPreset {
    pub fn tag_name(&self) -> &'static str {
        match self {
            Self::Obround20x8 => "OBROUND 20X8",
            Self::Obround30x7 => "OBROUND 30X7",
            Self::Obround50x7 => "OBROUND 50X7",
            Self::Obround29x12 => "OBROUND 29X12",
        }
    }

    pub fn dimensions_mm(&self) -> (f64, f64) {
        match self {
            Self::Obround20x8 => (20.0, 8.0),
            Self::Obround30x7 => (30.0, 7.0),
            Self::Obround50x7 => (50.0, 7.0),
            Self::Obround29x12 => (29.0, 12.0),
        }
    }

    pub fn all() -> &'static [CouplerSlotPreset] {
        &[
            Self::Obround20x8,
            Self::Obround30x7,
            Self::Obround50x7,
            Self::Obround29x12,
        ]
    }
}