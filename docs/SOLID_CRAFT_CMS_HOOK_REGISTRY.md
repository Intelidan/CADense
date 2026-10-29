# SolidCraft CMS Hook Registry
## Upstream Zero-Touch Compliance & Minimal Injection Log (Validated on SolidCraft v0.26.1, Format 43)

This document records every line of code injected into upstream `tangency-labs/solidcraft` native files to support **CMS truFold**. Following the agreed **99% Isolated Sub-Crate / 1% Hook Injection** strategy, 99% of domain logic, math, Flutter-derived scalers, and UI components live isolated in `crates/solidcraft_cms/`, while native SolidCraft files only receive minimal delegation hooks.

**Upstream Release Sync Status**:
- `v0.25.1` (Format 41): Initial integration
- `v0.26.1` (Format 43): Cleanly merged, zero upstream regressions, Rule R1 additive check passing 100%.

---

### 1. `Cargo.toml` (Workspace Root)
- **Line Injected**: `solidcraft-cms = { path = "crates/solidcraft_cms" }` under `[workspace.dependencies]`.
- **Purpose**: Registers the `solidcraft-cms` sub-crate within the workspace dependency tree.

---

### 2. `crates/ui-egui/Cargo.toml`
- **Line Injected**:
  ```toml
  # === [CADENSE-HOOK: CMS truFold sub-crate dependency] ===
  solidcraft-cms = { workspace = true }
  ```
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
- **Hook 2 (Dual-Ribbon Renderer & Command Bridge Delegation)**:
  ```rust
  // === [CADENSE-HOOK: CMS truFold Dual-Ribbon Renderer Delegation] ===
  if tab == "CMS TRUFOLD" {
      let panel_rect = Rect::from_min_max(pos2(ribbon_left, r.top() + TAB_ROW_HEIGHT), pos2(r.right(), r.bottom()));
      // ... extracts in_sketch, active_sketch_id, selected_curves, sketch_curves, existing_extrude_id, extrude_thickness ...
      let mut dispatch_fn = |action: solidcraft_cms::ui::CmsAppAction| -> Result<serde_json::Value, String> {
          match action {
              solidcraft_cms::ui::CmsAppAction::Run(cmd, params) => app.run(&cmd, params),
              solidcraft_cms::ui::CmsAppAction::Start(cmd) => { app.start(&cmd); Ok(serde_json::Value::Null) }
              solidcraft_cms::ui::CmsAppAction::EditFeature(id) => { app.edit_feature(id); Ok(serde_json::Value::Null) }
          }
      };
      let mut bridge = solidcraft_cms::ui::CmsAppBridge { in_sketch, active_sketch_id, selected_curves, sketch_curves, existing_extrude_id, extrude_thickness, dispatch: &mut dispatch_fn };
      solidcraft_cms::ui::render_cms_ribbon(ui, panel_rect, Some(&mut bridge));
      return;
  }
  ```
- **Purpose**: Hands over panel drawing for the `"CMS TRUFOLD"` tab to `solidcraft_cms::ui::render_cms_ribbon`. Bridges full command dispatching (`solid.extrude`, `timeline.rename`, `sketch.line`), existing extrude feature detection, evaluated thickness querying, and timeline feature activation (`app.edit_feature(id)`) for arrow drag manipulation.

---

### 5. `crates/ui-egui/src/viewport.rs`
- **Hook (3D Viewport Raycasting Projection & Interactive Hinge Drawing)**:
  ```rust
  // === [CADENSE-HOOK: CMS truFold Viewport Overlay & Interactive Hinge Drawing] ===
  if app.ui.tab == "CMS TRUFOLD" {
      let active_plane = app.session.active_sketch.and_then(|sid| app.session.world_state().sketch(sid).map(|ss| ss.plane)).unwrap_or(Plane::XY);
      let to_screen_fn = |p: Vec3| -> Option<egui::Pos2> { proj.to_screen(p) };
      let screen_to_sketch_fn = |p: egui::Pos2| -> Option<(f64, f64)> {
          let (o, d) = proj.ray(p);
          let hit = active_plane.intersect_ray(o, d)?;
          let local = active_plane.to_local(hit);
          Some((local.x, local.y))
      };
      let cms_proj = solidcraft_cms::ui::CmsViewportProjection { to_screen: &to_screen_fn, screen_to_sketch: &screen_to_sketch_fn, plane: active_plane };
      // ... builds bridge ...
      solidcraft_cms::ui::render_cms_viewport(ui, &painter, rect, Some(&cms_proj), Some(&mut bridge));
  }
  ```
- **Purpose**: Supplies true 3D camera projection (`Proj::to_screen`) and 3D raycasting (`active_plane.intersect_ray`) so sheet geometry, hinge lines, port badges, and folded 3D meshes adhere directly to the 3D sheet model and camera rather than a flat 2D screen plane. Automatically records placed hinge lines directly into SolidCraft's native sketch entities.

---

### 6. `crates/ui-egui/src/toolbar.rs` (Brand Identity Mark & Title Lockup)
- **Hook**:
  ```rust
  // === [CADENSE-HOOK: CADense Brand Identity Mark & Title Lockup] ===
  {
      let _ = crate::icons::glyphs::brand_mark;
      let c = pos2(left + 20.0, r.center().y);
      solidcraft_cms::branding::paint_cadense_brand_mark(ui.painter(), c, 22.0);
  }
  if !compact_actions {
      ui.painter().text(
          pos2(left + 36.0, r.center().y),
          Align2::LEFT_CENTER,
          "CADense",
          crate::fonts::semibold(t.font_heading),
          t.app_bar_text,
      );
  }
  ```
- **Purpose**: Replaces upstream SolidCraft brand mark and title text in the application toolbar with CADense Concept 23 Cobalt/Silver interlocking badge and `"CADense"` heading.

---

### 7. `crates/ui-egui/src/help.rs` (About Dialog & Diagnostics)
- **Hook 1 (Diagnostic Info)**:
  ```rust
  // === [CADENSE-HOOK: CADense Diagnostic Info] ===
  let v = format!("CADense {} (TruFold Engine v{}, built on {})", solidcraft_cms::branding::CadenseVersion::VERSION, solidcraft_cms::branding::CadenseVersion::TRUFOLD_ENGINE_VERSION, solidcraft_cms::branding::CadenseVersion::UPSTREAM_ENGINE);
  ```
- **Hook 2 (About Dialog Rebranding)**:
  ```rust
  // === [CADENSE-HOOK: CADense About Dialog Rebranding] ===
  ui.vertical_centered(|ui| {
      let (rect, _) = ui.allocate_exact_size(egui::vec2(64.0, 64.0), egui::Sense::hover());
      solidcraft_cms::branding::paint_cadense_brand_mark(ui.painter(), rect.center(), 56.0);
      ui.add_space(8.0);
      ui.heading(egui::RichText::new(solidcraft_cms::branding::CadenseVersion::APP_NAME).strong().size(22.0).color(solidcraft_cms::branding::CADENSE_SILVER_WHITE));
      ui.label(egui::RichText::new(solidcraft_cms::branding::CadenseVersion::TAGLINE).size(12.0).color(solidcraft_cms::branding::CADENSE_SILVER_BEVEL));
      ui.add_space(4.0);
      ui.label(egui::RichText::new(format!("Version {} (TruFold Engine {})", solidcraft_cms::branding::CadenseVersion::VERSION, solidcraft_cms::branding::CadenseVersion::TRUFOLD_ENGINE_VERSION)).small().color(solidcraft_cms::branding::CADENSE_COBALT_BLUE));
      ui.label(egui::RichText::new(format!("Built on {}", solidcraft_cms::branding::CadenseVersion::UPSTREAM_ENGINE)).italics().small().color(egui::Color32::from_rgb(140, 150, 165)));
  });
  ```
- **Purpose**: Rebrands the Help & Diagnostics dialog with CADense identity, TruFold engine version, license, and upstream SolidCraft attribution.

---

### 8. `crates/ui-egui/src/home.rs` (Home Hero Banner & CMS Preset Launcher)
- **Hook 1 (Hero Banner & Preset Launcher)**:
  ```rust
  // === [CADENSE-HOOK: CADense Home Hero Banner & Preset Launcher] ===
  solidcraft_cms::branding::render_cadense_home_hero(ui, &mut |preset_name| {
      match preset_name {
          "reducer" => solidcraft_cms::ui::load_preset_reducer(),
          "tee" => solidcraft_cms::ui::load_preset_unequal_tee(),
          "bend" => solidcraft_cms::ui::load_preset_bend(),
          "cross" => solidcraft_cms::ui::load_preset_cross(),
          _ => {}
      }
  });
  ```
- **Hook 2 (Nav Row Label)**:
  ```rust
  let (about_rect, _) = ui.allocate_exact_size(vec2(120.0, 24.0), Sense::click());
  // Changes label from "About SolidCraft" to "About CADense"
  ```
- **Purpose**: Displays the prominent CADense brand header on the Start/Landing page with 4 instant-start CMS fitting templates (Concentric Reducer, Unequal Tee, 90° Bend, 4-Way Cross).

---

### 9. `apps/solidcraft/src/main.rs` (Window Title & Process Name)
- **Hook 1 (Window Title)**:
  ```rust
  // === [CADENSE-HOOK: CADense Window Title & Branding] ===
  .with_title("CADense — Sheet Metal & CMS Automation")
  ```
- **Hook 2 (macOS Process Name)**:
  ```rust
  // === [CADENSE-HOOK: CADense Process Name] ===
  objc2_foundation::NSProcessInfo::processInfo().setProcessName(objc2_foundation::ns_string!("CADense"));
  ```
- **Purpose**: Sets OS desktop window title, taskbar label, and macOS application process identity to CADense.

---

### 10. `apps/solidcraft/src/native_menu.rs` (Native Application Submenu)
- **Hook 1 (Application Menu Title)**:
  ```rust
  // === [CADENSE-HOOK: CADense Native Menu Rebranding] ===
  let app_menu = Submenu::new("CADense", true);
  ```
- **Hook 2 (About Menu Item)**:
  ```rust
  // === [CADENSE-HOOK: CADense Native Menu About] ===
  MenuItem::new("About CADense", true, None)
  ```
- **Purpose**: Rebrands native OS menu bar roots and "About CADense" commands.

---

### 11. `apps/solidcraft/build.rs` (Windows Executable Binary Properties)
- **Hook**:
  ```rust
  // === [CADENSE-HOOK: CADense Windows Binary Properties] ===
  VALUE "CompanyName", "CADense Industrial Automation"
  VALUE "FileDescription", "CADense Sheet Metal & CMS Automation Platform"
  VALUE "FileVersion", "{version}"
  VALUE "InternalName", "cadense"
  VALUE "OriginalFilename", "cadense.exe"
  VALUE "ProductName", "CADense"
  VALUE "ProductVersion", "{version}"
  ```
- **Purpose**: Injects CADense product metadata into the generated Windows executable (`.exe` File Details in Windows Explorer).

---

### 12. Automation & Synchronization Script
- **Script**: `scripts/apply-branding.ps1`
- **Purpose**: Automatically checks, validates, and re-applies all CADense brand hooks into native files idempotently. Called automatically by `scripts/sync-upstream.ps1` immediately after upstream release merges to prevent regressions.
