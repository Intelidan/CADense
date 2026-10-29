//! CADense Modern Theme & Visual Design System.
//!
//! Provides three curated, professional aesthetic themes:
//! 1. `ModernLight` (Nordic Clean / Enterprise SaaS — white cards, crisp slate text, royal blue accent).
//! 2. `WarmClay` (Warm paper & terracotta — natural cream surfaces, warm charcoal typography).
//! 3. `DeepObsidian` (Deep midnight navy & electric cyan — premium layered dark mode, no flat gray).

use egui::{Color32, FontFamily, FontId, TextStyle};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CadenseTheme {
    #[default]
    SolidCraftCobalt,
    ModernLight,
    WarmClay,
    DeepObsidian,
    GruvboxDark,
}

impl CadenseTheme {
    pub fn name(&self) -> &'static str {
        match self {
            Self::SolidCraftCobalt => "SolidCraft Cobalt (Signature CAD Blue)",
            Self::ModernLight => "Modern Light (Nordic Clean)",
            Self::WarmClay => "Warm Clay (Paper & Terracotta)",
            Self::DeepObsidian => "Deep Obsidian (Midnight Navy)",
            Self::GruvboxDark => "Gruvbox Dark (Built in)",
        }
    }

    pub fn is_dark(&self) -> bool {
        matches!(self, Self::DeepObsidian | Self::GruvboxDark)
    }

    pub fn colors(&self) -> ThemeColors {
        match self {
            Self::SolidCraftCobalt => ThemeColors {
                bg_app: Color32::from_rgb(0xF4, 0xF6, 0xFA),      // Clean Light Blueprint Slate
                bg_sidebar: Color32::from_rgb(0x1B, 0x60, 0xAA),  // SolidCraft Signature Cobalt Blue #1B60AA
                bg_card: Color32::from_rgb(0xFF, 0xFF, 0xFF),     // Pure White
                bg_card_alt: Color32::from_rgb(0xEA, 0xEE, 0xF5), // Subtle Blue-Gray
                bg_input: Color32::from_rgb(0xFF, 0xFF, 0xFF),
                border_subtle: Color32::from_rgb(0xD2, 0xDC, 0xEB),
                border_strong: Color32::from_rgb(0xB5, 0xC6, 0xDF),
                text_primary: Color32::from_rgb(0x0F, 0x17, 0x2A),
                text_secondary: Color32::from_rgb(0x47, 0x55, 0x69),
                text_muted: Color32::from_rgb(0x94, 0xA3, 0xB8),
                accent: Color32::from_rgb(0x1D, 0x64, 0xB4),        // SolidCraft Royal Blue
                accent_hover: Color32::from_rgb(0x15, 0x52, 0x96),
                accent_subtle: Color32::from_rgb(0xE0, 0xEE, 0xFF), // Ice Blue
                accent_text: Color32::WHITE,
                badge_success_bg: Color32::from_rgb(0xDC, 0xFC, 0xE7),
                badge_success_text: Color32::from_rgb(0x16, 0x65, 0x34),
                badge_warning_bg: Color32::from_rgb(0xFE, 0xF3, 0xC7),
                badge_warning_text: Color32::from_rgb(0x92, 0x40, 0x0E),
                badge_info_bg: Color32::from_rgb(0xDB, 0xEA, 0xFE),
                badge_info_text: Color32::from_rgb(0x1E, 0x40, 0xAF),
                badge_danger_bg: Color32::from_rgb(0xFE, 0xE2, 0xE2),
                badge_danger_text: Color32::from_rgb(0x99, 0x1B, 0x1B),
                canvas_bg: Color32::from_rgb(0xEB, 0xF0, 0xF8),

                // SolidCraft Signature Design Tokens
                file_rail: Color32::from_rgb(35, 110, 173),        // #236EAD
                file_rail_hover: Color32::from_rgb(43, 118, 180),  // #2B76B4
                file_background: Color32::from_rgb(247, 249, 251), // #F7F9FB
                file_surface: Color32::WHITE,                      // Pure White Card
                file_border: Color32::from_rgb(224, 229, 236),     // #E0E5EC
                dialog_nav: Color32::from_rgb(244, 246, 248),      // Preferences Rail #F4F6F8
                dialog_rule: Color32::from_rgb(226, 232, 240),     // Divider #E2E8F0
                selected_bg: Color32::from_rgb(235, 244, 254),     // Active pill #EBF4FE
                selected_ink: Color32::from_rgb(27, 96, 170),      // Active ink #1B60AA
            },
            Self::ModernLight => ThemeColors {
                bg_app: Color32::from_rgb(0xF8, 0xFA, 0xFC),      // Slate 50
                bg_sidebar: Color32::from_rgb(0xF1, 0xF5, 0xF9),  // Slate 100
                bg_card: Color32::from_rgb(0xFF, 0xFF, 0xFF),     // Pure White
                bg_card_alt: Color32::from_rgb(0xF1, 0xF5, 0xF9), // Soft secondary
                bg_input: Color32::from_rgb(0xFF, 0xFF, 0xFF),
                border_subtle: Color32::from_rgb(0xE2, 0xE8, 0xF0), // Slate 200
                border_strong: Color32::from_rgb(0xCB, 0xD5, 0xE1), // Slate 300
                text_primary: Color32::from_rgb(0x0F, 0x17, 0x2A),  // Slate 900
                text_secondary: Color32::from_rgb(0x47, 0x55, 0x69),// Slate 600
                text_muted: Color32::from_rgb(0x94, 0xA3, 0xB8),    // Slate 400
                accent: Color32::from_rgb(0x25, 0x63, 0xEB),        // Royal Cobalt Blue
                accent_hover: Color32::from_rgb(0x1D, 0x4E, 0xD8),
                accent_subtle: Color32::from_rgb(0xDB, 0xEA, 0xFE), // Blue 100
                accent_text: Color32::WHITE,
                badge_success_bg: Color32::from_rgb(0xDC, 0xFC, 0xE7),
                badge_success_text: Color32::from_rgb(0x16, 0x65, 0x34),
                badge_warning_bg: Color32::from_rgb(0xFE, 0xF3, 0xC7),
                badge_warning_text: Color32::from_rgb(0x92, 0x40, 0x0E),
                badge_info_bg: Color32::from_rgb(0xDB, 0xEA, 0xFE),
                badge_info_text: Color32::from_rgb(0x1E, 0x40, 0xAF),
                badge_danger_bg: Color32::from_rgb(0xFE, 0xE2, 0xE2),
                badge_danger_text: Color32::from_rgb(0x99, 0x1B, 0x1B),
                canvas_bg: Color32::from_rgb(0xF1, 0xF5, 0xF9),

                file_rail: Color32::from_rgb(241, 245, 249),
                file_rail_hover: Color32::from_rgb(226, 232, 240),
                file_background: Color32::from_rgb(248, 250, 252),
                file_surface: Color32::WHITE,
                file_border: Color32::from_rgb(226, 232, 240),
                dialog_nav: Color32::from_rgb(248, 250, 252),
                dialog_rule: Color32::from_rgb(226, 232, 240),
                selected_bg: Color32::from_rgb(219, 234, 254),
                selected_ink: Color32::from_rgb(37, 99, 235),
            },
            Self::WarmClay => ThemeColors {
                bg_app: Color32::from_rgb(0xF5, 0xF2, 0xEB),      // Warm paper
                bg_sidebar: Color32::from_rgb(0xEC, 0xE7, 0xDC),  // Soft stone
                bg_card: Color32::from_rgb(0xFA, 0xF8, 0xF5),     // Cream surface
                bg_card_alt: Color32::from_rgb(0xEE, 0xE9, 0xDD),
                bg_input: Color32::from_rgb(0xFF, 0xFF, 0xFD),
                border_subtle: Color32::from_rgb(0xDD, 0xD5, 0xC0),
                border_strong: Color32::from_rgb(0xC5, 0xBB, 0xA4),
                text_primary: Color32::from_rgb(0x26, 0x23, 0x20),  // Warm charcoal
                text_secondary: Color32::from_rgb(0x5C, 0x55, 0x4E),
                text_muted: Color32::from_rgb(0x8C, 0x83, 0x79),
                accent: Color32::from_rgb(0xC2, 0x5E, 0x3E),        // Terracotta
                accent_hover: Color32::from_rgb(0xAA, 0x4E, 0x31),
                accent_subtle: Color32::from_rgb(0xF7, 0xE5, 0xDD),
                accent_text: Color32::WHITE,
                badge_success_bg: Color32::from_rgb(0xDF, 0xEB, 0xE1),
                badge_success_text: Color32::from_rgb(0x27, 0x56, 0x35),
                badge_warning_bg: Color32::from_rgb(0xFA, 0xEC, 0xD0),
                badge_warning_text: Color32::from_rgb(0x7D, 0x4C, 0x08),
                badge_info_bg: Color32::from_rgb(0xE1, 0xEB, 0xF2),
                badge_info_text: Color32::from_rgb(0x26, 0x55, 0x75),
                badge_danger_bg: Color32::from_rgb(0xF7, 0xDE, 0xD9),
                badge_danger_text: Color32::from_rgb(0x8A, 0x2E, 0x20),
                canvas_bg: Color32::from_rgb(0xEB, 0xE5, 0xD8),

                file_rail: Color32::from_rgb(236, 231, 220),
                file_rail_hover: Color32::from_rgb(222, 215, 202),
                file_background: Color32::from_rgb(245, 242, 235),
                file_surface: Color32::from_rgb(250, 248, 245),
                file_border: Color32::from_rgb(221, 213, 192),
                dialog_nav: Color32::from_rgb(240, 236, 227),
                dialog_rule: Color32::from_rgb(221, 213, 192),
                selected_bg: Color32::from_rgb(247, 229, 221),
                selected_ink: Color32::from_rgb(194, 94, 62),
            },
            Self::DeepObsidian => ThemeColors {
                bg_app: Color32::from_rgb(0x0B, 0x11, 0x20),      // Midnight navy
                bg_sidebar: Color32::from_rgb(0x0F, 0x17, 0x2A),  // Deep slate
                bg_card: Color32::from_rgb(0x15, 0x1E, 0x33),     // Elevated blue-slate
                bg_card_alt: Color32::from_rgb(0x1B, 0x26, 0x40),
                bg_input: Color32::from_rgb(0x0F, 0x17, 0x2A),
                border_subtle: Color32::from_rgb(0x1E, 0x29, 0x3B), // Slate 800
                border_strong: Color32::from_rgb(0x33, 0x41, 0x55), // Slate 700
                text_primary: Color32::from_rgb(0xF8, 0xFA, 0xFC),  // Crisp white
                text_secondary: Color32::from_rgb(0x94, 0xA3, 0xB8),// Slate 400
                text_muted: Color32::from_rgb(0x64, 0x74, 0x8B),    // Slate 500
                accent: Color32::from_rgb(0x38, 0xBD, 0xF8),        // Electric Cyan
                accent_hover: Color32::from_rgb(0x02, 0x84, 0xC7),
                accent_subtle: Color32::from_rgb(0x16, 0x38, 0x59),
                accent_text: Color32::from_rgb(0x0B, 0x11, 0x20),
                badge_success_bg: Color32::from_rgb(0x06, 0x4E, 0x3B),
                badge_success_text: Color32::from_rgb(0x6E, 0xEE, 0xB6),
                badge_warning_bg: Color32::from_rgb(0x78, 0x35, 0x0F),
                badge_warning_text: Color32::from_rgb(0xFC, 0xD3, 0x4D),
                badge_info_bg: Color32::from_rgb(0x1E, 0x3A, 0x8A),
                badge_info_text: Color32::from_rgb(0x93, 0xC5, 0xFD),
                badge_danger_bg: Color32::from_rgb(0x7F, 0x1D, 0x1D),
                badge_danger_text: Color32::from_rgb(0xFC, 0xA5, 0xA5),
                canvas_bg: Color32::from_rgb(0x07, 0x0D, 0x18),

                file_rail: Color32::from_rgb(30, 80, 128),
                file_rail_hover: Color32::from_rgb(46, 99, 149),
                file_background: Color32::from_rgb(11, 17, 32),
                file_surface: Color32::from_rgb(21, 30, 51),
                file_border: Color32::from_rgb(30, 41, 59),
                dialog_nav: Color32::from_rgb(15, 23, 42),
                dialog_rule: Color32::from_rgb(30, 41, 59),
                selected_bg: Color32::from_rgb(22, 56, 89),
                selected_ink: Color32::from_rgb(56, 189, 248),
            },
            Self::GruvboxDark => ThemeColors {
                bg_app: Color32::from_rgb(0x28, 0x28, 0x28),      // Gruvbox Dark background #282828
                bg_sidebar: Color32::from_rgb(0x1D, 0x20, 0x21),  // Gruvbox Dark hard #1D2021
                bg_card: Color32::from_rgb(0x32, 0x30, 0x2F),     // Card surface #32302F
                bg_card_alt: Color32::from_rgb(0x3C, 0x38, 0x36), // Card alt #3C3836
                bg_input: Color32::from_rgb(0x1D, 0x20, 0x21),
                border_subtle: Color32::from_rgb(0x50, 0x49, 0x45),
                border_strong: Color32::from_rgb(0x66, 0x5C, 0x54),
                text_primary: Color32::from_rgb(0xFB, 0xF1, 0xC7),  // Gruvbox bright fg0 #FBF1C7 (High contrast)
                text_secondary: Color32::from_rgb(0xEB, 0xDB, 0xB2),// Gruvbox clear fg1 #EBDBB2
                text_muted: Color32::from_rgb(0xD5, 0xC4, 0xA1),    // Gruvbox legible fg2 #D5C4A1
                accent: Color32::from_rgb(0x8E, 0xC0, 0x7C),        // Gruvbox Aqua #8EC07C
                accent_hover: Color32::from_rgb(0x68, 0x9D, 0x6A),
                accent_subtle: Color32::from_rgb(0x32, 0x44, 0x40),
                accent_text: Color32::from_rgb(0x1D, 0x20, 0x21),
                badge_success_bg: Color32::from_rgb(0x28, 0x3E, 0x2E),
                badge_success_text: Color32::from_rgb(0xB8, 0xBB, 0x26),
                badge_warning_bg: Color32::from_rgb(0x42, 0x36, 0x20),
                badge_warning_text: Color32::from_rgb(0xFA, 0xBD, 0x2F),
                badge_info_bg: Color32::from_rgb(0x20, 0x35, 0x3B),
                badge_info_text: Color32::from_rgb(0x83, 0xA5, 0x98),
                badge_danger_bg: Color32::from_rgb(0x45, 0x22, 0x22),
                badge_danger_text: Color32::from_rgb(0xFB, 0x49, 0x34),
                canvas_bg: Color32::from_rgb(0x1D, 0x20, 0x21),

                file_rail: Color32::from_rgb(0x1D, 0x20, 0x21),
                file_rail_hover: Color32::from_rgb(0x28, 0x28, 0x28),
                file_background: Color32::from_rgb(0x28, 0x28, 0x28),
                file_surface: Color32::from_rgb(0x32, 0x30, 0x2F),
                file_border: Color32::from_rgb(0x50, 0x49, 0x45),
                dialog_nav: Color32::from_rgb(0x1D, 0x20, 0x21),
                dialog_rule: Color32::from_rgb(0x50, 0x49, 0x45),
                selected_bg: Color32::from_rgb(0x3C, 0x38, 0x36),
                selected_ink: Color32::from_rgb(0x8E, 0xC0, 0x7C),
            },
        }
    }

    /// Dynamically applies the theme, colors, and typography scaling to the egui context
    /// using SolidCraft's core theme applicator.
    pub fn apply_to_ctx(&self, ctx: &egui::Context, font_scale: f32) {
        let theme_file_name = match self {
            Self::SolidCraftCobalt => "blueprint",
            Self::ModernLight => "light",
            Self::WarmClay => "clay",
            Self::DeepObsidian => "dark",
            Self::GruvboxDark => "gruvbox-dark",
        };

        let file = solidcraft_ui_egui::theme_files::find(theme_file_name, &[]);
        let custom = file.map(|f| solidcraft_ui_egui::theme::Custom {
            dark: f.base.is_dark(),
            tokens: solidcraft_ui_egui::theme_files::resolve(f),
        });

        solidcraft_ui_egui::theme::set_dark(self.is_dark());
        solidcraft_ui_egui::theme::set_custom(custom);
        solidcraft_ui_egui::theme::apply(ctx);

        // Adjust font scaling on top of SolidCraft IBM Plex typography
        let scale = font_scale.clamp(0.9, 1.4);
        let t = solidcraft_ui_egui::theme::Tokens::get();
        ctx.global_style_mut(|style| {
            style.text_styles = [
                (TextStyle::Heading, FontId::new(t.font_heading * scale * 1.25, FontFamily::Proportional)),
                (TextStyle::Name("Subheading".into()), FontId::new(t.font_subtitle * scale * 1.15, FontFamily::Proportional)),
                (TextStyle::Body, FontId::new(t.font_body * scale * 1.10, FontFamily::Proportional)),
                (TextStyle::Button, FontId::new(t.font_button * scale * 1.05, FontFamily::Proportional)),
                (TextStyle::Monospace, FontId::new(t.font_body * scale, FontFamily::Monospace)),
                (TextStyle::Small, FontId::new(t.font_small * scale * 1.05, FontFamily::Proportional)),
            ]
            .into();

            style.spacing.item_spacing = egui::vec2(t.space_6, t.space_6);
            style.spacing.button_padding = egui::vec2(t.space_8, t.space_4);
        });
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ThemeColors {
    pub bg_app: Color32,
    pub bg_sidebar: Color32,
    pub bg_card: Color32,
    pub bg_card_alt: Color32,
    pub bg_input: Color32,
    pub border_subtle: Color32,
    pub border_strong: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
    pub accent_subtle: Color32,
    pub accent_text: Color32,
    pub badge_success_bg: Color32,
    pub badge_success_text: Color32,
    pub badge_warning_bg: Color32,
    pub badge_warning_text: Color32,
    pub badge_info_bg: Color32,
    pub badge_info_text: Color32,
    pub badge_danger_bg: Color32,
    pub badge_danger_text: Color32,
    pub canvas_bg: Color32,

    // SolidCraft Visual Design Tokens
    pub file_rail: Color32,
    pub file_rail_hover: Color32,
    pub file_background: Color32,
    pub file_surface: Color32,
    pub file_border: Color32,
    pub dialog_nav: Color32,
    pub dialog_rule: Color32,
    pub selected_bg: Color32,
    pub selected_ink: Color32,
}

impl ThemeColors {
    pub fn is_sidebar_blue(&self) -> bool {
        self.bg_sidebar.b() > 140 && self.bg_sidebar.r() < 80
    }
}
