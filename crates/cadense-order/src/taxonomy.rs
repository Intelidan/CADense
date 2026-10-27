//! Master 65-Variation Product Taxonomy (`references/product_variations.csv`) and
//! Bonn Metal 16-Sheet Family `.dsf` Preset Mapping.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TaxonomyError {
    #[error("I/O error reading product_variations.csv: {0}")]
    Io(#[from] std::io::Error),
    #[error("CSV parse error in product_variations.csv: {0}")]
    Csv(#[from] csv::Error),
    #[error("Unrecognized product name or variation: '{0}'")]
    UnknownProduct(String),
}

/// The 16 Bonn Metal Cons. Ind. `.dsf` catalog codes covering all 37 submittal sheets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BonnCatalogCode {
    /// Sheets 1–4 (SN 01–07): Cable Tray Straight & W/Cover
    CtStr,
    /// Sheets 5, 10 (SN 08, 13): Cable Tray Bend 45 Deg
    CtB45,
    /// Sheet 6 (SN 09): Cable Tray Adjustable Inside Riser
    CtRiserIn,
    /// Sheet 8 (SN 11, 27, 30): Cable Tray Adjustable Outside Riser
    CtRiserOut,
    /// Sheet 7 (SN 10, 16, 21): Cable Tray Unequal Tee
    CtUtee,
    /// Sheets 11, 14 (SN 14, 18, 24, 31): Cable Tray Equal Tee
    CtEtee,
    /// Sheet 12 (SN 15): Cable Tray Equal Cross
    CtEcross,
    /// Sheets 9, 15–18 (SN 12, 19, 20, 22, 23, 25, 26, 28, 29): Cable Tray Bend 90 Deg & W/Cover
    CtB90,
    /// Sheet 13 (SN 17, 32): Cable Tray Straight Reducer
    CtRed,
    /// Sheets 19–20 (SN 33, 34): Cable Tray Straight Coupler
    CtCpl,
    /// Sheets 21–23 (SN 37–40): Cable Trunking Straight W/Cover
    TrkStr,
    /// Sheets 24–26 (SN 41–44): Cable Trunking Coupler
    TrkCpl,
    /// Sheets 27–29 (SN 45–47): Cable Trunking Reducer W/Cover
    TrkRed,
    /// Sheets 30–33 (SN 51–54): Cable Trunking Equal Tee W/Cover
    TrkEtee,
    /// Sheets 34–36 (SN 59–62): Cable Trunking End Cap
    TrkEnd,
    /// Sheet 37 (SN 70): Unistrut Slotted Channel 41x41
    Uni41,
}

impl BonnCatalogCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CtStr => "CT-STR",
            Self::CtB45 => "CT-B45",
            Self::CtRiserIn => "CT-RISER-IN",
            Self::CtRiserOut => "CT-RISER-OUT",
            Self::CtUtee => "CT-UTEE",
            Self::CtEtee => "CT-ETEE",
            Self::CtEcross => "CT-ECROSS",
            Self::CtB90 => "CT-B90",
            Self::CtRed => "CT-RED",
            Self::CtCpl => "CT-CPL",
            Self::TrkStr => "TRK-STR",
            Self::TrkCpl => "TRK-CPL",
            Self::TrkRed => "TRK-RED",
            Self::TrkEtee => "TRK-ETEE",
            Self::TrkEnd => "TRK-END",
            Self::Uni41 => "UNI-41",
        }
    }

    pub fn dsf_template_path(&self) -> String {
        format!("templates/bonn_metal/{}.dsf", self.as_str().to_lowercase())
    }

    pub fn port_count(&self) -> usize {
        match self {
            Self::TrkEnd | Self::CtCpl | Self::TrkCpl | Self::Uni41 => 1,
            Self::CtStr
            | Self::CtB45
            | Self::CtRiserIn
            | Self::CtRiserOut
            | Self::CtB90
            | Self::CtRed
            | Self::TrkStr
            | Self::TrkRed => 2,
            Self::CtUtee | Self::CtEtee | Self::TrkEtee => 3,
            Self::CtEcross => 4,
        }
    }

    pub fn equality_mode(&self) -> PortEqualityMode {
        match self {
            Self::CtUtee | Self::CtRed | Self::TrkRed => PortEqualityMode::Unequal,
            Self::CtEtee | Self::TrkEtee | Self::CtEcross => PortEqualityMode::EqualAllPorts,
            _ => PortEqualityMode::SymmetricTwoPort,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortEqualityMode {
    SymmetricTwoPort,
    EqualAllPorts,
    Unequal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CmsCategory {
    CableTray,
    TrayAccessories,
    CableTrunking,
    TrunkingAccessories,
    SupportChannels,
}

impl CmsCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CableTray => "Cable Tray",
            Self::TrayAccessories => "Tray Accessories",
            Self::CableTrunking => "Cable Trunking",
            Self::TrunkingAccessories => "Trunking Accessories",
            Self::SupportChannels => "Support Channels",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CmsFinishType {
    /// Hot-Dip Galvanized (BS EN ISO 1461) — standard for Bonn Metal Cable Trays
    Hdg,
    /// Pre-Galvanized / Galvanized Iron (BS EN 10346) — standard for Trunking & Unistrut
    Gi,
    /// Stainless Steel 316L (optional taxonomy extension)
    Ss316L,
    /// Powder Coated RAL (optional taxonomy extension)
    PowderCoated,
}

impl CmsFinishType {
    pub fn from_token(raw: &str) -> Self {
        let token = raw.trim().to_uppercase();
        if token.contains("HDG") || token.contains("HOT DIP") || token.contains("HOT-DIP") {
            Self::Hdg
        } else if token.contains("SS") || token.contains("316") {
            Self::Ss316L
        } else if token.contains("POWDER") || token.contains("EPOXY") {
            Self::PowderCoated
        } else {
            Self::Gi
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::Hdg => "HDG",
            Self::Gi => "GI",
            Self::Ss316L => "SS316L",
            Self::PowderCoated => "PC",
        }
    }

    pub fn tds_note_label(&self) -> &'static str {
        match self {
            Self::Hdg => "NOTE:- MATERIAL FINISH HDG",
            Self::Gi => "NOTE:- MATERIAL FINISH GI",
            Self::Ss316L => "NOTE:- MATERIAL FINISH SS316L",
            Self::PowderCoated => "NOTE:- MATERIAL FINISH POWDER COATED",
        }
    }
}

/// Represents one variation row in `references/product_variations.csv`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductVariationEntry {
    pub variation_id: u32,
    pub category: CmsCategory,
    pub product_family: String,
    pub variation_name: String,
    pub catalog_code: BonnCatalogCode,
    pub port_count: usize,
    pub default_with_cover: bool,
    pub default_finish: CmsFinishType,
    pub default_thickness_mm: f64,
    pub default_length_mm: Option<f64>,
    pub has_sectional_inset: bool,
}

/// Registry holding the 65 canonical product variations from `references/product_variations.csv`.
#[derive(Debug, Clone)]
pub struct ProductVariationRegistry {
    entries: Vec<ProductVariationEntry>,
    lookup_by_normalized_name: HashMap<String, usize>,
}

impl Default for ProductVariationRegistry {
    fn default() -> Self {
        Self::new_builtin_65()
    }
}

impl ProductVariationRegistry {
    /// Builds the full 65-variation taxonomy registry matching `references/product_variations.csv`.
    pub fn new_builtin_65() -> Self {
        use BonnCatalogCode::*;
        use CmsCategory::*;
        use CmsFinishType::*;

        let raw_specs: Vec<(
            u32,
            CmsCategory,
            &str,
            &str,
            BonnCatalogCode,
            bool,
            CmsFinishType,
            f64,
            Option<f64>,
            bool,
        )> = vec![
            // 1–8: Cable Tray Straight variations (Sheets 1–4, SN 01–07)
            (1, CableTray, "Perforated Cable Tray", "CABLE TRAY STRAIGHT", CtStr, false, Hdg, 1.5, Some(3000.0), true),
            (2, CableTray, "Perforated Cable Tray", "CABLE TRAY STRAIGHT W/COVER", CtStr, true, Hdg, 1.5, Some(3000.0), true),
            (3, CableTray, "Perforated Cable Tray", "CABLE TRAY STRAIGHT HD 2.0MM", CtStr, false, Hdg, 2.0, Some(3000.0), true),
            (4, CableTray, "Perforated Cable Tray", "CABLE TRAY STRAIGHT W/COVER HD 2.0MM", CtStr, true, Hdg, 2.0, Some(3000.0), true),
            (5, CableTray, "Perforated Cable Tray", "CABLE TRAY STRAIGHT GI", CtStr, false, Gi, 1.5, Some(3000.0), true),
            (6, CableTray, "Perforated Cable Tray", "CABLE TRAY STRAIGHT W/COVER GI", CtStr, true, Gi, 1.5, Some(3000.0), true),
            (7, CableTray, "Solid Bottom Cable Tray", "CABLE TRAY NON-PERFORATED STRAIGHT", CtStr, false, Hdg, 1.5, Some(3000.0), true),
            (8, CableTray, "Solid Bottom Cable Tray", "CABLE TRAY NON-PERFORATED STRAIGHT W/COVER", CtStr, true, Hdg, 1.5, Some(3000.0), true),
            // 9–14: Cable Tray 45-Deg Bend variations (Sheets 5, 10, SN 08, 13)
            (9, TrayAccessories, "45 Degree Flat Cable Tray Bend", "CABLE TRAY BEND 45DEG", CtB45, false, Hdg, 2.0, None, false),
            (10, TrayAccessories, "45 Degree Flat Cable Tray Bend", "CABLE TRAY BEND 45DEG W/COVER", CtB45, true, Hdg, 2.0, None, false),
            (11, TrayAccessories, "45 Degree Flat Cable Tray Bend", "CABLE TRAY BEND 45DEG 1.5MM", CtB45, false, Hdg, 1.5, None, false),
            (12, TrayAccessories, "45 Degree Flat Cable Tray Bend", "CABLE TRAY BEND 45DEG W/COVER 1.5MM", CtB45, true, Hdg, 1.5, None, false),
            (13, TrayAccessories, "45 Degree Flat Cable Tray Bend", "CABLE TRAY BEND 45DEG GI", CtB45, false, Gi, 1.5, None, false),
            (14, TrayAccessories, "45 Degree Flat Cable Tray Bend", "CABLE TRAY BEND 45DEG W/COVER GI", CtB45, true, Gi, 1.5, None, false),
            // 15–20: Cable Tray 90-Deg Bend variations (Sheets 9, 15–18, SN 12, 19, 20, 22, 23, 25, 26, 28, 29)
            (15, TrayAccessories, "90 Degree Flat Cable Tray Bend", "CABLE TRAY BEND 90DEG", CtB90, false, Hdg, 2.0, None, false),
            (16, TrayAccessories, "90 Degree Flat Cable Tray Bend", "CABLE TRAY BEND 90DEG W/COVER", CtB90, true, Hdg, 2.0, None, false),
            (17, TrayAccessories, "90 Degree Flat Cable Tray Bend", "CABLE TRAY BEND 90DEG 1.5MM", CtB90, false, Hdg, 1.5, None, false),
            (18, TrayAccessories, "90 Degree Flat Cable Tray Bend", "CABLE TRAY BEND 90DEG W/COVER 1.5MM", CtB90, true, Hdg, 1.5, None, false),
            (19, TrayAccessories, "90 Degree Flat Cable Tray Bend", "CABLE TRAY BEND 90DEG GI", CtB90, false, Gi, 1.5, None, false),
            (20, TrayAccessories, "90 Degree Flat Cable Tray Bend", "CABLE TRAY BEND 90DEG W/COVER GI", CtB90, true, Gi, 1.5, None, false),
            // 21–26: Adjustable Inside & Outside Risers (Sheets 6, 8, SN 09, 11, 27, 30)
            (21, TrayAccessories, "Cable Tray Inside Riser", "CABLE TRAY ADJUSTABLE INSIDE RISER", CtRiserIn, false, Hdg, 2.0, None, false),
            (22, TrayAccessories, "Cable Tray Inside Riser", "CABLE TRAY ADJUSTABLE INSIDE RISER W/COVER", CtRiserIn, true, Hdg, 2.0, None, false),
            (23, TrayAccessories, "Cable Tray Inside Riser", "CABLE TRAY INSIDE RISER 90DEG", CtRiserIn, false, Hdg, 1.5, None, false),
            (24, TrayAccessories, "Cable Tray Outside Riser", "CABLE TRAY ADJUSTABLE OUTSIDE RISER", CtRiserOut, false, Hdg, 2.0, None, false),
            (25, TrayAccessories, "Cable Tray Outside Riser", "CABLE TRAY ADJUSTABLE OUTSIDE RISER W/COVER", CtRiserOut, true, Hdg, 2.0, None, false),
            (26, TrayAccessories, "Cable Tray Outside Riser", "CABLE TRAY OUTSIDE RISER 90DEG", CtRiserOut, false, Hdg, 1.5, None, false),
            // 27–32: Cable Tray Equal & Unequal Tees (Sheets 7, 11, 14, SN 10, 14, 16, 18, 21, 24, 31)
            (27, TrayAccessories, "Equal Cable Tray Tee", "CABLE TRAY EQUAL TEE", CtEtee, false, Hdg, 2.0, None, false),
            (28, TrayAccessories, "Equal Cable Tray Tee", "CABLE TRAY EQUAL TEE W/COVER", CtEtee, true, Hdg, 2.0, None, false),
            (29, TrayAccessories, "Equal Cable Tray Tee", "CABLE TRAY EQUAL TEE 1.5MM", CtEtee, false, Hdg, 1.5, None, false),
            (30, TrayAccessories, "Unequal Cable Tray Tee", "CABLE TRAY UNEQUAL TEE", CtUtee, false, Hdg, 2.0, None, false),
            (31, TrayAccessories, "Unequal Cable Tray Tee", "CABLE TRAY UNEQUAL TEE W/COVER", CtUtee, true, Hdg, 2.0, None, false),
            (32, TrayAccessories, "Unequal Cable Tray Tee", "CABLE TRAY UNEQUAL TEE 1.5MM", CtUtee, false, Hdg, 1.5, None, false),
            // 33–36: Cable Tray 4-Way Crosses (Sheet 12, SN 15)
            (33, TrayAccessories, "4-Way Cable Tray Cross", "CABLE TRAY EQUAL CROSS", CtEcross, false, Hdg, 2.0, None, false),
            (34, TrayAccessories, "4-Way Cable Tray Cross", "CABLE TRAY EQUAL CROSS W/COVER", CtEcross, true, Hdg, 2.0, None, false),
            (35, TrayAccessories, "4-Way Cable Tray Cross", "CABLE TRAY EQUAL CROSS 1.5MM", CtEcross, false, Hdg, 1.5, None, false),
            (36, TrayAccessories, "4-Way Cable Tray Cross", "CABLE TRAY EQUAL CROSS GI", CtEcross, false, Gi, 1.5, None, false),
            // 37–42: Cable Tray Reducers (Sheet 13, SN 17, 32)
            (37, TrayAccessories, "Unequal Cable Tray Reducer", "CABLE TRAY STRAIGHT REDUCER", CtRed, false, Hdg, 2.0, None, false),
            (38, TrayAccessories, "Unequal Cable Tray Reducer", "CABLE TRAY STRAIGHT REDUCER W/COVER", CtRed, true, Hdg, 2.0, None, false),
            (39, TrayAccessories, "Unequal Cable Tray Reducer", "CABLE TRAY LEFT HAND REDUCER", CtRed, false, Hdg, 2.0, None, false),
            (40, TrayAccessories, "Unequal Cable Tray Reducer", "CABLE TRAY RIGHT HAND REDUCER", CtRed, false, Hdg, 2.0, None, false),
            (41, TrayAccessories, "Unequal Cable Tray Reducer", "CABLE TRAY STRAIGHT REDUCER 1.5MM", CtRed, false, Hdg, 1.5, None, false),
            (42, TrayAccessories, "Unequal Cable Tray Reducer", "CABLE TRAY STRAIGHT REDUCER GI", CtRed, false, Gi, 1.5, None, false),
            // 43–45: Cable Tray Straight Couplers (Sheets 19–20, SN 33, 34)
            (43, TrayAccessories, "Cable Tray Coupler", "CABLE TRAY STRAIGHT COUPLER", CtCpl, false, Hdg, 2.0, Some(180.0), false),
            (44, TrayAccessories, "Cable Tray Coupler", "CABLE TRAY STRAIGHT COUPLER H50", CtCpl, false, Hdg, 2.0, Some(180.0), false),
            (45, TrayAccessories, "Cable Tray Coupler", "CABLE TRAY STRAIGHT COUPLER H75", CtCpl, false, Hdg, 2.0, Some(180.0), false),
            // 46–50: Cable Trunking Straight (Sheets 21–23, SN 37–40)
            (46, CableTrunking, "Cable Trunking Straight", "CABLE TRUNKING W/COVER", TrkStr, true, Gi, 1.2, Some(3000.0), true),
            (47, CableTrunking, "Cable Trunking Straight", "CABLE TRUNKING STRAIGHT W/COVER", TrkStr, true, Gi, 1.2, Some(3000.0), true),
            (48, CableTrunking, "Cable Trunking Straight", "CABLE TRUNKING STRAIGHT 1.5MM W/COVER", TrkStr, true, Gi, 1.5, Some(3000.0), true),
            (49, CableTrunking, "Cable Trunking Straight", "CABLE TRUNKING W/COVER HDG", TrkStr, true, Hdg, 1.5, Some(3000.0), true),
            (50, CableTrunking, "Cable Trunking Straight", "CABLE TRUNKING WITHOUT COVER", TrkStr, false, Gi, 1.2, Some(3000.0), true),
            // 51–54: Cable Trunking Couplers (Sheets 24–26, SN 41–44)
            (51, TrunkingAccessories, "Cable Trunking Coupler", "CABLE TRUNKING COUPLER", TrkCpl, false, Gi, 1.2, None, false),
            (52, TrunkingAccessories, "Cable Trunking Coupler", "CABLE TRUNKING COUPLER H50", TrkCpl, false, Gi, 1.2, Some(50.0), false),
            (53, TrunkingAccessories, "Cable Trunking Coupler", "CABLE TRUNKING COUPLER H75", TrkCpl, false, Gi, 1.2, Some(70.0), false),
            (54, TrunkingAccessories, "Cable Trunking Coupler", "CABLE TRUNKING COUPLER H150", TrkCpl, false, Gi, 1.2, Some(70.0), false),
            // 55–57: Cable Trunking Reducers (Sheets 27–29, SN 45–47)
            (55, TrunkingAccessories, "Cable Trunking Reducer", "CABLE TRUNKING REDUCER W/COVER", TrkRed, true, Gi, 1.2, None, false),
            (56, TrunkingAccessories, "Cable Trunking Reducer", "CABLE TRUNKING REDUCER", TrkRed, false, Gi, 1.2, None, false),
            (57, TrunkingAccessories, "Cable Trunking Reducer", "CABLE TRUNKING REDUCER W/COVER 1.5MM", TrkRed, true, Gi, 1.5, None, false),
            // 58–60: Cable Trunking Equal Tees (Sheets 30–33, SN 51–54)
            (58, TrunkingAccessories, "Cable Trunking Equal Tee", "CABLE TRUNKING EQUAL TEE W/COVER", TrkEtee, true, Gi, 1.2, None, false),
            (59, TrunkingAccessories, "Cable Trunking Equal Tee", "CABLE TRUNKING EQUAL TEE", TrkEtee, false, Gi, 1.2, None, false),
            (60, TrunkingAccessories, "Cable Trunking Equal Tee", "CABLE TRUNKING EQUAL TEE W/COVER 1.5MM", TrkEtee, true, Gi, 1.5, None, false),
            // 61–63: Cable Trunking End Caps (Sheets 34–36, SN 59–62)
            (61, TrunkingAccessories, "Cable Trunking End Cap", "CABLE TRUNKING END CAP", TrkEnd, false, Gi, 1.2, None, false),
            (62, TrunkingAccessories, "Cable Trunking End Cap", "CABLE TRUNKING END CAP 1.5MM", TrkEnd, false, Gi, 1.5, None, false),
            (63, TrunkingAccessories, "Cable Trunking End Cap", "CABLE TRUNKING END CAP HDG", TrkEnd, false, Hdg, 1.5, None, false),
            // 64–65: Unistrut Slotted Channels (Sheet 37, SN 70)
            (64, SupportChannels, "Unistrut Channel", "UNISTRUT SLOTTED CHANNEL", Uni41, false, Gi, 2.0, Some(3000.0), true),
            (65, SupportChannels, "Unistrut Channel", "UNISTRUT SLOTTED CHANNEL 41X41 HDG", Uni41, false, Hdg, 2.0, Some(3000.0), true),
        ];

        let mut entries = Vec::with_capacity(raw_specs.len());
        let mut lookup = HashMap::new();

        for (
            id,
            cat,
            family,
            var_name,
            code,
            with_cover,
            finish,
            thickness,
            default_len,
            has_inset,
        ) in raw_specs
        {
            let entry = ProductVariationEntry {
                variation_id: id,
                category: cat,
                product_family: family.to_string(),
                variation_name: var_name.to_string(),
                catalog_code: code,
                port_count: code.port_count(),
                default_with_cover: with_cover,
                default_finish: finish,
                default_thickness_mm: thickness,
                default_length_mm: default_len,
                has_sectional_inset: has_inset,
            };
            let idx = entries.len();
            lookup.insert(Self::normalize_key(var_name), idx);
            lookup.insert(Self::normalize_key(code.as_str()), idx);
            entries.push(entry);
        }

        Self {
            entries,
            lookup_by_normalized_name: lookup,
        }
    }

    /// Loads additional or overridden variations from `references/product_variations.csv` if present on disk.
    pub fn load_from_csv_path<P: AsRef<Path>>(path: P) -> Result<Self, TaxonomyError> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Self::load_from_csv_str(&content)
    }

    /// Parses `product_variations.csv` content and merges into the 65-variation baseline registry.
    pub fn load_from_csv_str(csv_text: &str) -> Result<Self, TaxonomyError> {
        let mut registry = Self::new_builtin_65();
        let mut rdr = csv::ReaderBuilder::new()
            .flexible(true)
            .trim(csv::Trim::All)
            .from_reader(csv_text.as_bytes());

        for result in rdr.records() {
            let record = result?;
            if record.is_empty() {
                continue;
            }
            let product_col = record.get(1).unwrap_or("").trim();
            let variation_col = record.get(2).unwrap_or(product_col).trim();
            if variation_col.is_empty() {
                continue;
            }
            if let Ok(matched) = registry.resolve_product(variation_col) {
                let cloned = matched.clone();
                let idx = registry.entries.len();
                registry.entries.push(cloned);
                registry
                    .lookup_by_normalized_name
                    .insert(Self::normalize_key(variation_col), idx);
            }
        }
        Ok(registry)
    }

    pub fn entries(&self) -> &[ProductVariationEntry] {
        &self.entries
    }

    /// Resolves a raw customer order product name (e.g. `"CABLE TRAY BEND 90DEG W/COVER"` or `"TRK-END"`)
    /// to its canonical `ProductVariationEntry`.
    pub fn resolve_product(&self, raw_name: &str) -> Result<&ProductVariationEntry, TaxonomyError> {
        let norm = Self::normalize_key(raw_name);
        if let Some(&idx) = self.lookup_by_normalized_name.get(&norm) {
            return Ok(&self.entries[idx]);
        }

        let upper = raw_name.trim().to_uppercase();
        let wants_cover = upper.contains("W/COVER")
            || upper.contains("WITH COVER")
            || upper.contains("W/ COVER");

        let target_code = if upper.contains("UNISTRUT") || upper.contains("UNI-41") || upper.contains("41X41") {
            Some(BonnCatalogCode::Uni41)
        } else if upper.contains("TRUNKING") || upper.starts_with("TRK") {
            if upper.contains("END CAP") || upper.contains("ENDCAP") || upper.contains("TRK-END") {
                Some(BonnCatalogCode::TrkEnd)
            } else if upper.contains("COUPLER") || upper.contains("TRK-CPL") {
                Some(BonnCatalogCode::TrkCpl)
            } else if upper.contains("REDUCER") || upper.contains("TRK-RED") {
                Some(BonnCatalogCode::TrkRed)
            } else if upper.contains("TEE") || upper.contains("TRK-ETEE") {
                Some(BonnCatalogCode::TrkEtee)
            } else {
                Some(BonnCatalogCode::TrkStr)
            }
        } else if upper.contains("TRAY") || upper.starts_with("CT-") {
            if upper.contains("COUPLER") || upper.contains("CT-CPL") {
                Some(BonnCatalogCode::CtCpl)
            } else if upper.contains("INSIDE RISER") || upper.contains("CT-RISER-IN") {
                Some(BonnCatalogCode::CtRiserIn)
            } else if upper.contains("OUTSIDE RISER") || upper.contains("CT-RISER-OUT") {
                Some(BonnCatalogCode::CtRiserOut)
            } else if upper.contains("UNEQUAL TEE") || upper.contains("CT-UTEE") {
                Some(BonnCatalogCode::CtUtee)
            } else if upper.contains("EQUAL TEE") || upper.contains("TEE") || upper.contains("CT-ETEE") {
                Some(BonnCatalogCode::CtEtee)
            } else if upper.contains("CROSS") || upper.contains("CT-ECROSS") {
                Some(BonnCatalogCode::CtEcross)
            } else if upper.contains("REDUCER") || upper.contains("CT-RED") {
                Some(BonnCatalogCode::CtRed)
            } else if upper.contains("45") || upper.contains("CT-B45") {
                Some(BonnCatalogCode::CtB45)
            } else if upper.contains("90") || upper.contains("BEND") || upper.contains("CT-B90") {
                Some(BonnCatalogCode::CtB90)
            } else {
                Some(BonnCatalogCode::CtStr)
            }
        } else {
            None
        };

        if let Some(code) = target_code {
            if let Some(entry) = self
                .entries
                .iter()
                .find(|e| e.catalog_code == code && e.default_with_cover == wants_cover)
                .or_else(|| self.entries.iter().find(|e| e.catalog_code == code))
            {
                return Ok(entry);
            }
        }

        Err(TaxonomyError::UnknownProduct(raw_name.to_string()))
    }

    fn normalize_key(s: &str) -> String {
        s.trim()
            .to_uppercase()
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect()
    }
}