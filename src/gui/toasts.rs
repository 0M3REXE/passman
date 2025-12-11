//! Toast Notifications Module
//!
//! Handles toast notifications with auto-dismiss and animations.

use eframe::egui;
use super::types::{Toast, ToastType};

/// Render toast notifications
pub fn render_toasts(ctx: &egui::Context, toasts: &[Toast]) {
    if toasts.is_empty() {
        return;
    }
    
    // Request repaint for animation
    ctx.request_repaint();
    
    // Render toasts in top-right corner
    egui::Area::new(egui::Id::new("toast_area"))
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-20.0, 50.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                for (i, toast) in toasts.iter().enumerate() {
                    let (bg_color, icon, text_color) = match toast.toast_type {
                        ToastType::Success => (
                            egui::Color32::from_rgb(40, 167, 69),
                            "✓",
                            egui::Color32::WHITE,
                        ),
                        ToastType::Error => (
                            egui::Color32::from_rgb(220, 53, 69),
                            "✕",
                            egui::Color32::WHITE,
                        ),
                        ToastType::Info => (
                            egui::Color32::from_rgb(23, 162, 184),
                            "ℹ",
                            egui::Color32::WHITE,
                        ),
                        ToastType::Warning => (
                            egui::Color32::from_rgb(255, 193, 7),
                            "⚠",
                            egui::Color32::BLACK,
                        ),
                    };
                    
                    // Fade out effect
                    let alpha = (toast.progress() * 255.0) as u8;
                    let bg_with_alpha = egui::Color32::from_rgba_unmultiplied(
                        bg_color.r(), bg_color.g(), bg_color.b(), alpha
                    );
                    
                    egui::Frame::none()
                        .fill(bg_with_alpha)
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                        .shadow(egui::epaint::Shadow {
                            offset: egui::vec2(2.0, 2.0),
                            blur: 8.0,
                            spread: 0.0,
                            color: egui::Color32::from_black_alpha(50),
                        })
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.colored_label(text_color, icon);
                                ui.colored_label(text_color, &toast.message);
                            });
                            
                            // Progress bar showing remaining time
                            let progress_color = egui::Color32::from_white_alpha(100);
                            let rect = ui.available_rect_before_wrap();
                            let progress_rect = egui::Rect::from_min_size(
                                egui::pos2(rect.min.x, rect.max.y - 2.0),
                                egui::vec2(rect.width() * toast.progress(), 2.0),
                            );
                            ui.painter().rect_filled(progress_rect, 0.0, progress_color);
                        });
                    
                    if i < toasts.len() - 1 {
                        ui.add_space(8.0);
                    }
                }
            });
        });
}
