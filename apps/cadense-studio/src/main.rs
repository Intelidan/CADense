//! `cadense-studio` — Main Binary Entrypoint.
//! Launches the branded CADense Studio wrapper application with the Concept 23
//! Deep Cobalt Steel-Blue (`#3B82F6`) T-Bar Tongue icon and multi-workspace shell.

mod app;
mod landing;
mod solid_renderer;

use app::CadenseStudioApp;
use cadense_ui::CadenseLogoWidget;
use eframe::egui;
use std::sync::Arc;

fn main() -> eframe::Result<()> {
    let (rgba, width, height) = CadenseLogoWidget::generate_window_icon_rgba();
    let icon_data = egui::IconData { rgba, width, height };

    let viewport = egui::ViewportBuilder::default()
        .with_title("CADense Studio v1.0.0 — CMS Sheet Metal Automation (SolidCraft v0.25.1, Format 41)")
        .with_inner_size([1480.0, 920.0])
        .with_min_inner_size([1080.0, 720.0])
        .with_icon(Arc::new(icon_data));

    let native_options = eframe::NativeOptions { viewport, ..Default::default() };

    eframe::run_native(
        "CADense Studio",
        native_options,
        Box::new(|cc| {
            // Register SolidCraft bundled IBM Plex Sans interface fonts (400, 500, 600, 700) + Mono + Phosphor icons
            let mut fonts = solidcraft_ui_egui::fonts::base_font_definitions();
            egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::new(CadenseStudioApp::new(cc)))
        }),
    )
}
