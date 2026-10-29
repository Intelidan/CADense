# SolidCraft CMS Hook Registry
## Upstream Zero-Touch Compliance & Minimal Injection Log

This document records every line of code injected into upstream `tangency-labs/solidcraft` native files to support **CMS truFold**. Following the agreed **99% Isolated Sub-Crate / 1% Hook Injection** strategy, 99% of domain logic, math, Flutter-derived scalers, and UI components live isolated in `crates/solidcraft_cms/`, while native SolidCraft files only receive minimal delegation hooks.

---

### 1. `Cargo.toml` (Workspace Root)
- **Line Injected**: `solidcraft-cms = { path = "crates/solidcraft_cms" }` under `[workspace.dependencies]`.
- **Purpose**: Registers the `solidcraft-cms` sub-crate within the workspace dependency tree.

---

### 2. `crates/ui-egui/Cargo.toml`
- **Line Injected**: `solidcraft-cms = { workspace = true }` under `[dependencies]`.
- **Purpose**: Allows `ui-egui` to link to `solidcraft_cms::ui` for rendering delegation.

---

### 3. `crates/ui-egui/src/workspace.rs`
- **Hook 1 (Ribbon Section Definitions)**:
  ```rust
  // === [CADENSE-HOOK: CMS truFold Ribbon Section Definition] ===
  Section { tab: "CMS TRUFOLD", panel: "FOLDING", promoted: 1, ids: &[] },
  Section { tab: "CMS TRUFOLD", panel: "SCALING", promoted: 1, ids: &[] },
  ```
- **Hook 2 (Visible Tabs List)**:
  ```rust
  // === [CADENSE-HOOK: CMS truFold Tab Registration] ===
  tabs.push("CMS TRUFOLD");
  ```
- **Purpose**: Makes `"CMS TRUFOLD"` appear in the tab row right alongside `"SHEET METAL"`, selectable by the user.

---

### 4. `crates/ui-egui/src/toolbar.rs`
- **Hook 1 (Tab Accent Underline)**:
  ```rust
  if matches!(app.ui.tab.as_str(), "SKETCH" | "SHEET METAL" | "CMS TRUFOLD")
  ```
- **Hook 2 (Dual-Ribbon Renderer Delegation)**:
  ```rust
  // === [CADENSE-HOOK: CMS truFold Dual-Ribbon Renderer Delegation] ===
  if tab == "CMS TRUFOLD" {
      let panel_rect = Rect::from_min_max(pos2(ribbon_left, r.top() + TAB_ROW_HEIGHT), pos2(r.right(), r.bottom()));
      solidcraft_cms::ui::render_cms_ribbon(ui, panel_rect);
      return;
  }
  ```
- **Purpose**: Hands over panel drawing for the `"CMS TRUFOLD"` tab to `solidcraft_cms::ui::render_cms_ribbon`.

---

### 5. `crates/ui-egui/src/viewport.rs`
- **Hook (Viewport Overlay & Interactive Hinge Drawing)**:
  ```rust
  // === [CADENSE-HOOK: CMS truFold Viewport Overlay & Interactive Hinge Drawing] ===
  if app.ui.tab == "CMS TRUFOLD" {
      solidcraft_cms::ui::render_cms_viewport(ui, &painter, rect);
  }
  ```
- **Purpose**: Paints the CMS 2D sheet blank, port markers, coupler slot holes, 3D folded geometry, and interactive rubber-band hinge drawing directly onto the 3D viewport canvas.
