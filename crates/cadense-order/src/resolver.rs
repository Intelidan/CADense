//! Bonn Metal Taxonomy, Dimensional Clearance, Sheet-Metal Rule, and Rigid Slot Resolver.
//!
//! Translates `RawOrderRow` + `ProductVariationRegistry` into a complete `ResolvedOrderSpec`
//! with SolidCraft v0.23.1+ (Format 38) engine commands (`parameters.set`, `sheet.manage_rules`,
//! `sheet.flat_pattern`).

use crate::csv_parser::RawOrderRow;
use crate::taxonomy::{BonnCatalogCode, CmsCategory, CmsFinishType, PortEqualityMode, ProductVariationRegistry, TaxonomyError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResolverError {
    #[error("Taxonomy lookup error on SN {sn}: {source}")]
    Taxonomy {
        sn: String,
        #[source]
        source: TaxonomyError,
    },
    #[error("Invalid dimension on SN {sn}: {reason}")]
    InvalidDimension { sn: String, reason: String },
}

/// Locked obround slot specification that never stretches when port dimensions scale.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObroundSlotSpec {
    /// Callout label for TDS drawing (e.g., `"OBROUND 30X7"`, `"OBROUND 20X8"`, `"OBROUND 50X7"`, `"OBROUND 29X12"`).
    pub callout_label: String,
    pub length_mm: f64,
    pub width_mm: f64,
    /// Discrete slot count per bolting tab or splice plate.
    pub slots_per_feature: u32,
    /// Arrangement pattern description (e.g., `"1x1 per tab"`, `"1x2 vertical per tab"`, `"2x2 splice grid"`).
    pub arrangement: String,
}

/// Records any manufacturing clearance deduction applied between nominal order size and actual sheet-metal size.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClearanceAdjustment {
    pub nominal_width_mm: f64,
    pub actual_width_mm: f64,
    pub width_deduction_mm: f64,
    pub nominal_height_mm: f64,
    pub actual_height_mm: f64,
    pub height_deduction_mm: f64,
    pub rule_explanation: String,
}

/// Coupled cover parameters when an order item specifies `W/COVER`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoupledCoverSpec {
    pub included: bool,
    pub exploded_in_tds_view: bool,
    pub return_lip_height_mm: f64,
    pub cam_lock_hole_count: u32,
    pub thickness_mm: f64,
}

/// Sheet metal thickness, bend radius, and K-factor rule applied via `sheet.manage_rules`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetMetalRuleSpec {
    pub thickness_mm: f64,
    pub inside_bend_radius_mm: f64,
    pub k_factor: f64,
    pub return_flange_mm: Option<f64>,
    pub finish: CmsFinishType,
}

/// SolidCraft `crates/engine` command payload for non-destructive model updates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EngineCommandPayload {
    pub command_name: String,
    pub arguments: BTreeMap<String, serde_json::Value>,
}

/// Fully resolved manufacturing and CAD specification for a single customer order row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedOrderSpec {
    pub row_number: usize,
    pub sn: String,
    pub raw_product_name: String,
    pub canonical_variation_name: String,
    pub category: CmsCategory,
    pub catalog_code: BonnCatalogCode,
    pub dsf_template_path: String,
    pub port_count: usize,
    pub equality_mode: PortEqualityMode,
    /// Nominal port dimensions `(Width, Height)` in mm indexed by 1-based port number.
    pub nominal_ports_mm: BTreeMap<usize, (f64, f64)>,
    /// Actual manufacturing port dimensions `(Actual_W, Actual_H)` after clearance deductions.
    pub actual_ports_mm: BTreeMap<usize, (f64, f64)>,
    pub length_mm: Option<f64>,
    pub qty: u32,
    pub clearance: ClearanceAdjustment,
    pub sheet_rule: SheetMetalRuleSpec,
    pub rigid_slot_rule: ObroundSlotSpec,
    pub cover_spec: CoupledCoverSpec,
    pub has_sectional_inset: bool,
    /// SolidCraft parameter bindings (`#W`, `#W1`..`#W4`, `#H`, `#H1`, `#H2`, `#T`, `#L`, `#SLOT_COUNT`).
    pub parameter_bindings: BTreeMap<String, f64>,
    /// Ordered sequence of SolidCraft engine commands to instantiate this item.
    pub engine_commands: Vec<EngineCommandPayload>,
}

pub struct TaxClearanceResolver;

impl TaxClearanceResolver {
    /// Resolves a `RawOrderRow` against the 65-variation taxonomy and Bonn Metal 37-sheet engineering rules.
    pub fn resolve_row(row: &RawOrderRow, registry: &ProductVariationRegistry) -> Result<ResolvedOrderSpec, ResolverError> {
        let variation = registry.resolve_product(&row.product_name).map_err(|source| ResolverError::Taxonomy { sn: row.sn.clone(), source })?;

        let code = variation.catalog_code;
        let finish = row.finish.as_deref().map(CmsFinishType::from_token).unwrap_or(variation.default_finish);

        let thickness_mm = row.t.unwrap_or(variation.default_thickness_mm);
        if thickness_mm <= 0.0 || thickness_mm > 10.0 {
            return Err(ResolverError::InvalidDimension {
                sn: row.sn.clone(),
                reason: format!("Sheet thickness T={}mm is out of valid CMS range", thickness_mm),
            });
        }

        let with_cover = row.explicit_with_cover.unwrap_or_else(|| {
            let upper = row.product_name.to_uppercase();
            variation.default_with_cover || upper.contains("W/COVER") || upper.contains("WITH COVER")
        });

        // 1. Resolve Nominal Port Widths and Heights
        let (nominal_ports_mm, primary_w, primary_h) = Self::resolve_nominal_ports(row, code);

        // 2. Apply Bonn Metal Clearance Formulas (End Caps, Trunking Couplers, Tray Couplers)
        let clearance = Self::compute_clearance(code, primary_w, primary_h);

        let mut actual_ports_mm = BTreeMap::new();
        for (&port_idx, &(nw, nh)) in &nominal_ports_mm {
            actual_ports_mm.insert(port_idx, ((nw - clearance.width_deduction_mm).max(5.0), (nh - clearance.height_deduction_mm).max(5.0)));
        }

        // 3. Resolve Component Length L (including TRK-CPL height-dependent length rule)
        let length_mm = Self::resolve_length_mm(row, code, primary_h, variation.default_length_mm);

        // 4. Resolve Rigid Obround Slot Specification & Discrete Slot Count Rules
        let rigid_slot_rule = Self::resolve_obround_slot_spec(code, primary_h);

        // 5. Resolve Coupled Cover Specification (Exploded vs Closed, 15mm Tray vs 10mm Trunking Lip)
        let cover_spec = Self::resolve_cover_spec(code, with_cover, thickness_mm);

        // 6. Resolve Sheet Metal Rule (Thickness, K-factor, Bend Radius, Return Flange)
        let k_factor = match finish {
            CmsFinishType::Hdg => 0.44,
            CmsFinishType::Gi => 0.42,
            CmsFinishType::Ss316L => 0.45,
            CmsFinishType::PowderCoated => 0.43,
        };
        let inside_bend_radius_mm = (thickness_mm * 1.0).max(1.2);
        let return_flange_mm = match code {
            BonnCatalogCode::CtStr => Some(11.0),
            BonnCatalogCode::Uni41 => Some(7.5),
            _ => None,
        };

        let sheet_rule = SheetMetalRuleSpec { thickness_mm, inside_bend_radius_mm, k_factor, return_flange_mm, finish };

        // 7. Build SolidCraft Parameter Bindings (`#W`, `#W1`..`#W4`, `#H`, `#H1`, `#H2`, `#T`, `#L`)
        let mut parameter_bindings = BTreeMap::new();
        parameter_bindings.insert("#W".to_string(), clearance.actual_width_mm);
        parameter_bindings.insert("#H".to_string(), clearance.actual_height_mm);
        parameter_bindings.insert("#NOMINAL_W".to_string(), clearance.nominal_width_mm);
        parameter_bindings.insert("#NOMINAL_H".to_string(), clearance.nominal_height_mm);
        parameter_bindings.insert("#T".to_string(), thickness_mm);
        if let Some(len) = length_mm {
            parameter_bindings.insert("#L".to_string(), len);
        }
        for (&port_idx, &(aw, ah)) in &actual_ports_mm {
            parameter_bindings.insert(format!("#W{}", port_idx), aw);
            parameter_bindings.insert(format!("#H{}", port_idx), ah);
        }
        parameter_bindings.insert("#SLOT_LEN".to_string(), rigid_slot_rule.length_mm);
        parameter_bindings.insert("#SLOT_WID".to_string(), rigid_slot_rule.width_mm);
        parameter_bindings.insert("#SLOT_COUNT".to_string(), rigid_slot_rule.slots_per_feature as f64);
        if cover_spec.included {
            parameter_bindings.insert("#COVER_LIP_H".to_string(), cover_spec.return_lip_height_mm);
            parameter_bindings.insert("#COVER_CAM_LOCKS".to_string(), cover_spec.cam_lock_hole_count as f64);
        }

        // 8. Build SolidCraft Engine Commands (`sheet.manage_rules`, `parameters.set`, `sheet.flat_pattern`)
        let engine_commands = Self::build_engine_commands(&parameter_bindings, &sheet_rule, &rigid_slot_rule);

        Ok(ResolvedOrderSpec {
            row_number: row.row_number,
            sn: row.sn.clone(),
            raw_product_name: row.product_name.clone(),
            canonical_variation_name: variation.variation_name.clone(),
            category: variation.category,
            catalog_code: code,
            dsf_template_path: code.dsf_template_path(),
            port_count: code.port_count(),
            equality_mode: code.equality_mode(),
            nominal_ports_mm,
            actual_ports_mm,
            length_mm,
            qty: row.qty,
            clearance,
            sheet_rule,
            rigid_slot_rule,
            cover_spec,
            has_sectional_inset: variation.has_sectional_inset,
            parameter_bindings,
            engine_commands,
        })
    }

    fn resolve_nominal_ports(row: &RawOrderRow, code: BonnCatalogCode) -> (BTreeMap<usize, (f64, f64)>, f64, f64) {
        let mut ports = BTreeMap::new();
        let default_w = match code {
            BonnCatalogCode::Uni41 => 41.0,
            BonnCatalogCode::TrkStr | BonnCatalogCode::TrkCpl | BonnCatalogCode::TrkRed | BonnCatalogCode::TrkEtee | BonnCatalogCode::TrkEnd => 100.0,
            _ => 300.0,
        };
        let default_h = match code {
            BonnCatalogCode::Uni41 => 41.0,
            _ => 50.0,
        };

        let w_base = row.w.or(row.w1).unwrap_or(default_w);
        let h_base = row.h.or(row.h1).unwrap_or(default_h);

        let h1 = row.h1.or(row.h).unwrap_or(h_base);
        let h2 = row.h2.or(row.h).unwrap_or(h1);
        let h3 = row.h3.or(row.h).unwrap_or(h1);
        let h4 = row.h4.or(row.h).unwrap_or(h1);

        match code {
            BonnCatalogCode::Uni41 => {
                ports.insert(1, (row.w.unwrap_or(41.0), row.h.unwrap_or(41.0)));
            }
            BonnCatalogCode::TrkEnd | BonnCatalogCode::CtCpl | BonnCatalogCode::TrkCpl => {
                ports.insert(1, (w_base, h1));
            }
            BonnCatalogCode::CtRed | BonnCatalogCode::TrkRed => {
                let w1 = row.w1.or(row.w).unwrap_or(300.0);
                let w2 = row.w2.unwrap_or((w1 * 0.5).max(50.0));
                ports.insert(1, (w1, h1));
                ports.insert(2, (w2, h2));
            }
            BonnCatalogCode::CtUtee => {
                let w1 = row.w1.or(row.w).unwrap_or(w_base);
                let w2 = row.w2.unwrap_or(300.0);
                let w3 = row.w3.unwrap_or(w1);
                ports.insert(1, (w1, h1));
                ports.insert(2, (w2, h2));
                ports.insert(3, (w3, h3));
            }
            BonnCatalogCode::CtEtee | BonnCatalogCode::TrkEtee => {
                // Even on Equal Tee presets, if the user explicitly overrides W1..W3 or H1..H3, respect them!
                let w1 = row.w1.unwrap_or(w_base);
                let w2 = row.w2.unwrap_or(w_base);
                let w3 = row.w3.unwrap_or(w_base);
                ports.insert(1, (w1, h1));
                ports.insert(2, (w2, h2));
                ports.insert(3, (w3, h3));
            }
            BonnCatalogCode::CtEcross => {
                let w1 = row.w1.unwrap_or(w_base);
                let w2 = row.w2.unwrap_or(w_base);
                let w3 = row.w3.unwrap_or(w_base);
                let w4 = row.w4.unwrap_or(w_base);
                ports.insert(1, (w1, h1));
                ports.insert(2, (w2, h2));
                ports.insert(3, (w3, h3));
                ports.insert(4, (w4, h4));
            }
            _ => {
                let w1 = row.w1.or(row.w).unwrap_or(w_base);
                let w2 = row.w2.unwrap_or(w1);
                ports.insert(1, (w1, h1));
                ports.insert(2, (w2, h2));
            }
        }

        (ports, w_base, h_base)
    }

    fn compute_clearance(code: BonnCatalogCode, nominal_w: f64, nominal_h: f64) -> ClearanceAdjustment {
        match code {
            // Sheets 34–36 (SN 59–62): Cable Trunking End Cap -> Actual_W = W - 5mm, Actual_H = H - 5mm
            BonnCatalogCode::TrkEnd => ClearanceAdjustment {
                nominal_width_mm: nominal_w,
                actual_width_mm: (nominal_w - 5.0).max(5.0),
                width_deduction_mm: 5.0,
                nominal_height_mm: nominal_h,
                actual_height_mm: (nominal_h - 5.0).max(5.0),
                height_deduction_mm: 5.0,
                rule_explanation: "Bonn Metal TRK-END internal U-cap fit clearance: Actual_W = W - 5mm, Actual_H = H - 5mm".to_string(),
            },
            // Sheets 24–26 (SN 41–44): Cable Trunking Coupler -> Actual_H = H - 5mm (45, 70, 145mm)
            BonnCatalogCode::TrkCpl => ClearanceAdjustment {
                nominal_width_mm: nominal_w,
                actual_width_mm: nominal_w,
                width_deduction_mm: 0.0,
                nominal_height_mm: nominal_h,
                actual_height_mm: (nominal_h - 5.0).max(5.0),
                height_deduction_mm: 5.0,
                rule_explanation: "Bonn Metal TRK-CPL splice height clearance: Actual_H = H - 5mm (45mm/70mm/145mm)".to_string(),
            },
            // Sheets 19–20 (SN 33–34): Cable Tray Straight Coupler -> Actual_H = H - 5mm (45mm for H=50, 70mm for H=75)
            BonnCatalogCode::CtCpl => ClearanceAdjustment {
                nominal_width_mm: nominal_w,
                actual_width_mm: nominal_w,
                width_deduction_mm: 0.0,
                nominal_height_mm: nominal_h,
                actual_height_mm: (nominal_h - 5.0).max(5.0),
                height_deduction_mm: 5.0,
                rule_explanation: "Bonn Metal CT-CPL splice plate clearance: Actual_H = H - 5mm (70mm for H=75, 45mm for H=50)".to_string(),
            },
            _ => ClearanceAdjustment {
                nominal_width_mm: nominal_w,
                actual_width_mm: nominal_w,
                width_deduction_mm: 0.0,
                nominal_height_mm: nominal_h,
                actual_height_mm: nominal_h,
                height_deduction_mm: 0.0,
                rule_explanation: "Nominal port dimensions applied directly (0mm clearance deduction)".to_string(),
            },
        }
    }

    fn resolve_length_mm(row: &RawOrderRow, code: BonnCatalogCode, nominal_h: f64, default_length: Option<f64>) -> Option<f64> {
        if let Some(explicit_l) = row.l {
            return Some(explicit_l);
        }
        match code {
            // Sheets 24–26: TRK-CPL length is 50mm when H <= 50mm, and 70mm when H >= 75mm
            BonnCatalogCode::TrkCpl => {
                if nominal_h <= 50.0 {
                    Some(50.0)
                } else {
                    Some(70.0)
                }
            }
            // Sheets 19–20: CT-CPL splice length is fixed at 180mm
            BonnCatalogCode::CtCpl => Some(180.0),
            _ => default_length,
        }
    }

    fn resolve_obround_slot_spec(code: BonnCatalogCode, nominal_h: f64) -> ObroundSlotSpec {
        match code {
            // Sheets 1–4: Straight Cable Tray -> OBROUND 30X7 bed perforations
            BonnCatalogCode::CtStr => ObroundSlotSpec {
                callout_label: "OBROUND 30X7".to_string(),
                length_mm: 30.0,
                width_mm: 7.0,
                slots_per_feature: 12,
                arrangement: "Perforated tray bed array + side coupler slots".to_string(),
            },
            // Sheets 19–20: Cable Tray Straight Coupler -> H=75 uses 6x OBROUND 50X7; H=50 uses 6x OBROUND 30X7
            BonnCatalogCode::CtCpl => {
                if nominal_h >= 75.0 {
                    ObroundSlotSpec {
                        callout_label: "OBROUND 50X7".to_string(),
                        length_mm: 50.0,
                        width_mm: 7.0,
                        slots_per_feature: 6,
                        arrangement: "6x (2x3) splice plate pattern for H>=75mm".to_string(),
                    }
                } else {
                    ObroundSlotSpec {
                        callout_label: "OBROUND 30X7".to_string(),
                        length_mm: 30.0,
                        width_mm: 7.0,
                        slots_per_feature: 6,
                        arrangement: "6x (2x3) splice plate pattern for H<=50mm".to_string(),
                    }
                }
            }
            // Sheets 24–26: Cable Trunking Coupler -> 2 slots (H<=75mm) or 4 slots 2x2 (H>=100mm) of OBROUND 20X8
            BonnCatalogCode::TrkCpl => {
                if nominal_h >= 100.0 {
                    ObroundSlotSpec {
                        callout_label: "OBROUND 20X8".to_string(),
                        length_mm: 20.0,
                        width_mm: 8.0,
                        slots_per_feature: 4,
                        arrangement: "4 slots (2x2 grid) for H>=100mm".to_string(),
                    }
                } else {
                    ObroundSlotSpec {
                        callout_label: "OBROUND 20X8".to_string(),
                        length_mm: 20.0,
                        width_mm: 8.0,
                        slots_per_feature: 2,
                        arrangement: "2 horizontal slots (1x2) for H<=75mm".to_string(),
                    }
                }
            }
            // Sheets 27–33: Trunking Reducer & Equal Tee -> 1 slot/tab (H<=75mm) or 2 slots/tab (H>=100mm) of OBROUND 20X8
            BonnCatalogCode::TrkRed | BonnCatalogCode::TrkEtee => {
                if nominal_h >= 100.0 {
                    ObroundSlotSpec {
                        callout_label: "OBROUND 20X8".to_string(),
                        length_mm: 20.0,
                        width_mm: 8.0,
                        slots_per_feature: 2,
                        arrangement: "2 slots per side bolting tab (H>=100mm)".to_string(),
                    }
                } else {
                    ObroundSlotSpec {
                        callout_label: "OBROUND 20X8".to_string(),
                        length_mm: 20.0,
                        width_mm: 8.0,
                        slots_per_feature: 1,
                        arrangement: "1 slot per side bolting tab (H<=75mm)".to_string(),
                    }
                }
            }
            // Sheets 34–36: Cable Trunking End Cap -> 2x OBROUND 20X8 per side wing
            BonnCatalogCode::TrkEnd => ObroundSlotSpec {
                callout_label: "OBROUND 20X8".to_string(),
                length_mm: 20.0,
                width_mm: 8.0,
                slots_per_feature: 2,
                arrangement: "2x OBROUND 20X8 per U-cap side wing".to_string(),
            },
            // Sheets 21–23: Cable Trunking Straight -> unperforated body with cam-lock holes on cover
            BonnCatalogCode::TrkStr => ObroundSlotSpec {
                callout_label: "CAM-LOCK DIA 8".to_string(),
                length_mm: 8.0,
                width_mm: 8.0,
                slots_per_feature: 4,
                arrangement: "Unperforated trunking body with cover cam-lock holes".to_string(),
            },
            // Sheet 37: Unistrut 41x41 Slotted Channel -> OBROUND 29X12
            BonnCatalogCode::Uni41 => ObroundSlotSpec {
                callout_label: "OBROUND 29X12".to_string(),
                length_mm: 29.0,
                width_mm: 12.0,
                slots_per_feature: 60,
                arrangement: "Continuous centerline OBROUND 29X12 on 50mm pitch".to_string(),
            },
            // All Cable Tray Bends, Risers, Tees, Crosses, Reducers -> rigid OBROUND 20X8 coupler slots
            _ => ObroundSlotSpec {
                callout_label: "OBROUND 20X8".to_string(),
                length_mm: 20.0,
                width_mm: 8.0,
                slots_per_feature: 4,
                arrangement: "4x rigid OBROUND 20X8 coupler slots per port flange".to_string(),
            },
        }
    }

    fn resolve_cover_spec(code: BonnCatalogCode, with_cover: bool, body_thickness_mm: f64) -> CoupledCoverSpec {
        if !with_cover {
            return CoupledCoverSpec {
                included: false,
                exploded_in_tds_view: false,
                return_lip_height_mm: 0.0,
                cam_lock_hole_count: 0,
                thickness_mm: 0.0,
            };
        }

        match code {
            // Sheets 21–23: Cable Trunking Straight W/Cover -> exploded cover, 10mm lip, cam-locks
            BonnCatalogCode::TrkStr => CoupledCoverSpec {
                included: true,
                exploded_in_tds_view: true,
                return_lip_height_mm: 10.0,
                cam_lock_hole_count: 4,
                thickness_mm: body_thickness_mm,
            },
            // Sheets 27–33: Cable Trunking Reducer & Equal Tee W/Cover -> closed cover with 2 cam-locks
            BonnCatalogCode::TrkRed | BonnCatalogCode::TrkEtee => CoupledCoverSpec {
                included: true,
                exploded_in_tds_view: false,
                return_lip_height_mm: 10.0,
                cam_lock_hole_count: 2,
                thickness_mm: body_thickness_mm,
            },
            // Sheets 1–4, 9, 15–18: Cable Tray Straight & 90-Deg Bend W/Cover -> exploded cover, 15mm lip
            _ => CoupledCoverSpec {
                included: true,
                exploded_in_tds_view: true,
                return_lip_height_mm: 15.0,
                cam_lock_hole_count: 0,
                thickness_mm: body_thickness_mm.min(1.5),
            },
        }
    }

    fn build_engine_commands(
        params: &BTreeMap<String, f64>,
        sheet_rule: &SheetMetalRuleSpec,
        slot_rule: &ObroundSlotSpec,
    ) -> Vec<EngineCommandPayload> {
        let mut commands = Vec::with_capacity(3);

        // 1. `sheet.manage_rules`: update thickness, bend radius, and K-factor
        let mut rule_args = BTreeMap::new();
        rule_args.insert("thickness_mm".to_string(), serde_json::json!(sheet_rule.thickness_mm));
        rule_args.insert("inside_bend_radius_mm".to_string(), serde_json::json!(sheet_rule.inside_bend_radius_mm));
        rule_args.insert("k_factor".to_string(), serde_json::json!(sheet_rule.k_factor));
        rule_args.insert("finish".to_string(), serde_json::json!(sheet_rule.finish.code()));
        commands.push(EngineCommandPayload { command_name: "sheet.manage_rules".to_string(), arguments: rule_args });

        // 2. `parameters.set`: apply port widths, heights, clearance-adjusted values, and rigid slot locks
        let mut param_args = BTreeMap::new();
        for (k, v) in params {
            param_args.insert(k.clone(), serde_json::json!(v));
        }
        param_args.insert("rigid_slot_callout".to_string(), serde_json::json!(slot_rule.callout_label));
        commands.push(EngineCommandPayload { command_name: "parameters.set".to_string(), arguments: param_args });

        // 3. `sheet.flat_pattern`: compute non-destructive 2D unfolded flat blank for dual-viewport preview
        let mut flat_args = BTreeMap::new();
        flat_args.insert("mutate_timeline".to_string(), serde_json::json!(false));
        flat_args.insert("lock_rigid_slots".to_string(), serde_json::json!(true));
        commands.push(EngineCommandPayload { command_name: "sheet.flat_pattern".to_string(), arguments: flat_args });

        commands
    }
}
