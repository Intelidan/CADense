# SolidCraft v0.24.0 Upstream Upgrade & CMS Impact Analysis
**CADense Industrial Sheet Metal & Cable Management Systems (CMS) Automation Platform**  
**Document Ref**: `CADENSE-UPGRADE-2026-002`  
**Date**: 2026-10-27  
**Upstream Target**: `tangency-labs/solidcraft` release `v0.24.0` (commit `5965439a`, Design Format 39)  
**Baseline Version**: `v0.23.1` (commit `586a358e`, Design Format 38)  
**Previous Baseline**: `v0.23.0` (commit `7770f898`, Design Format 37)  

---

## 1. Executive Summary & Release Progression

SolidCraft has completed three rapid, high-impact release cycles:
1. **v0.23.0 (Format 37)**: Introduced Perforate, edge flanges with material outside/tangent bend positions, text parameters from variables, and independent GPU card selection.
2. **v0.23.1 (Format 38)**: Introduced auto-adapting joints that follow part dimensional changes, DXF open/insert outside sketches, and parametric sketch pattern counts.
3. **v0.24.0 (Format 39, Current Release)**: Introduces **132 commits** featuring major breakthroughs for industrial sheet metal, parametric expressions, structural frames/weldments, off-thread timeline evaluation, and real-time event-driven IPC.

This analysis evaluates the exact changes in `v0.24.0`, assesses their direct utility for our Cable Management Systems (CMS) workflow (Cable Trays, Cable Trunking, Unistrut Channels, Perforations, Coupler Flaps), and details the seamless upstream sync procedure while preserving the CADense wrapper intact under **Rule R1 (Zero-Touch Upstream Compliance)**.

---

## 2. Comprehensive Inventory of Upstream SolidCraft v0.24.0 Changes

Across the 132 commits between `v0.23.1` and `v0.24.0`, the upstream team touched **350 files** (+23,281 / -2,207 lines). The key functional clusters are detailed below:

### 2.1 Sheet Metal & Flat Pattern Enhancements
- **Pattern of Cuts in Flat Preserved (`fa961eaa`, PR #6853)**:
  - *Previous Behavior*: In `v0.23.1`, creating a rectangular or circular pattern of an extruded cut on an unfolded flat pattern failed or was refused upon refolding with the error `"this sheet machining cannot be preserved in its flat material"`.
  - *New Behavior*: Patterns of cuts created in the flat pattern are recognized as native sheet metal machining operations. They are fully preserved in the flat export and accurately folded back in 3D. The pattern count and spacing dynamically update when driven by parameters (`count1: N`).
- **Sheet Metal Flat DXF Export Picker (`c482dab4`, PR #6856)**:
  - Flat pattern DXF export now directly invokes native operating system file save dialogs with customizable layer configurations and naming schemes.

### 2.2 Sketcher, Solver & Geometric Dimensioning
- **Direct Model-View Dimension Editing (`3de2f96b`, PR #7080)**:
  - Dimensions displayed on a finished 3D body (`Show Dimensions`) can now be double-clicked directly in the 3D viewport without entering sketch edit mode. Typing a new value or formula updates `parameters.change` and triggers an instant model rebuild.
- **Offset Slot Solver Degree of Freedom Fix (`e2c9f908`, PR #6697)**:
  - *Previous Behavior*: Offsetting a slot (rounded rectangle / obround) in a sketch caused the solver to lock the center arc degrees of freedom, causing any subsequent placement dimensions to fail with conflicting constraint errors.
  - *New Behavior*: The solver now treats offset arcs as following the master source center without redundant arc conditions, allowing full dimensional repositioning.
- **Parametric Expressions & Unit Arithmetic (`3bcf77ee`, PR #6331)**:
  - Numbers and units now evaluate with unified precedence. Previously, expressions like `10 mm / 2 mm` were parsed as `(10 mm / 2) mm = 5 mm²`, causing unit-squared errors. In 0.24.0, dividing dimensions yields dimensionless scalar ratios (`5`), and fractional units (`1/2 in`) evaluate correctly.
- **3D Arc Sketch Command (`2863b805`, PR #5682)**:
  - Native 3-point 3D arc entity creation in 3D sketch mode.
- **Midpoint Line Tool HUD (`bd38bc40`, PR #6401)**:
  - Shows real-time length and angle readouts directly on screen while drawing lines.
- **Dimension Unit Labels (`8b98f6c7`, PR #6131)**:
  - In-sketch dimension annotations explicitly show their physical units (e.g., `100 mm`).

### 2.3 Frames, Weldments & Structural Members
- **Dedicated Weldments Ribbon Tab (`44b0056d`, PR #5262)**:
  - A new top-level `WELDMENTS` ribbon tab joins `MODEL`, `SKETCH`, and `SHEET METAL`.
  - Frame Member tools have been promoted from sub-menus into a first-class workflow with dedicated contextual tabs, configuration profiles, and structural cross-sections.

### 2.4 Multi-Threading & Off-Thread Rebuild Engine
- **Asynchronous Feature Application (`13cd45d1`, PR #3135)**:
  - Feature transactions and geometry regeneration are offloaded from the UI window thread to worker threads.
- **Evaluation Cache State Deduplication (`daecea6e`, PR #6897)**:
  - The 64 MB evaluation cache budget previously charged each cached state for the entire model. Now, each state is only charged for delta bodies/sketches, drastically improving cache hit rates when toggling parameters.
- **Recomputation Ahead (`ffbcdc61`, PR #7002)**:
  - Fast-path results computed during off-thread evaluation are reused when returning to the main thread, eliminating multi-second UI stutters when dragging parameter sliders.

### 2.5 Headless Automation, IPC & Control Channel
- **Pushed `design.changed` Event Subscriptions (`d895fef3`, PR #6986)**:
  - External applications connecting to SolidCraft over IPC/TCP socket can execute `control.subscribe`.
  - SolidCraft automatically pushes a `{"event":"design.changed","params":{"document_id":...,"revision":...}}` payload whenever geometry, parameters, or body visibility change, eliminating the need for client polling loops.
- **Unified Part Properties & BOM Metadata Table (`64ba396c`, PR #4786)**:
  - Standardized `metadata::KEYS` table for part properties (`part_number`, `revision`, `project`, `description`, etc.) with zero unnecessary model re-evaluations when properties are modified.

### 2.6 Native Bug Fixes & Windows Portability
- **Unix-Only Symlink Test Fixed Upstream (`cfb3f52a`, PR #6546)**:
  - Upstream officially applied `#[cfg(unix)]` to `fn slot` and `fn links_only_where_it_may` in `crates/ui-egui/src/cli_link.rs`, resolving the Windows compilation warning without local patches.
- **Top-Level Cut/Join Reach Sub-Components (`0d32821e`, PR #6246)**:
  - Boolean operations now seamlessly penetrate nested sub-assembly bodies.
- **New Themes (`c391ef17`, PR #7342)**:
  - Added "Clay" (warm paper) and "Clay Dark" (charcoal with terracotta accent) themes.

---

## 3. Deep Impact on CMS Sheet Metal & TruFold Workflow

| SolidCraft 0.24.0 Feature | CMS Production Use Case (Bonn Metal Cons. Ind.) | Immediate Advantage |
| :--- | :--- | :--- |
| **Flat Cut Patterns (`fa961eaa`)** | **Cable Tray Perforation Arrays**: Cable trays have dense longitudinal arrays of $20\times 8\text{ mm}$ obround slots. | **Critical Fix**: Enables unfolding the tray flat, cutting 1 slot, applying a rectangular pattern driven by `count: (Length - 50)/25`, and refolding without errors. |
| **Unit Expression Division (`3bcf77ee`)** | **Dynamic Slot Spacing Formulas**: Calculating slot count via formulas like `(W - 20 mm) / 15 mm`. | **Critical Fix**: In 0.23.1 this failed with unit squared errors (`mm²`). In 0.24.0 it computes exact dimensionless counts for parametric rigging. |
| **Offset Slot Solver Fix (`e2c9f908`)** | **Coupler Slot Margins**: Offsetting $20\times 8$ obround slots from side-wall edges ($13\text{ mm}$ margin) and positioning them with dimensions. | **Reliability**: Eliminates false over-constrained sketch errors when parametric dimensions adjust slot positions. |
| **Weldments Ribbon Tab (`44b0056d`)** | **Unistrut Support Channels**: Channel profiles (`UNI_41X41_SLOTTED`, `UNI_41X21`), Cantilever Arms, and Trapeze Hangers. | **Architectural Fit**: Separates structural framing members from folded sheet metal fittings in the UI, improving operator clarity. |
| **Direct Dimension Double-Click (`3de2f96b`)** | **Shop-Floor Dimension Overrides**: Fast manual adjustment of width $W$, height $H$, or return lip without opening sketches. | **Productivity**: Production engineers can visually double-click dimensions on 3D isometric models to verify clearances. |
| **Off-Thread Engine & Cache (`daecea6e`, `ffbcdc61`)** | **Batch Customer Order Instantiation**: Batch generation of 100+ line items from CSV orders (`cadense-order`). | **Speed**: Eliminates UI freezes during automated model generation; cached states rebuild in milliseconds. |
| **Pushed `design.changed` Events (`d895fef3`)** | **CADense Studio Live Sync**: Instant synchronization between SolidCraft CAD and CADense Studio landing hub. | **Real-Time Hook**: CADense Studio can subscribe to SolidCraft and refresh 2D previews and TDS sheets instantaneously. |
| **Design Format 39 (`0b2d3ecd`)** | **Future-Proof Model Storage**: Updating native `.solidcraft` files to latest schema. | **Compatibility**: Seamlessly upgrade our generated candidate models to Format 39. |

---

## 4. CADense Wrapper Compatibility & Rule R1 Compliance Assessment

### 4.1 Zero-Touch Compliance (Rule R1)
Our CADense extensions reside strictly in isolated workspace crates:
- `crates/cadense-dsf`: Serializes `.dsf` containers; completely decoupled from upstream internal structs.
- `crates/cadense-order`: Parses CSV order matrices; pure Rust business logic.
- `crates/cadense-dds-pdf`: Vector TDS submittal generator; independent graphics pipeline.
- `crates/cadense-ui`: Custom egui widgets and order queues.
- `apps/cadense-studio`: Standalone workflow hub application.

### 4.2 Workspace Dependencies Check
- Root `Cargo.toml` workspace dependencies:
  - `egui = "0.36"` and `eframe = "0.36"` remain unchanged in `v0.24.0`.
  - Upstream package version bumps from `0.23.1` to `0.24.0`.
  - Local hook in `crates/ui-egui/src/cli_link.rs` is no longer needed because commit `cfb3f52a` officially implemented the `#[cfg(unix)]` guard upstream.

---

## 5. Implementation Roadmap & Sync Execution Plan

### Phase 1: Pre-Sync Snapshot (Completed)
- Staged all CADense wrapper files and verified working tree.
- Analyzed all 132 commits between `v0.23.1` and `v0.24.0`.

### Phase 2: Upstream Merge via Safety Branch
Execute upstream sync using `./scripts/sync-upstream.ps1 -TargetTag v0.24.0`:
```powershell
# 1. Fetch official upstream tag v0.24.0
git fetch upstream --tags

# 2. Create safety branch sync/v0.24.0
git checkout -B sync/v0.24.0

# 3. Merge v0.24.0 tag
git merge v0.24.0 --no-edit

# 4. Verify Rule R1 additive-only compliance
powershell -ExecutionPolicy Bypass -File .\scripts\check-additive.ps1 -UpstreamRef v0.24.0

# 5. Verify cargo compilation across workspace
cargo check --workspace
cargo check -p cadense-studio
```

### Phase 3: Format 39 Model & Template Update
- Upgrade the generated candidate models in `drawings/cms_in_progress/` and `assets/templates/bonn_metal/` from Format 38 to Format 39.
- Leverage the new sheet metal cut pattern capability (`fa961eaa`) to implement parametric perforation patterns in Cable Tray templates.
