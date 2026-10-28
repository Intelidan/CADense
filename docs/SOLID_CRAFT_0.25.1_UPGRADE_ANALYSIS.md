# SolidCraft v0.25.1 Upstream Upgrade & CMS Impact Analysis
**CADense Industrial Sheet Metal & Cable Management Systems (CMS) Automation Platform**  
**Document Ref**: `CADENSE-UPGRADE-2026-003`  
**Date**: 2026-10-28  
**Upstream Target**: `tangency-labs/solidcraft` release `v0.25.1` (commit `37a586dd`, Design Format 41)  
**Baseline Version**: `v0.24.0` (commit `5965439a`, Design Format 39)  
**Intermediate Releases**: `v0.24.1` (Format 40), `v0.25.0` (Format 41)  

---

## 1. Executive Summary & Progression

Upstream SolidCraft has delivered two major release cycles (`v0.24.1` and `v0.25.0`) along with a maintenance patch (`v0.25.1`), encompassing **168 commits** modifying **478 files** (+47,874 / -3,299 lines):

1. **v0.24.1 (Format 40)**:
   - In-sketch right-click dimension deletion.
   - Canvas value expressions auto-expanding fields.
   - Viewport highlight for selected sketch and hover-edge performance improvements.
   - Redefine feature pipeline optimizations (features following an edited feature evaluated only once).
   - Theme additions: Built-in CGA and 1-Bit themes.

2. **v0.25.0 (Format 41)**:
   - **Interactive Drawing Insertion (`Insert SVG / DXF`)**: Interactive 3D placement, sizing, and rotation before inserting 2D profile vectors into a design.
   - **Order-Independent Transparency (OIT)**: Accurate rendering for transparent, translucent, and faded bodies in the 3D viewport.
   - **Cross-Component Body & Component Copy/Paste**: Ability to duplicate modular sub-assemblies across components and designs.
   - **Send to Slicer Integration**: One-click dispatch of 3D geometry to additive manufacturing slicers.
   - **Cone 3D Primitive (`solid.cone`)**: Direct conical solids with top and base radii.
   - **Origin Planes Renamed**: Intuitive Top / Front / Right standard labels.
   - **Sketches Nested Under Features**: Clean timeline tree where sketches are automatically grouped under the feature that consumes them.
   - **Free Orbit Direction Snapping**: Precise 45°/90° snaps with Shift key.
   - **Session Recovery**: Automatic re-opening of designs active at the last session exit.

3. **v0.25.1 (Format 41 Maintenance)**:
   - Extrude-to-object join boundary safety across external components.
   - Browser bodies folder settings: Always, Several bodies only (default), or Never.

Under **Rule R1 (Strict Zero-Touch Upstream Compliance)**, all upstream crates remained 100% pure and untouched, allowing `./scripts/sync-upstream.ps1 -TargetTag v0.25.1` to execute with **zero merge conflicts** and 100% compiler and test pass rates.

---

## 2. Key CMS & Industrial Sheet Metal Benefits for CADense

| SolidCraft Capability | Upstream Commit | CADense CMS Impact |
| :--- | :--- | :--- |
| **Interactive DXF/SVG Placement** | `5622963b` | Direct placement and orientation of flat pattern cutouts and custom punch tools onto sheet faces prior to folding. |
| **Order-Independent Transparency (OIT)** | `efa2cae0` | Photorealistic inspection of transparent Cable Tray covers, acrylic inspection panels, and overlapping sheet metal flanges. |
| **Component & Body Copy/Paste** | `e8eb7ee1` | Rapid modular assembly of Cable Ladder rungs, Coupler Bracket pairs, and Unistrut bracket fixtures. |
| **Nested Sketch Timeline Hierarchy** | `f1cc49bc` | Decluttered feature tree for complex CMS multi-port transitions (reduces visual clutter in 4-port reducers and tees). |
| **Design Format 41 Migration** | `03d03424` | Upgraded `.solidcraft` and `.dsf` project container compatibility ensuring CADense files load transparently in all future builds. |

---

## 3. Verification & Compliance Evidence

- **Rule R1 Additive Check**:
  ```powershell
  ./scripts/check-additive.ps1 -UpstreamRef v0.25.1
  # Result: Rule R1 Check Passed: All CADense extensions are strictly additive!
  ```
- **Cargo Build Verification**:
  ```powershell
  cargo check -p cadense-dsf -p cadense-order -p cadense-ui -p cadense-dds-pdf -p cadense-studio
  # Result: Finished `dev` profile [optimized + debuginfo] (0 errors, 0 warnings in CADense crates)
  ```
- **Automated Test Suite**:
  ```powershell
  cargo test -p cadense-dsf -p cadense-order -p cadense-ui -p cadense-dds-pdf
  # Result: ok. All tests passed (Trapezoidal scaler, Bonn Metal CSV batching, Clearance rules, DDS PDF generation).
  ```
