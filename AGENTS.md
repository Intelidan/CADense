# AGENTS.md — Instructions for AI Coding Agents in `CADense`

Welcome to the **CADense** repository. This repository is an additive industrial sheet-metal and Cable Management Systems (CMS) automation platform built on top of **SolidCraft (`v0.26.1`, Format 43)**.

Every AI agent (Antigravity IDE Agent, Gemini, Claude, Cursor, Windsurf) operating in this workspace **MUST** follow the rules below.

---

## 1. Rule R1: Strict Additive-Only Compliance (CRITICAL)

1. **Never Modify or Delete Upstream SolidCraft Files Destructively**:
   - All existing crates under `crates/geom`, `crates/sketch`, `crates/kernel`, `crates/render`, `crates/doc`, `crates/io`, `crates/engine`, `crates/gpu`, `crates/ui-egui`, `crates/mcp`, and `apps/solidcraft` belong to upstream `tangency-labs/solidcraft`.
   - Do **not** delete, rename, or destructively refactor upstream files. Any UI integration follows the **99% Isolated Sub-Crate / 1% Hook Injection** model with designated `[CADENSE-HOOK]` markers recorded in `docs/SOLID_CRAFT_CMS_HOOK_REGISTRY.md`.
2. **Where All CADense & CMS truFold Code Lives**:
   - Place all functionality inside isolated workspace crates:
     - `crates/solidcraft_cms`: CMS truFold pipeline, DXF perimeter tracing, port classifier, relaxed B-Rep folding solver, dual-ribbon frontend, viewport raycasting overlay, and automated CAM batch exporter.
     - `crates/cadense-dsf` (Layer L3/L4): `.dsf` ("Danish Sheet Fold") container I/O, Port Assignment schema (`P1`..`P4`), `Equal`/`Unequal` constraint solver, and Rigid Feature (`OBROUND`) rules.
     - `crates/cadense-order` (Layer L4): Customer Order CSV parser, derived dimension formulas (`W - 5mm`, `H - 5mm`), and batch model instantiator.
     - `crates/cadense-dds-pdf` (Layer L4): `.dds` ("Danish Data Sheet") collection manager, 3D-to-2D HLR isometric vector projector, automated red dimension callouts.
     - `crates/cadense-ui` (Layer L4): Custom widgets, Order queue panel, and SolidCraft UI hooks.
     - `apps/cadense-studio`: Standalone CADense Studio 4-stage workflow pipeline hub.

---

## 2. Upstream SolidCraft: Core Architectural Rules

SolidCraft is a clean-room, pure-Rust parametric 3D CAD app (sketch solver, feature timeline, B-rep kernel, an egui UI that agents drive).

### Hard Rules
- **Independent**: Never copy another product's code, icons, text or templates; never drive or take data from commercial CAD (`skills/solidcraft-ground-rules/SKILL.md`).
- **No GPL/LGPL/AGPL code**: Not even reading it (OpenCASCADE, FreeCAD, SolveSpace, CadQuery, LibreCAD, QCAD). Enforce with `cargo xtask licences --check`.
- **Never crash**: No `unwrap`/`expect`/`panic`/`unsafe` in non-test code; errors are `Result`, input is hostile (`skills/solidcraft-crash-safety/SKILL.md`).
- **Everything is a command**: Programmatic calls never open dialogs (`skills/solidcraft-commands/SKILL.md`).
- **Layering**: `geom` -> `sketch`/`kernel`/`render` -> `doc` -> `io` -> `engine` -> `ui-egui` -> `apps`. Only `crates/kernel` uses truck.
- **Design Format**: Current design format is **Format 43** (SolidCraft v0.26.1).

### Working on X: Read the Skill First
| Working on | Read |
|---|---|
| A PR, a claim, review, CI or evidence | `skills/solidcraft-contributing/SKILL.md` |
| Licences, assets, clean-room, the name | `skills/solidcraft-ground-rules/SKILL.md` |
| Non-test Rust, hostile input, a crash report | `skills/solidcraft-crash-safety/SKILL.md` |
| A command, a crate boundary | `skills/solidcraft-commands/SKILL.md` |
| A dialog, panel, camera, icon, colour | `skills/solidcraft-ui/SKILL.md` |
| Labels, messages, translations | `skills/solidcraft-ui-text/SKILL.md` |
| Saving something new in a design, import, export | `skills/solidcraft-file-formats/SKILL.md` |
| Preferences, a changed default | `skills/solidcraft-preferences/SKILL.md` |
| Writing or running tests | `skills/solidcraft-tests/SKILL.md` |
| Slow work, the window thread, wasm | `skills/solidcraft-performance/SKILL.md` |
| Releases, installers, packages | `skills/solidcraft-packaging/SKILL.md` |
| Running the app, control channel, MCP tools | `skills/solidcraft-agent-tools/SKILL.md` |
| Building a part over MCP | `skills/solidcraft-modeling/SKILL.md` |
| Changing an existing design | `skills/solidcraft-editing/SKILL.md` |
| A bug report, a crash, a wrong result | `skills/solidcraft-reporting/SKILL.md` |
| Checking, porting, designs in git | `skills/solidcraft-checking/SKILL.md` |
| Assemblies, joints, motion | `skills/solidcraft-mechanisms/SKILL.md` |
