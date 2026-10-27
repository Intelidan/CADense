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
     - `crates/cadense-dds-pdf` (Layer L4): `.dds` ("Danish Data Sheet") collection manager, 3D-to-2D HLR isometric vector projector, automated red dimension call