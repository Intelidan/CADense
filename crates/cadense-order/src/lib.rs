//! `cadense-order` — Phase 3 Customer Order CSV Parser, Clearance & Slot Rule Resolver,
//! and Interactive Order Queue Engine for CADense on SolidCraft v0.23.1+ (Format 38).
//!
//! Complies strictly with Rule R1 (Additive-Only Compliance): drives SolidCraft via
//! engine command payloads (`parameters.set`, `sheet.manage_rules`, `sheet.flat_pattern`)
//! and outer `.dsf` ("Danish Sheet Fold") containers without modifying `crates/doc/src/format.rs`.

pub mod csv_parser;
pub mod queue;
pub mod resolver;
pub mod taxonomy;

pub use csv_parser::{CustomerOrderCsvParser, OrderCsvDiagnostic, OrderCsvParseReport, RawOrderRow};
pub use queue::{OrderQueueAction, OrderQueueEngine, OrderQueueFilter, OrderQueueItem, OrderQueueItemStatus, OrderQueueSummary};
pub use resolver::{
    ClearanceAdjustment, CoupledCoverSpec, EngineCommandPayload, ObroundSlotSpec, ResolvedOrderSpec, ResolverError, SheetMetalRuleSpec,
    TaxClearanceResolver,
};
pub use taxonomy::{BonnCatalogCode, CmsCategory, CmsFinishType, PortEqualityMode, ProductVariationEntry, ProductVariationRegistry};

/// Target SolidCraft document format version supported by this crate.
pub const SOLIDCRAFT_TARGET_FORMAT_VERSION: u32 = 38;
/// Target SolidCraft upstream version string.
pub const SOLIDCRAFT_TARGET_RELEASE: &str = "v0.23.1+";
