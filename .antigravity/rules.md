# CADense Antigravity Agent Rules

1. **Rule R1 (Additive-Only Compliance & Upstream Safety)**:
   - Never delete or destructively modify upstream SolidCraft (`tangency-labs/solidcraft`) source files.
   - All CADense features (`.dsf`, `.dds`, Port Assignment, 2D Unfolded Flat Preview, CSV Order Processor, and Vector PDF Technical Data Sheets) MUST reside inside isolated crates (`crates/cadense-*`, `apps/cadense-*`, `scripts/*`).
   - If a hook into upstream `Cargo.toml` or `crates/ui-egui` is required, wrap it inside `// === [CADENSE-HOOK START] ===` and `// === [CADENSE-HOOK END] ===`.

2. **SolidCraft v0.23.1 Sheet Metal Rules**:
   - Drive SolidCraft via `crates/engine` commands (`parameters.set`, `sheet.flange`, `sheet.flat_pattern`, `sheet.manage_rules`, `sheet.perforate`) rather than mutating private structs in `crates/doc/src/sheet.rs`.
   - Store `.dsf` ("Danish Sheet Fold") as a ZIP container containing an unmodified `design.solidcraft` (Format 37+) plus `dsf_meta.json`.
   - Maintain `CADENSE_STATUS.md` after every completed task.
