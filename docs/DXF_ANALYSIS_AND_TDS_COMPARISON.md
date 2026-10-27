# DXF Analysis & TDS Comparison Report
**Bonn Metal Cons. Ind. — Cable Management Systems (CMS) Sheet Metal Automation**  
**Document Ref**: `CADENSE-ANA-2026-001`  
**Date**: 2026-10-27  
**Reference Documents**:
- `references/product_variations.csv` (65 Product Variations)
- `references/DATA SHEET SITE TECHNOLOGY (1).pdf` (37 Submittal Sheets, Bonn Metal Cons. Ind.)
- `assets/dxf/tray_accessories/` (15 production DXF files)
- `assets/dxf/trunking_accessories/` (14 production DXF files)

---

## 1. Executive Summary & Survey Scope

To transition from manually drafted drawings to full parametric 3D sheet metal bending in SolidCraft (`v0.23.1+`, Format 38), we performed a deep geometric, trigonometric, and dimensional analysis across all **29 production DXF files** provided in `assets/dxf/`. 

By comparing size variations across the same product families (e.g. width scaling `100mm` → `200mm` → `450mm`, height scaling `50mm` → `100mm` → `200mm`), we reverse-engineered the exact parametric rules governing how 2D flat patterns are constructed, where bend lines are situated, how relief slits/V-notches function, and how coupler slots are arrayed.

These 2D flat patterns were then cross-referenced against the 37 technical submittal sheets in `references/DATA SHEET SITE TECHNOLOGY (1).pdf` (Site Technology Integrated System LLC / Bonn Metal Cons. Ind.).

---

## 2. Master Catalog & DXF Mapping Table

The table below maps each provided production DXF file to its corresponding entry in `references/product_variations.csv` and its corresponding submittal sheet in `references/DATA SHEET SITE TECHNOLOGY (1).pdf`:

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

## 3. Flat Pattern Anatomy & Geometric Rules Learned from Size Variations

Comparing products provided in multiple sizes reveals the underlying engineering formulas.

### 3.1 Rule R-1: Flange Height Linear Expansion ($\Delta H$)
Comparing `45_deg_bend_100x50.dxf` ($H=50$) with `45_deg_bend_100x100.dxf` ($H=100$):

1. **Overall Flat Pattern Height**:
   - For $H = 50\text{ mm}$: Flat height = **$237.41\text{ mm}$**
   - For $H = 100\text{ mm}$: Flat height = **$337.41\text{ mm}$**
   - **Difference**: Exactly **$+100.00\text{ mm}$** ($+50\text{ mm}$ on top flange, $+50\text{ mm}$ on bottom flange).

2. **Flange Extension Edge Line Lengths**:
   - For $H = 50\text{ mm}$: Edge line length = **$63.00\text{ mm}$** ($50\text{ mm}$ side flange $+ 13\text{ mm}$ return lip/tab).
   - For $H = 100\text{ mm}$: Edge line length = **$113.00\text{ mm}$** ($100\text{ mm}$ side flange $+ 13\text{ mm}$ return lip/tab).
   - **Difference**: Exactly **$+50.00\text{ mm}$**.

3. **Flange Bending Line Lengths**:
   - For $H = 50\text{ mm}$: Flange vertical lines = **$50.00\text{ mm}$**.
   - For $H = 100\text{ mm}$: Flange vertical lines = **$100.00\text{ mm}$**.
   - **Formula**: $\text{Flange Line} \equiv H$.

---

### 3.2 Rule R-2: Bed Width Linear Expansion ($\Delta W$)
Comparing `45_deg_inside_riser_100x200.dxf` ($W=100$) with `45_deg_inside_riser_200x200.dxf` ($W=200$):

1. **Overall Flat Pattern Width**:
   - For $W = 100\text{ mm}$: Flat width = **$517.51\text{ mm}$**
   - For $W = 200\text{ mm}$: Flat width = **$617.51\text{ mm}$**
   - **Difference**: Exactly **$+100.00\text{ mm}$**.

2. **Rigid Wings Invariance**:
   - Left profile wing ($X = 0.0$ to $210.25\text{ mm}$): Identical coordinates down to $0.01\text{ mm}$ across both files.
   - Right profile wing: Shifted horizontally by exactly **$+100.00\text{ mm}$**.
   - **Central Bed Width Line**:
     - At $W = 100\text{ mm}$: Segment length = **$97.00\text{ mm}$** ($W - 3.0\text{ mm}$ bend deduction).
     - At $W = 200\text{ mm}$: Segment length = **$197.00\text{ mm}$** ($W - 3.0\text{ mm}$ bend deduction).

3. **Diagonal Slant Scaling across Width**:
   Comparing `45_deg_bend_100x50` ($W=100$) with `45_deg_bend_450x50` ($W=450$):
   - At $W = 100\text{ mm}$: Slant line length = **$163.00\text{ mm}$**.
   - At $W = 450\text{ mm}$: Slant line length = **$513.00\text{ mm}$**.
   - **Exact Linear Formula**:
     $$\text{Length}_{\text{slant}} = W + H + 13\text{ mm}$$
     - Check $100\times 50$: $100 + 50 + 13 = 163\text{ mm}$ (Exact)
     - Check $100\times 100$: $100 + 100 + 13 = 213\text{ mm}$ (Exact)
     - Check $450\times 50$: $450 + 50 + 13 = 513\text{ mm}$ (Exact)

---

### 3.3 Rule R-3: Branch Width Scaling in Tees and Crosses
1. **Equal vs Unequal Cross** (`tray_accessories`):
   - `equal_cross_100x50.dxf`: $474.84\text{ mm} \times 474.84\text{ mm}$ (Symmetric in both axes).
   - `unequal_cross_100x50x100x50.dxf`: $474.84\text{ mm} \times 424.84\text{ mm}$.
   - **Difference in Y**: Exactly **$-50.00\text{ mm}$** reflecting the $W_2=50\text{ mm}$ branch instead of $W_2=100\text{ mm}$.

2. **Equal vs Unequal Tee** (`tray_accessories`):
   - `equal_tee_100x50.dxf`: $474.84\text{ mm} \times 350.42\text{ mm}$.
   - `unequal_tee_100x50x100x50.dxf`: $424.84\text{ mm} \times 350.42\text{ mm}$.
   - **Difference in X**: Exactly **$-50.00\text{ mm}$** reflecting the $W_1=50\text{ mm}$ run reduction.

---

### 3.4 Rule R-4: Riser Relief Cut V-Notches (Slits for 3D Bending)
In all Inside and Outside Riser DXFs (e.g., `45_deg_inside_riser_100_50.dxf` and `90_deg_inside_riser_100x50.dxf`), the side flanges feature V-notch cutouts:

- **45° Inside Riser**:
  - The cut lines start at $(X=150.0, Y=162.0)$ and radiate outward to $(123.9, 225.0)$ and $(176.1, 225.0)$.
  - Angle: $\arctan(63 / 26.1) = 67.5^\circ \implies 180^\circ - (2 \times 67.5^\circ) = \mathbf{45.0^\circ}$ included angle.
  - When folded along the horizontal bend axis at $Y=162.0$, the two angled edges meet, closing the $45^\circ$ notch and forming the 3D riser wall.
- **90° Inside Riser**:
  - Employs two sequential $45^\circ$ notches (or a double-miter), allowing the sheet to fold twice through $45^\circ$ to achieve the $90^\circ$ vertical elevation.

> **Crucial Diagnostic Finding for Step 1**:  
> In conventional DXF sanitizers, these V-notch slits might be mistakenly flagged as "unclosed gaps" or "floating lines". In sheet metal manufacturing, these are **intentional relief cuts** essential for folding. The DXF Pre-Flight Inspector must recognize V-notches that intersect perimeter boundaries and treat them as valid cut features rather than defects.

---

## 4. Comparison with the Manually Created TDS (`DATA SHEET SITE TECHNOLOGY (1).pdf`)

We conducted a side-by-side comparison between the DXF flat patterns and the 3D isometric submittal sheets:

```
+---------------------------------------------------------------------------------------------+
| DXF Flat Pattern Features                | TDS Isometric & Sectional Callouts               |
+---------------------------------------------------------------------------------------------+
| 1. Return Lip on Cable Tray (13mm tab)   | Sheet 01-04: "Inside Return Flange Height = 11mm"|
| 2. Flange Slot Pattern (r=4.0 arcs)      | Sheet 05, 07, 09, 10, 11: "OBROUND 20X8"         |
| 3. Straight Cable Tray Base Slots        | Sheet 01-04: "OBROUND 30X7"                      |
| 4. Straight Cable Tray Couplers          | Sheet 19-20: Obround 50X7 (H=75) / 30X7 (H=50)  |
| 5. Cable Trunking End Caps               | Sheet 34-36: "WIDTH - 5mm", "HEIGHT - 5mm"       |
| 6. Trunking Couplers                     | Sheet 24-26: H=45 (for H=50), H=70 (for H=75),    |
|                                          |              H=145 (for H=150) (all H-5mm rule!)  |
| 7. Trunking Equal Tee Cover Screws       | Sheet 30-33: Side tabs on tee for cover screws   |
+---------------------------------------------------------------------------------------------+
```

### Detailed Observations:
1. **Coupler Philosophy (Integral Flap vs Separate Coupler)**:
   - On **Tray Accessories** (`45_deg_bend_100x50.dxf`, `equal_tee_100x50.dxf`), the ports feature **integrated coupler flap tabs** with `OBROUND 20X8` slots, extending $63\text{ mm}$ from the miter. This allows adjacent tray lengths or fittings to bolt directly without needing a separate wrap-over coupler.
   - On **Straight Cable Trays** (TDS Sheets 01–04), the ends are flush, and joining is achieved using separate **Straight Coupler Plates** (`SN 33` / `SN 34` with `OBROUND 50X7` and `30X7` slots, TDS Sheets 19–20).
   - On **Trunking Accessories** (TDS Sheets 30–33), fittings have integral side-bolting tabs, whereas straight trunking lengths use **Wrap-Over Trunking Couplers** (`SN 41`–`SN 44`, TDS Sheets 24–26).

2. **Slot Pattern Standards**:
   - `OBROUND 20X8`: Used uniformly across all accessory flanges, riser segments, and trunking couplers.
   - `OBROUND 30X7`: Used for Cable Tray straight runs ($H=50$) and straight couplers ($H=50$).
   - `OBROUND 50X7`: Used for heavier Cable Tray couplers ($H=75$, TDS Sheet 19).
   - `OBROUND 29X12`: Used exclusively on Unistrut slotted channels (TDS Sheet 37).

3. **Insertion & Clearance Formulas**:
   - Trunking End Caps (Sheets 34–36) explicitly state:
     $$\text{Insertion Width} = W - 5\text{ mm}, \quad \text{Insertion Height} = H - 5\text{ mm}$$
   - Trunking Couplers (Sheets 24–26) similarly apply the clearance rule:
     - For nominal $H=50\text{ mm} \implies$ Coupler height is **$45\text{ mm}$**.
     - For nominal $H=75\text{ mm} \implies$ Coupler height is **$70\text{ mm}$**.
     - For nominal $H=150\text{ mm} \implies$ Coupler height is **$145\text{ mm}$**.

---

## 5. Technical Findings on the Provided DXF Files

1. **Model Space vs Paper Space**:
   - Several DXFs (`45_deg_bend_100x100.dxf`, `45_deg_bend_100x50.dxf`, `90_bend_100x50.dxf`, `unequal_tee_...`) contain 2 AutoCAD Paper Space Viewport entities (`VIEWPORT` with group code `67 = 1`).
   - The paper space viewports sit at $(0, 0)$, while the actual sheet metal flat pattern sits at model space coordinates (e.g., $X \approx 1950\text{ mm} - 2320\text{ mm}$).
   - *Fix applied in CADense*: The DXF loader now filters exclusively for Model Space entities (`code 67 == 0` or omitted) and normalizes coordinates to origin $(0, 0)$.

2. **Single Layer Representation**:
   - In all 29 provided DXFs, **all entities reside on Layer `0`**.
   - Outer contour cuts, internal hole cutouts (`OBROUND 20X8`), relief slits, and potential bend lines are not segregated by DXF layer.
   - To perform automated 3D bending in SolidCraft (Step 3), we will programmatically classify entities into distinct semantic layers:
     - `OUTLINE` (outer boundary cut)
     - `SLOTS` (closed obround hole cutouts)
     - `CUT_RELIEF` (open interior slit lines for riser notches)
     - `BEND_UP` / `BEND_DOWN` (fold lines with bend angle & radius metadata)

---

---

## 6. Queries, Suggestions & Options for User Review

### 6.1 User Decisions & Domain Insights (Recorded 2026-10-27)
1. **Laser Cutting Purpose & Layer Segregation**:
   - DXF files are created for CNC laser cutting and therefore intentionally lack layer segregation or bend lines.
   - Bend lines must be created downstream during SolidCraft modeling operations.
   - Minor millimeter variations exist in real parts to accommodate physical press-brake bending tooling and cover-fixing clearances.
   - User confirmed approval of **Option A**: CADense auto-segregates processed DXFs into `OUTLINE`, `BEND_UP`, `BEND_DOWN`, `CUT_RELIEF`, and `SLOTS` for downstream 3D operations.
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
5. **Approval of Candidate Products for Step 3**:
   - User confirmed approval for the 3 candidate products:
     1. `45_deg_bend_100x50.dxf` (Tray 45° Flat Bend)
     2. `45_deg_inside_riser_100x100.dxf` (Trunking 45° Inside Riser with V-notches)
     3. `equal_tee_100x100.dxf` (Trunking Equal Tee with side tabs)

---

## 7. Status & Implemented Deliverables

1. **DXF Visual Inspector Canvas & Abnormality Detection**:
   - Implemented an interactive 2D canvas in CADense Studio (`apps/cadense-studio/src/landing.rs`) inside the DXF Pre-Flight Inspection modal.
   - Model space filtering (`code 67 == 0`) ensures accurate bounding boxes and dimensions, ignoring paper space viewports.
   - Real-time classification and color coding:
     - **Cyan (`#38BDF8`)**: Outer contour boundary perimeter.
     - **Green (`#10B981`)**: Rigid slot perforations and coupler holes (`OBROUND 20X8`, `30X7`).
     - **Orange (`#F59E0B`)**: Legitimate bending relief slits (e.g. 45.0° and 22.5° riser V-notches).
     - **Red (`#EF4444`)**: True open gaps or abnormalities.
   - Integrated full catalog list of all 29 production DXFs across Tray and Trunking accessories.

2. **3D Folded SolidCraft Models (Format 38) for 3 Approved Candidates**:
   - Generated native `.solidcraft` files with baseline thickness $T=1.2\text{ mm}$, rigid return flanges ($11\text{--}13\text{ mm}$), vertical side walls, and fold lines:
     - [`drawings/cms_in_progress/tray_45_deg_bend_100x50.solidcraft`](file:///d:/RustProjects/cadense/drawings/cms_in_progress/tray_45_deg_bend_100x50.solidcraft) (W=100, H=50, T=1.2)
     - [`drawings/cms_in_progress/trunking_45_deg_inside_riser_100x100.solidcraft`](file:///d:/RustProjects/cadense/drawings/cms_in_progress/trunking_45_deg_inside_riser_100x100.solidcraft) (W=100, H=100, T=1.2)
     - [`drawings/cms_in_progress/trunking_equal_tee_100x100.solidcraft`](file:///d:/RustProjects/cadense/drawings/cms_in_progress/trunking_equal_tee_100x100.solidcraft) (W=100, H=100, T=1.2)
   - Mirrored into template repository [`assets/templates/bonn_metal/`](file:///d:/RustProjects/cadense/assets/templates/bonn_metal/).
   - Integrated into Stage 2 ("SolidCraft 3D Bending") in CADense Studio for immediate launching into SolidCraft CAD.

3. **Workspace Health & Zero-Touch Rule R1**:
   - `cargo check --workspace` and `cargo check -p cadense-studio` pass cleanly.
   - `check-additive.ps1` verifies 100% Rule R1 additive-only compliance.
