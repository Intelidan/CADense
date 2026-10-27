//! CADense Version & Engine Introspection System.
//!
//! Exposes synchronized versioning for CADense Studio, TruFold CMS Core, and upstream SolidCraft CAD.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CadenseAppVersion {
    /// Product branding name
    pub app_name: &'static str,
    /// CADense Studio semver release
    pub app_version: &'static str,
    /// Enterprise build identifier
    pub build_id: &'static str,
    /// Release channel (e.g. "Industrial Production")
    pub release_channel: &'static str,
    /// Core engine name
    pub core_engine: &'static str,
    /// Core engine semver
    pub core_version: &'static str,
    /// Integrated upstream SolidCraft version
    pub solidcraft_version: &'static str,
    /// Native 3D CAD design format schema version
    pub solidcraft_format: u32,
    /// Release date string
    pub release_date: &'static str,
}

impl CadenseAppVersion {
    /// Formatted one-line summary string.
    pub fn summary_line(&self) -> String {
        format!(
            "{} {} (Build {}) | {} {} | SolidCraft {} (Format {})",
            self.app_name,
            self.app_version,
            self.build_id,
            self.core_engine,
            self.core_version,
            self.solidcraft_version,
            self.solidcraft_format
        )
    }

    /// Compact badge text for sidebar or header.
    pub fn compact_badge(&self) -> String {
        format!("{} • SC {}", self.app_version, self.solidcraft_version)
    }
}

/// The authoritative current version descriptor for CADense Studio.
pub const CURRENT_VERSION: CadenseAppVersion = CadenseAppVersion {
    app_name: "CADense Studio",
    app_version: "v1.0.0",
    build_id: "2026.10-PROD",
    release_channel: "Industrial Enterprise",
    core_engine: "TruFold CMS Core",
    core_version: "v0.2.0",
    solidcraft_version: "v0.24.0",
    solidcraft_format: 39,
    release_date: "2026-10-27",
};
