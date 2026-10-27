# AGENTS.md — Instructions for AI Coding Agents in `CADense`

Welcome to the **CADense** repository. This repository is an additive industrial sheet-metal and Cable Management Systems (CMS) automation platform built on top of **SolidCraft (`v0.23.0`, Format 37)**.

Every AI agent (Antigravity IDE Agent, Gemini, Claude, Cursor, Windsurf) operating in this workspace **MUST** follow the rules below.

---

## 1. Rule R1: Strict Additive-Only Compliance (CRITICAL)

1. **Never Modify or Delete Upstream SolidCraft Files**:
   - All existing crates under `crates/geom`, `crates/sketch`, `crates/kernel`, `crates/render`, `crates/doc`, `crates/io`, `crates/engine`, `crates/gpu`, `crates/ui-egui`, `crates/mcp`, and `apps/solidcraft` belong to upstream `tangency-labs/solidcraft`.
   - Do **not** delete, rename, or destructively refactor upstream files. This guarantees zero merge conflicts when running `./scripts/sync-upstream.ps1` for new SolidCraft releases (`v0.23.1`, `v0.24.0`, etc.).
2. **Where All CADense Code Lives**:
   - Place all new functionality inside isolated workspace crates:
     - `crates/cadense-dxf-rigger` (Layer L3/L4): DXF import auto-rigging & parametric sheet-metal template builders.
     - `crates/cadense-dsf` (Layer L3/L4): `.dsf` ("Danish Sheet Fold") container I/O, Port Assignment schema (`P1`..`P4`), `Equal`/`Unequal` constraint solver, and Rigid Feature (`OBROUND`) rules.
     - `crates/cadense-order` (Layer L4): Customer Order CSV parser, derived dimension formulas (`W - 5mm`, `H - 5mm`), and batch model instantiator.
     - `crates/cadense-dds-pdf` (Layer L4): `.dds` ("Danish Data Sheet") collection manager, 3D-to-2D HLR isometric vector projector, automated red dimension callouts, and Bonn Metal Cons. Ind. PDF generator.
     - `crates/cadense-ui` (Layer L5): `egui` panels for Port Assignment, Simultaneous 3D Folded + Live 2D Unfolded Flat Pattern window, Order Queue table, and DDS PDF exporter.
     - `apps/cadense-studio` (Binary): Custom desktop application entry point combining SolidCraft's engine/UI with `crates/cadense-ui`.
   - If a minimal registration hook is ever needed in `Cargo.toml` (workspace members) or an upstream entry point, wrap it clearly inside:
     ```rust
     // === [CADENSE-HOOK START] ===
     // ... additive hook code ...
     // === [CADENSE-HOOK END] ===
     ```

---

## 2. SolidCraft v0.23.0 Architectural Guardrails

1. **Respect Workspace Layering (`cargo xtask layers`)**:
   - `L0`: `crates/geom`
   - `L1`: `crates/sketch`, `crates/kernel`, `crates/render`
   - `L2`: `crates/doc`
   - `L3`: `crates/io`
   - `L4`: `crates/engine`, `crates/cadense-dsf`, `crates/cadense-order`, `crates/cadense-dds-pdf`
   - `L5`: `crates/gpu`, `crates/ui-egui`, `crates/mcp`, `crates/cadense-ui`
   - Lower layers must never depend on higher layers.
2. **Use `crates/engine` Commands as the Stable Bridge**:
   - Drive SolidCraft operations through `crates/engine` (`parameters.set`, `sheet.flange`, `sheet.fold`, `sheet.perforate`, `sheet.flat_pattern`, `sheet.manage_rules`, `sheet.export_bend_table`) rather than coupling to private internals of `crates/doc/src/sheet.rs` (which upstream is refactoring under Issue #6748).
3. **Mitered / Angled Sheet-Metal Bends (SolidCraft Issue #6957)**:
   - In `v0.23.0`, `sheet.fold` rejects flat blanks whose outline crosses the bend zone at an angle (e.g., 45° bends, 90° bends, tees, reducers). Always build angled sheet-metal templates using `sheet.flange` (Edge/Path Flanges) or the `fold rectangular blank -> sheet.unfold -> cut slanted outline -> sheet.refold` sequence.
4. **Preserve Native `.solidcraft` Inside `.dsf` Containers**:
   - Never alter `crates/doc/src/format.rs` (Format 37). Store `.dsf` files as a container bundling a standard `design.solidcraft` file + `dsf_meta.json` so SolidCraft's built-in format migrator upgrades `design.solidcraft` automatically on future releases.

---

## 3. Mandatory Workflow for Applying Task Archives in Antigravity IDE

Whenever the user provides a task `.zip` archive (or asks you to apply a task):
1. Extract the `.zip` archive at the repository root (or run `./scripts/apply-task-zip.ps1 -ZipPath <path-to-zip>`).
2. Run `./scripts/check-additive.ps1` to verify Rule R1 compliance.
3. Run `cargo check --workspace` and `cargo test --workspace` (fixing any minor local compiler warnings/imports if needed).
4. Ensure `CADENSE_STATUS.md` is updated with the completed task details.
5. Stage, commit, and push to `origin/main`:
   ```powershell
   git add .
   git commit -m "feat(cadense): <Task Name and Summary>"
   git push origin main
   ```
