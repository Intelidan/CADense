# CADense CMS Architecture & Bonn Metal Cons. Ind. Technical Specification

## 1. Executive Summary
This document defines the 4-phase engineering architecture for automating **Cable Management Systems (CMS)** design, port-based parametric scaling, customer CSV order processing, and Technical Data Sheet (TDS) PDF generation in **CADense** (powered by **SolidCraft v0.23.0**).

---

## 2. Detailed 4-Phase Architecture

### Phase 1: Parametric Sheet-Metal Master Models (`.solidcraft`)
- Each CMS product family is modeled as a parametric sheet-metal part in `.solidcraft` Format 37+.
- Base plates and side walls are driven by named parameters (`#W`, `#W1`, `#W2`, `#W3`, `#W4`, `#H`, `#H1`, `#H2`, `#T`, `#L`) and sheet-metal rules (`BendRadius`, `KFactor`).
- **Dual Input Support**:
  1. **3D-First Master Templates**: Built with parametric sketches, `sheet.flange` (including 11mm inside return flanges and 10mm/15mm cover lips), and `sheet.perforate` (`OBROUND 30X7`). Unfolded laser DXFs are generated automatically via `sheet.flat_pattern`.
  2. **2D Unfolded DXF Auto-Rigging (`crates/cadense-dxf-rigger`)**: Imports existing flat shop-floor DXFs, detects standard obround slots (`20x8`, `30x7`, `50x7`, `29x12`), locks their dimensions as rigid parametric stamps, and converts bend lines into fold/flange features.

### Phase 2: Port Assignment, Dual Viewport & `.dsf` ("Danish Sheet Fold")
- **`.dsf` Container Format**:
  - A ZIP container with extension `.dsf` containing:
    - `design.solidcraft`: Unmodified native SolidCraft document.
    - `dsf_meta.json`: Port definitions, equality rules, coupled cover settings, rigid slot rules, and TDS camera/dimensioning metadata.
- **Port Assignment Rules**:
  - Open ends of a product are tagged as Ports (`P1`..`P4`).
  - Each Port binds `width` and `height` to underlying SolidCraft parameters and topological face tags (`sheet:Body1:...`).
  - **Equality Modes**:
    - `PortEquality::AllEqual`: Editing `WIDTH` updates all ports simultaneously (e.g., `CABLE TRAY EQUAL TEE`, `CABLE TRAY EQUAL CROSS`, `CABLE TRUNKING EQUAL TEE W/COVER`).
    - `PortEquality::SymmetricRun`: `W1 == W3` on the main run while branch `W2` varies independently (e.g., `CABLE TRAY UNEQUAL TEE` Sheet 7 where `W1=W3=900mm, W2=300mm`).
    - `PortEquality::Independent`: Every port width/height is independently editable.
- **Simultaneous 3D Folded + 2D Unfolded Window**:
  - The main WGPU viewport displays the 3D folded model.
  - A dockable `egui::Window` calls `sheet.flat_pattern` in the background and renders the live 2D unfolded flat pattern (`OUTLINE`, `CUT`, `BEND_UP`, `BEND_DOWN`, `PERFORATION` layers) with interactive pan/zoom.

### Phase 3: Customer Order CSV Batch Processor (`crates/cadense-order`)
- Reads customer order CSV files containing columns:
  `SN, Product_Name, W, W1, W2, W3, H, H1, H2, T, L, QTY, Finish, With_Cover`
- Applies **Derived Clearance Formulas**:
  - Cable Tray Straight Coupler (`CT-CPL`): `Actual_H = H - 5mm`, `Slot_Type = if(H >= 75, "50X7", "30X7")`.
  - Cable Trunking Coupler (`TRK-CPL`): `Actual_H = H - 5mm`, `Actual_L = if(H == 50, 50, 70)`, `Slot_Count = if(H >= 150, 4, 2)`.
  - Cable Trunking End Cap (`TRK-END`): `Actual_W = W - 5mm`, `Actual_H = H - 5mm`.
  - Cable Trunking Tee & Reducer Flange Tabs (`TRK-ETEE`, `TRK-RED`): `Slots_Per_Tab = if(H >= 100, 2, 1)`.

### Phase 4: `.dds` ("Danish Data Sheet") & Vector TDS PDF Generator (`crates/cadense-dds-pdf`)
- Packages all instantiated `.dsf` items for a customer order into a `.dds` project file.
- Generates crisp vector PDF pages matching **Bonn Metal Cons. Ind.**:
  - Hidden-Line Removed (HLR) isometric vector wireframe projection.
  - Automated red extension lines, arrows, and angled dimension labels (`WIDTH`, `WIDTH-1`, `HEIGHT`, `LENGTH`, `45°`, `OBROUND 20X8`).
  - Top-left `SECTIONAL VIEW` inset for Straight Trays, Trunking, and Unistrut channels.
  - Supports **1 Page Per Product Row** OR **Grouped Submittal Mode** (grouping matching product rows into the `SN | DIMENSION TABLE | QTY` table on a single page).
