# DXF Analysis & TDS Comparison Report
**Bonn Metal Cons. Ind. — Cable Management Systems (CMS) Sheet Metal Automation**  
**Document Ref**: `CADENSE-ANA-2026-001`  
**Date**: 2026-10-27  
**Workspace File**: [`docs/DXF_ANALYSIS_AND_TDS_COMPARISON.md`](file:///d:/RustProjects/cadense/docs/DXF_ANALYSIS_AND_TDS_COMPARISON.md)

---

## 1. Executive Summary & Survey Scope

We analyzed all **29 production DXF files** provided in `assets/dxf/` across `tray_accessories` (15 files) and `trunking_accessories` (14 files), and compared them systematically against `references/product_variations.csv` and the 37 technical submittal sheets in `references/DATA SHEET SITE TECHNOLOGY (1).pdf`.

By comparing size variations across the same product families (e.g. width scaling `100mm` → `200mm` → `450mm`, height scaling `50mm` → `100mm` → `200mm`), we reverse-engineered the exact parametric rules governing how 2D flat patterns are constructed, where bend lines are situated, how relief slits/V-notches function, and how coupler slots are arrayed.

---

## 2. Master Catalog & DXF Mapping Table

| Category | DXF Filename | Mapped Product (`product_variations.csv`) | TDS Sheet Ref (`DATA SHEET SITE TECHNOLOGY (1).pdf`) | Nominal 3D Size (W × H × T mm) | 2D Model Flat Envelope (W × H mm) | Slot Pattern | Distinct Features |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Tray Acc.** | `45_deg_bend_100x50.dxf` | Tray Accessories > Bend > 45 Deg Bend | Sheet 05, 10 (SN 08, 13) | W=100, H=50, T=2.0 | 324.75 × 237.41 | Obround 20×8 (15 slots) | 45° miter, 63mm end tab |
| **Tray Acc.** | `45_deg_bend_100x100.dxf` | Tray Accessories > Bend > 45 Deg Bend | Sheet 05, 10 (SN 08, 13) | W=100, H=100, T=2.0 | 363.02 × 337.41 | Obround 20×8 (24 slots) | +100mm flat height (+50mm/flange) |
| **Tray Acc.** | `45_deg_bend_450x50.dxf` | Tray Accessories > Bend > 45 Deg Bend | Sheet 05, 10 (SN 08, 13) | W=450, H=50, T=2.0 | 592.63 × 595.24 | Obround 20×8 (31 slots) | Slant edge = 513mm (W+63mm) |
| **Tray Acc.** | `45_deg_inside_riser_100_50.dxf` | Tray Accessories > Inside Riser > 45 Deg Inside Riser | Sheet 06 (SN 09) | W=100, H=50, T=2.0 | 300.00 × 225.00 | Obround 20×8 (40 slots) | Dual 45.0° V-notches for bending |
| **Tray Acc.** | `45_deg_outside_riser_100_50.dxf` | Tray Accessories > Outside Riser > 45 Deg Outside Riser | Sheet 08 (SN 11, 27, 30) | W=100, H=50, T=2.0 | 300.00 × 226.00 | Continuous edge | Flanged outside bend contour |
| **Tray Acc.** | `90_bend_100x50.dxf` / `90_deg_bend_100x50.dxf` | Tray Accessories > Bend > 90 Deg Bend | Sheet 09, 15, 17, 18 (SN 12, 19, 22) | W=100, H=50, T=2.0 | 430.52 × 243.45 | Obround 20×8 (14 slots) | 90° circular/mitered quadrant |
| **Tray Acc.** | `90_deg_bend_100x100.dxf` | Tray Accessories > Bend > 90 Deg Bend | Sheet 09, 15, 17, 18 (SN 12, 19, 22) | W=100, H=100, T=2.0 | 501.23 × 343.45 | Obround 20×8 (24 slots) | +100mm flat height (+50mm/flange) |
| **Tray Acc.** | `90_deg_inside_riser_100x50.dxf` | Tray Accessories > Inside Riser > 90 Deg Inside Riser | Sheet 06 (SN 09) | W=100, H=50, T=2.0 | 226.00 × 400.00 | Slotted riser segments | Segmented riser for 90° bend |
| **Tray Acc.** | `90_deg_outside_riser_100X50.dxf` | Tray Accessories > Outside Riser > 90 Deg Outside Riser | Sheet 08 (SN 11, 27, 30) | W=100, H=50, T=2.0 | 226.00 × 400.00 | Segmented riser contour | Outside 90° profile |
| **Tray Acc.** | `equal_cross_100x50.dxf` | Tray Accessories > Cross > Equal Cross | Sheet 12 (SN 15) | W=100, H=50, T=2.0 | 474.84 × 474.84 | Obround 20×8 (24 slots) | Symmetric 4-way cross |
| **Tray Acc.** | `unequal_cross_100x50x100x50.dxf` | Tray Accessories > Cross > Unequal Cross | Sheet 12 variant | W1=100, W2=50, H=50 | 474.84 × 424.84 | Obround 20×8 (24 slots) | Exactly -50mm in Y dimension |
| **Tray Acc.** | `equal_tee_100x50.dxf` | Tray Accessories > Tee > Equal Tee | Sheet 11, 14 (SN 14, 18, 24, 31) | W=100, H=50, T=2.0 | 474.84 × 350.42 | Obround 20×8 (24 slots) | 3-way T-junction |
| **Tray Acc.** | `unequal_tee_100x50x100x50.dxf` | Tray Accessories > Tee > Unequal Tee | Sheet 07 (SN 10, 16, 21) | W1=100, W2=50, H=50 | 424.84 × 350.42 | Integral boundary | Exactly -50mm in X run width |
| **Tray Acc.** | `reducer_100x50x50.dxf` | Tray Accessories > Reducer > Reducer (straight) | Sheet 13 (SN 17, 32) | W1=100, W2=50, H=50 | 225.37 × 256.27 | Obround 20×8 (19 slots) | Slanted side walls |
| **Trunking** | `45_deg_bend_100x100.dxf` | Trunking Accessories > Bend > 45 Deg Bend | Sheet 05 variant (Trunking) | W=100, H=100, T=1.2 | 248.45 × 320.38 | Corner fillets r=10 | Flanged enclosed trunking bend |
| **Trunking** | `90_deg_bend_100x100.dxf` | Trunking Accessories > Bend > 90 Deg Bend | Sheet 09 variant (Trunking) | W=100, H=100, T=1.2 | 274.45 × 315.37 | Corner fillets r=10 | 90° trunking elbow |
| **Trunking** | `45_deg_inside_riser_100x100.dxf` | Trunking Accessories > Inside Riser > 45 Deg Inside Riser | Sheet 06 variant (Trunking) | W=100, H=100, T=1.2 | 315.34 × 170.66 | Obround 20×8 (8 slots) | Side tabs with slots |
| **Trunking** | `45_deg_inside_riser_100x200.dxf` | Trunking Accessories > Inside Riser > 45 Deg Inside Riser | Sheet 06 variant (Trunking) | W=100, H=200, T=1.2 | 517.51 × 254.30 | Obround 20×8 (8 slots) | Height doubled: flat H +83.64mm |
| **Trunking** | `45_deg_inside_riser_200x200.dxf` | Trunking Accessories > Inside Riser > 45 Deg Inside Riser | Sheet 06 variant (Trunking) | W=200, H=200, T=1.2 | 617.51 × 254.30 | Obround 20×8 (8 slots) | Exactly +100.00mm flat W (617 vs 517) |
| **Trunking** | `45_deg_outside_riser_100x100.dxf`| Trunking Accessories > Outside Riser > 45 Deg Outside Riser | Sheet 08 variant (Trunking) | W=100, H=100, T=1.2 | 309.00 × 179.99 | Obround 20×8 (8 slots) | Inverted V-notches |
| **Trunking** | `90_deg_inside_riser_100x100.dxf` | Trunking Accessories > Inside Riser > 90 Deg Inside Riser | Sheet 06 variant (Trunking) | W=100, H=100, T=1.2 | 317.78 × 227.69 | Obround 20×8 (8 slots) | 90° 2-stage riser notch |
| **Trunking** | `90_deg_outside_riser_100x100.dxf`| Trunking Accessories > Outside Riser > 90 Deg Outside Riser | Sheet 08 variant (Trunking) | W=100, H=100, T=1.2 | 227.81 × 310.00 | Obround 20×8 (8 slots) | 90° outside profile |
| **Trunking** | `equal_cross_100x100.dxf` | Trunking Accessories > Cross > Equal Cross | Sheet 12 variant (Trunking) | W=100, H=100, T=1.2 | 321.45 × 321.45 | Obround 20×8 (16 slots) | 4-port square cross |
| **Trunking** | `unequal_cross_100x50x100x50.dxf` | Trunking Accessories > Cross > Unequal Cross | Sheet 12 variant (Trunking) | W1=100, W2=50, H=100 | 307.30 × 271.45 | Obround 20×8 (12 slots) | Exactly -50mm in Y dimension |
| **Trunking** | `equal_tee_100x100.dxf` | Trunking Accessories > Tee > Equal Tee | Sheet 30–33 (SN 51–54) | W=100, H=100, T=1.2 | 320.03 × 316.02 | Obround 20×8 (12 slots) | Side tabs for trunking cover screws |
| **Trunking** | `unequal_tee_100x100x50x100...dxf`| Trunking Accessories > Tee > Unequal Tee | Sheet 30–33 variant | W1=100, W2=50, H=100 | 716.44 × 330.87 | Obround 20×8 (10 slots) | Extended branch with mounting holes |
| **Trunking** | `reducer_100x100x50x50.dxf` | Trunking Accessories > Reducer > Reducer (straight) | Sheet 27–29 (SN 45–47) | W1=100, W2=50, H=100 | 294.99 × 212.94 | Bolting tabs | Slanted side walls + top cover tabs |

---

## 3. Parametric Scaling Rules Discovered

1. **Flange Height Scaling ($\Delta H$)**:
   - `45_deg_bend_100x50` ($237.41\text{ mm}$ flat H) vs `45_deg_bend_100x100` ($337.41\text{ mm}$ flat H):
     - Flat height increases by exactly **$+100.00\text{ mm}$** ($+50\text{ mm}$ per side flange).
     - Flange edge tab is $63\text{ mm}$ for $H=50$ ($50\text{ mm} + 13\text{ mm}$ return lip/tab), and $113\text{ mm}$ for $H=100$ ($100\text{ mm} + 13\text{ mm}$).
2. **Bed Width Scaling ($\Delta W$)**:
   - `45_deg_inside_riser_100x200` ($517.51\text{ mm}$ flat W) vs `45_deg_inside_riser_200x200` ($617.51\text{ mm}$ flat W):
     - Flat width increases by exactly **$+100.00\text{ mm}$**.
     - Left and right profile wings are **100% rigid and identical**; only the center bed width stretches by $\Delta W$ ($W - 3\text{ mm}$ bend deduction).
   - In 45° bends, the slant line scales according to:
     $$\text{Length}_{\text{slant}} = W + H + 13\text{ mm}$$
     ($100\times 50 \implies 163\text{ mm}$, $100\times 100 \implies 213\text{ mm}$, $450\times 50 \implies 513\text{ mm}$).
3. **Riser Relief Cutlines (V-Notches)**:
   - In all Inside and Outside Risers, the side flanges feature **$45.0^\circ$ V-notches** (two $22.5^\circ$ cut edges converging at the fold line).
   - **Crucial Rule**: These cut lines are intentional sheet metal relief cuts, NOT unclosed boundary errors.

---

## 4. User Confirmed Decisions & Domain Insights (2026-10-27)

1. **Laser Cutting Purpose & Layer Segregation**:
   - DXF files are created for CNC laser cutting and therefore intentionally lack layer segregation or bend lines.
   - Bend lines must be created downstream during SolidCraft modeling operations.
   - Minor millimeter variations exist in real parts to accommodate physical press-brake bending tooling and cover-fixing clearances.
   - Approved **Option A**: CADense auto-segregates processed DXFs into `OUTLINE`, `BEND_UP`, `BEND_DOWN`, `CUT_RELIEF`, and `SLOTS` for downstream 3D operations.
2. **Perforations vs Coupler Holes**:
   - Obround slots serve two distinct functions:
     - **Perforations**: Slots located on the flat base/bed or middle of side walls (used for cable tying, ventilation, and general mounting).
     - **Coupler Holes**: Slots located at the boundary edges of side walls (used as mounting holes for bolting fittings to length items like trays or trunkings).
3. **Coupler Flaps Strategy**:
   - Coupler flaps are standard and fixed on Trunking accessories.
   - For Cable Tray accessories, coupler flaps are an optional feature adding protruding tabs from side wall edges around each port. We will retain the toggle in CADense and address tray flap sizing in a later phase.
4. **Sheet Metal Thickness & Flanging vs Bending**:
   - Regular CMS products range from $0.45\text{ mm}$ to $3.0\text{ mm}$ (with heavy fittings like Baseposts at $4.0 - 6.0\text{ mm}$).
   - Default baseline thickness: **$1.2\text{ mm}$**. Defined in SolidCraft via sheet metal extrusion.
   - Two distinct folding operations:
     - **Flanging**: Forms the boundary stiffening return lips ($11\text{ mm} - 20\text{ mm}$) to make the part robust.
     - **Bending**: Forms the main vertical side walls ($H = 25\text{ mm} - 200\text{ mm}$).
   - **Rigid Features**: In parametric scaling, flanged return lips, coupler holes, and perforations remain **rigid and unscaled** while $W$ and $H$ stretch.
5. **Approved Candidate Products for Step 3**:
   1. `45_deg_bend_100x50.dxf` (Tray 45° Flat Bend)
   2. `45_deg_inside_riser_100x100.dxf` (Trunking 45° Inside Riser with V-notches)
   3. `equal_tee_100x100.dxf` (Trunking Equal Tee with side tabs)

---

## 5. Completed Implementation & Deliverables

1. **Interactive 2D DXF Canvas with Relief Cut Recognition**:
   - Implemented in `apps/cadense-studio/src/landing.rs`.
   - Parses CNC flat contours directly from model space (`code 67 == 0`).
   - Color coded: Cyan boundary (`#38BDF8`), Green perforations & coupler holes (`#10B981`), Orange riser relief V-notches (`#F59E0B`), and Red abnormalities (`#EF4444`).
2. **3D Folded SolidCraft Models (Format 38)**:
   - Generated for all 3 candidate products with baseline thickness $T=1.2\text{ mm}$, side flanges ($H=50/100\text{ mm}$), and return flanging lips ($11\text{--}13\text{ mm}$):
     - `drawings/cms_in_progress/tray_45_deg_bend_100x50.solidcraft`
     - `drawings/cms_in_progress/trunking_45_deg_inside_riser_100x100.solidcraft`
     - `drawings/cms_in_progress/trunking_equal_tee_100x100.solidcraft`
   - Available in Stage 2 with one-click direct launcher into SolidCraft CAD.
3. **Verification**:
   - `cargo check --workspace` and `cargo check -p cadense-studio` pass with 0 errors.
   - `check-additive.ps1` verifies 100% Rule R1 additive-only compliance.
