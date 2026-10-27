use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaterialFinish {
    HDG,
    GI,
    SS316L,
}

impl MaterialFinish {
    pub fn note_text(&self) -> &'static str {
        match self {
            MaterialFinish::HDG => "NOTE:- MATERIAL FINISH HDG",
            MaterialFinish::GI => "NOTE:- MATERIAL FINISH GI",
            MaterialFinish::SS316L => "NOTE:- MATERIAL FINISH SS316L",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CmsCategory {
    CableTray,
    CableTrunking,
    UnistrutChannel,
    AccessoryCouplerOrEndCap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortEqualityMode {
    AllEqual,
    SymmetricOpposite,
    Unequal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CoverStyle {
    None,
    Exploded {
        lip_height_mm: f64,
        exploded_z_offset_mm: f64,
    },
    ClosedWithCamLocks {
        lip_height_mm: f64,
        cam_lock_count: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortDefinition {
    pub id: String,
    pub label: String,
    pub width_param: String,
    pub height_param: String,
    pub default_width_mm: f64,
    pub default_height_mm: f64,
    pub anchor_faces: Vec<String>,
    pub normal_dir: [f64; 3],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SlotCountCondition {
    Fixed(u32),
    Threshold {
        param_name: String,
        min_value_mm: f64,
        count_if_ge: u32,
        count_if_lt: u32,
    },
    PitchArray {
        span_param: String,
        pitch_mm: f64,
    },
}

impl SlotCountCondition {
    pub fn evaluate(&self, params: &BTreeMap<String, f64>) -> u32 {
        match self {
            SlotCountCondition::Fixed(n) => *n,
            SlotCountCondition::Threshold {
                param_name,
                min_value_mm,
                count_if_ge,
                count_if_lt,
            } => {
                let val = params.get(param_name).copied().unwrap_or(0.0);
                if val >= *min_value_mm {
                    *count_if_ge
                } else {
                    *count_if_lt
                }
            }
            SlotCountCondition::PitchArray {
                span_param,
                pitch_mm,
            } => {
                let span = params.get(span_param).copied().unwrap_or(3000.0);
                if *pitch_mm <= 1e-3 {
                    1
                } else {
                    ((span / *pitch_mm).floor() as u32).max(1)
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RigidFeatureRule {
    pub id: String,
    pub callout_label: String,
    pub locked_length_mm: f64,
    pub locked_width_mm: f64,
    pub locked_param_names: [String; 2],
    pub count_rule: SlotCountCondition,
    pub count_param_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DerivedFormula {
    pub target_param: String,
    pub source_param: String,
    pub multiplier: f64,
    pub offset_mm: f64,
}

impl DerivedFormula {
    pub fn evaluate(&self, params: &BTreeMap<String, f64>) -> Option<f64> {
        params
            .get(&self.source_param)
            .map(|v| v * self.multiplier + self.offset_mm)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TdsViewConfig {
    pub camera_iso_angles_deg: [f64; 2],
    pub include_sectional_view: bool,
    pub inside_return_flange_mm: Option<f64>,
    pub bend_angle_callout_deg: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DsfMeta {
    pub dsf_version: String,
    pub solidcraft_min_format: u32,
    pub product_code: String,
    pub product_title: String,
    pub category: CmsCategory,
    pub default_finish: MaterialFinish,
    pub equality_mode: PortEqualityMode,
    pub cover_style: CoverStyle,
    pub default_thickness_mm: f64,
    pub default_length_mm: Option<f64>,
    pub ports: Vec<PortDefinition>,
    pub rigid_features: Vec<RigidFeatureRule>,
    pub derived_formulas: Vec<DerivedFormula>,
    pub tds_view_config: TdsViewConfig,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PortDimensionOverrides {
    pub values_mm: BTreeMap<String, f64>,
    pub finish_override: Option<MaterialFinish>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolidCraftCommand {
    pub command: String,
    pub args: serde_json::Value,
}

impl DsfMeta {
    pub fn resolve_parameters(
        &self,
        overrides: &PortDimensionOverrides,
    ) -> BTreeMap<String, f64> {
        let mut resolved = BTreeMap::new();
        resolved.insert("T".to_string(), self.default_thickness_mm);
        if let Some(len) = self.default_length_mm {
            resolved.insert("L".to_string(), len);
        }
        for port in &self.ports {
            resolved.insert(port.width_param.clone(), port.default_width_mm);
            resolved.insert(port.height_param.clone(), port.default_height_mm);
        }
        for (k, v) in &overrides.values_mm {
            resolved.insert(k.clone(), *v);
        }
        match self.equality_mode {
            PortEqualityMode::AllEqual => {
                if let Some(first) = self.ports.first() {
                    let master_w = overrides
                        .values_mm
                        .get("W")
                        .or_else(|| overrides.values_mm.get(&first.width_param))
                        .copied()
                        .unwrap_or(first.default_width_mm);
                    let master_h = overrides
                        .values_mm
                        .get("H")
                        .or_else(|| overrides.values_mm.get(&first.height_param))
                        .copied()
                        .unwrap_or(first.default_height_mm);

                    resolved.insert("W".to_string(), master_w);
                    resolved.insert("H".to_string(), master_h);
                    for port in &self.ports {
                        resolved.insert(port.width_param.clone(), master_w);
                        resolved.insert(port.height_param.clone(), master_h);
                    }
                }
            }
            PortEqualityMode::SymmetricOpposite => {
                if let Some(w1) = resolved.get("W1").copied() {
                    if !overrides.values_mm.contains_key("W3") {
                        resolved.insert("W3".to_string(), w1);
                    }
                }
            }
            PortEqualityMode::Unequal => {}
        }
        match &self.cover_style {
            CoverStyle::None => {
                resolved.insert("Has_Cover".to_string(), 0.0);
            }
            CoverStyle::Exploded {
                lip_height_mm,
                exploded_z_offset_mm,
            } => {
                resolved.insert("Has_Cover".to_string(), 1.0);
                resolved.insert("Cover_Lip_H".to_string(), *lip_height_mm);
                resolved.insert("Cover_Exploded_Z".to_string(), *exploded_z_offset_mm);
            }
            CoverStyle::ClosedWithCamLocks {
                lip_height_mm,
                cam_lock_count,
            } => {
                resolved.insert("Has_Cover".to_string(), 1.0);
                resolved.insert("Cover_Lip_H".to_string(), *lip_height_mm);
                resolved.insert("Cam_Lock_Count".to_string(), *cam_lock_count as f64);
            }
        }
        for formula in &self.derived_formulas {
            if let Some(val) = formula.evaluate(&resolved) {
                resolved.insert(formula.target_param.clone(), val);
            }
        }
        for rigid in &self.rigid_features {
            resolved.insert(rigid.locked_param_names[0].clone(), rigid.locked_length_mm);
            resolved.insert(rigid.locked_param_names[1].clone(), rigid.locked_width_mm);
            let count = rigid.count_rule.evaluate(&resolved);
            resolved.insert(rigid.count_param_name.clone(), count as f64);
        }
        resolved
    }

    pub fn build_solidcraft_commands(
        &self,
        overrides: &PortDimensionOverrides,
    ) -> Vec<SolidCraftCommand> {
        let resolved = self.resolve_parameters(overrides);
        let mut cmds = Vec::new();
        let thickness = resolved.get("T").copied().unwrap_or(self.default_thickness_mm);
        let finish = overrides.finish_override.unwrap_or(self.default_finish);
        let k_factor = match finish {
            MaterialFinish::HDG => 0.44,
            MaterialFinish::GI => 0.42,
            MaterialFinish::SS316L => 0.45,
        };
        cmds.push(SolidCraftCommand {
            command: "sheet.manage_rules".to_string(),
            args: serde_json::json!({
                "thickness": format!("{} mm", thickness),
                "bend_radius": format!("{} mm", thickness),
                "k_factor": k_factor,
            }),
        });
        for (name, val) in &resolved {
            let expression = if name.ends_with("_Count") || name == "Has_Cover" {
                format!("{}", *val as i64)
            } else {
                format!("{:.2} mm", val)
            };
            cmds.push(SolidCraftCommand {
                command: "parameters.set".to_string(),
                args: serde_json::json!({
                    "name": name,
                    "expression": expression,
                }),
            });
        }
        cmds.push(SolidCraftCommand {
            command: "sheet.flat_pattern".to_string(),
            args: serde_json::json!({
                "include_perforations": true,
                "include_bend_lines": true,
            }),
        });
        cmds
    }
}
