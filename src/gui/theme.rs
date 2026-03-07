//! Theme Module
//!
//! Handles application theming and visual styling with a modern dark aesthetic.

use eframe::egui;
use super::types::{Theme, SPACING};

// ============================================================================
// MODERN COLOR PALETTE
// ============================================================================

/// Deep background (panels, window)
const BG_BASE: egui::Color32 = egui::Color32::from_rgb(22, 22, 26);
/// Elevated surface (cards, frames)
const BG_SURFACE: egui::Color32 = egui::Color32::from_rgb(30, 31, 36);
/// Widget backgrounds (text inputs idle)
const BG_WIDGET: egui::Color32 = egui::Color32::from_rgb(38, 39, 46);
/// Widget hovered
const BG_WIDGET_HOVER: egui::Color32 = egui::Color32::from_rgb(46, 47, 56);
/// Widget active/focused
const BG_WIDGET_ACTIVE: egui::Color32 = egui::Color32::from_rgb(52, 53, 64);
/// Text input background (extreme_bg)
const BG_INPUT: egui::Color32 = egui::Color32::from_rgb(26, 27, 34);

/// Primary accent (blue)
const ACCENT: egui::Color32 = egui::Color32::from_rgb(99, 138, 255);
/// Selection highlight
const SELECTION: egui::Color32 = egui::Color32::from_rgb(99, 138, 255);

/// Borders - subtle
const BORDER_SUBTLE: egui::Color32 = egui::Color32::from_rgb(50, 52, 60);
/// Borders - medium (widget strokes)
const BORDER_MEDIUM: egui::Color32 = egui::Color32::from_rgb(62, 64, 74);
/// Borders - hover
const BORDER_HOVER: egui::Color32 = egui::Color32::from_rgb(80, 82, 96);
/// Borders - active / focus ring
const BORDER_FOCUS: egui::Color32 = egui::Color32::from_rgb(99, 138, 255);

/// Primary text
const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(230, 232, 240);
/// Secondary/muted text
const TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(140, 144, 160);

// ============================================================================
// THEME APPLICATION
// ============================================================================

/// Apply theme to egui context
pub fn apply_theme(_theme: &Theme, ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    // ── Dark mode ──────────────────────────────────────────────────────
    style.visuals.dark_mode = true;
    style.visuals.override_text_color = Some(TEXT_PRIMARY);
    
    // ── Panel / Window fills ───────────────────────────────────────────
    style.visuals.window_fill = BG_SURFACE;
    style.visuals.panel_fill = BG_BASE;
    style.visuals.faint_bg_color = egui::Color32::from_rgb(34, 35, 42);
    style.visuals.code_bg_color = egui::Color32::from_rgb(34, 35, 42);
    style.visuals.extreme_bg_color = BG_INPUT;
    
    // ── Non-interactive widgets ────────────────────────────────────────
    style.visuals.widgets.noninteractive.bg_fill = BG_WIDGET;
    style.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, BORDER_SUBTLE);
    style.visuals.widgets.noninteractive.rounding = egui::Rounding::same(2.0);
    style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, TEXT_MUTED);
    style.visuals.widgets.noninteractive.expansion = 0.0;
    
    // ── Inactive (idle interactive widgets) ────────────────────────────
    style.visuals.widgets.inactive.bg_fill = BG_WIDGET;
    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, BORDER_MEDIUM);
    style.visuals.widgets.inactive.rounding = egui::Rounding::same(2.0);
    style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);
    style.visuals.widgets.inactive.expansion = 0.0;
    
    // ── Hovered widgets ────────────────────────────────────────────────
    style.visuals.widgets.hovered.bg_fill = BG_WIDGET_HOVER;
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.5, BORDER_HOVER);
    style.visuals.widgets.hovered.rounding = egui::Rounding::same(2.0);
    style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, egui::Color32::WHITE);
    style.visuals.widgets.hovered.expansion = 1.0;
    
    // ── Active (pressed / focused) widgets ─────────────────────────────
    style.visuals.widgets.active.bg_fill = BG_WIDGET_ACTIVE;
    style.visuals.widgets.active.bg_stroke = egui::Stroke::new(2.0, BORDER_FOCUS);
    style.visuals.widgets.active.rounding = egui::Rounding::same(2.0);
    style.visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, egui::Color32::WHITE);
    style.visuals.widgets.active.expansion = 0.0;
    
    // ── Selection ──────────────────────────────────────────────────────
    style.visuals.selection.bg_fill = SELECTION.linear_multiply(0.35);
    style.visuals.selection.stroke = egui::Stroke::new(1.0, ACCENT);
    
    // ── Window styling ─────────────────────────────────────────────────
    style.visuals.window_rounding = egui::Rounding::same(2.0);
    style.visuals.window_shadow = egui::epaint::Shadow {
        offset: egui::vec2(0.0, 4.0),
        blur: 16.0,
        spread: 0.0,
        color: egui::Color32::from_black_alpha(80),
    };
    
    // ── Spacing ────────────────────────────────────────────────────────
    style.spacing.item_spacing = egui::vec2(SPACING, SPACING);
    style.spacing.button_padding = egui::vec2(16.0, 10.0);
    style.spacing.window_margin = egui::Margin::same(16.0);
    style.spacing.indent = 20.0;
    
    // ── Interaction ────────────────────────────────────────────────────
    style.visuals.interact_cursor = Some(egui::CursorIcon::PointingHand);
    
    ctx.set_style(style);
}

// ============================================================================
// COLOR ACCESSORS
// ============================================================================

/// Get panel fill color for theme
pub fn panel_fill(_theme: &Theme) -> egui::Color32 {
    BG_BASE
}

/// Get frame fill color for theme (for entry cards, etc.)
pub fn frame_fill(_theme: &Theme) -> egui::Color32 {
    BG_SURFACE
}

/// Get subtle border color
pub fn border_color(_theme: &Theme) -> egui::Color32 {
    BORDER_SUBTLE
}

/// Get muted text color
pub fn muted_text_color(_theme: &Theme) -> egui::Color32 {
    TEXT_MUTED
}

/// Get header background color
pub fn header_bg_color(_theme: &Theme) -> egui::Color32 {
    BG_SURFACE
}

/// Get search bar background color  
pub fn search_bg_color(_theme: &Theme) -> egui::Color32 {
    BG_INPUT
}

/// Get accent color
pub fn accent_color(_theme: &Theme) -> egui::Color32 {
    ACCENT
}

/// Get dim text color
pub fn dim_text_color(_theme: &Theme) -> egui::Color32 {
    egui::Color32::from_rgb(90, 93, 108)
}
