//! Theme Module
//!
//! Handles application theming and visual styling.

use eframe::egui;
use super::types::{Theme, SPACING};

/// Apply theme to egui context
pub fn apply_theme(_theme: &Theme, ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    // Dark theme only
    style.visuals.dark_mode = true;
    style.visuals.override_text_color = Some(egui::Color32::WHITE);
    style.visuals.window_fill = egui::Color32::from_rgb(32, 33, 36);
    style.visuals.panel_fill = egui::Color32::from_rgb(32, 33, 36);
    style.visuals.faint_bg_color = egui::Color32::from_rgb(45, 46, 49);
    style.visuals.code_bg_color = egui::Color32::from_rgb(45, 46, 49);
    style.visuals.extreme_bg_color = egui::Color32::from_rgb(45, 46, 49);
    
    // Widget colors
    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(45, 46, 49);
    style.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100));
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(50, 52, 56);
    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(120, 120, 120));
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(60, 62, 66);
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.5, egui::Color32::from_rgb(150, 150, 150));
    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(70, 72, 76);
    style.visuals.widgets.active.bg_stroke = egui::Stroke::new(1.5, egui::Color32::from_rgb(70, 130, 180));
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(100, 150, 255);
    
    // Common styling
    style.visuals.window_rounding = egui::Rounding::same(6.0);
    style.visuals.widgets.noninteractive.rounding = egui::Rounding::same(4.0);
    style.visuals.widgets.inactive.rounding = egui::Rounding::same(4.0);
    style.visuals.widgets.hovered.rounding = egui::Rounding::same(4.0);
    style.visuals.widgets.active.rounding = egui::Rounding::same(4.0);
    style.spacing.item_spacing = egui::vec2(SPACING, SPACING);
    style.spacing.button_padding = egui::vec2(12.0, 8.0);
    
    ctx.set_style(style);
}

/// Get panel fill color for theme
pub fn panel_fill(_theme: &Theme) -> egui::Color32 {
    egui::Color32::from_rgb(32, 33, 36)
}

/// Get frame fill color for theme (for entry cards, etc.)
pub fn frame_fill(_theme: &Theme) -> egui::Color32 {
    egui::Color32::from_rgb(40, 42, 46)
}

/// Get subtle border color
pub fn border_color(_theme: &Theme) -> egui::Color32 {
    egui::Color32::from_rgb(60, 63, 68)
}

/// Get muted text color
pub fn muted_text_color(_theme: &Theme) -> egui::Color32 {
    egui::Color32::from_rgb(140, 145, 155)
}

/// Get header background color
pub fn header_bg_color(_theme: &Theme) -> egui::Color32 {
    egui::Color32::from_rgb(38, 40, 44)
}

/// Get search bar background color  
pub fn search_bg_color(_theme: &Theme) -> egui::Color32 {
    egui::Color32::from_rgb(45, 47, 52)
}
