//! Overlays Module
//!
//! Modal dialogs, loading overlays, and onboarding wizard.

use eframe::egui;
use super::types::{SPACING, BUTTON_HEIGHT};
use super::widgets::ButtonWidgets;

/// Show loading overlay with animated spinner
pub fn render_loading_overlay(ctx: &egui::Context, is_loading: bool, loading_message: &str) {
    if !is_loading {
        return;
    }
    
    // Modal background overlay
    egui::Area::new(egui::Id::new("loading_overlay"))
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(0.0, 0.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let screen_rect = ctx.screen_rect();
            ui.painter().rect_filled(
                screen_rect,
                0.0,
                egui::Color32::from_black_alpha(180),
            );
            
            // Center the spinner
            let center = screen_rect.center();
            
            // Draw spinner animation
            let time = ctx.input(|i| i.time);
            let angle = (time * 2.0) % (2.0 * std::f64::consts::PI);
            let spinner_radius = 20.0;
            let spinner_center = egui::pos2(center.x, center.y - 20.0);
            
            // Draw spinning arc
            for i in 0..8 {
                let segment_angle = angle + (i as f64 * std::f64::consts::PI / 4.0);
                let alpha = ((i as f32 + 1.0) / 8.0 * 255.0) as u8;
                let start = egui::pos2(
                    spinner_center.x + (spinner_radius * segment_angle.cos() as f32),
                    spinner_center.y + (spinner_radius * segment_angle.sin() as f32),
                );
                let end = egui::pos2(
                    spinner_center.x + ((spinner_radius - 5.0) * segment_angle.cos() as f32),
                    spinner_center.y + ((spinner_radius - 5.0) * segment_angle.sin() as f32),
                );
                ui.painter().line_segment(
                    [start, end],
                    egui::Stroke::new(3.0, egui::Color32::from_rgba_unmultiplied(70, 130, 180, alpha)),
                );
            }
            
            // Loading message
            if !loading_message.is_empty() {
                let text_pos = egui::pos2(center.x, center.y + 30.0);
                ui.painter().text(
                    text_pos,
                    egui::Align2::CENTER_CENTER,
                    loading_message,
                    egui::FontId::proportional(16.0),
                    egui::Color32::WHITE,
                );
            }
        });
    
    // Request repaint for animation
    ctx.request_repaint();
}

/// Render onboarding wizard for first-time users
pub fn render_onboarding(
    ctx: &egui::Context,
    show_onboarding: &mut bool,
    onboarding_step: &mut u8,
) {
    if !*show_onboarding {
        return;
    }
    
    // Modal background overlay
    egui::Area::new(egui::Id::new("onboarding_overlay"))
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(0.0, 0.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let screen_rect = ctx.screen_rect();
            ui.painter().rect_filled(
                screen_rect,
                0.0,
                egui::Color32::from_black_alpha(200),
            );
        });
    
    // Onboarding window
    egui::Window::new("👋 Welcome to Passman!")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .fixed_size([450.0, 350.0])
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            ui.add_space(SPACING);
            
            match *onboarding_step {
                0 => {
                    ui.heading("🔐 Secure Password Management");
                    ui.add_space(SPACING * 2.0);
                    ui.label("Passman helps you securely store and manage all your passwords in one place.");
                    ui.add_space(SPACING);
                    ui.label("• Military-grade AES-256-GCM encryption");
                    ui.label("• Argon2id key derivation for maximum security");
                    ui.label("• Zero-knowledge design - only you have access");
                    ui.add_space(SPACING);
                }
                1 => {
                    ui.heading("🏁 Getting Started");
                    ui.add_space(SPACING * 2.0);
                    ui.label("1. Create a new vault with a strong master password");
                    ui.label("2. Add your passwords with descriptive IDs");
                    ui.label("3. Copy passwords to clipboard with one click");
                    ui.label("4. Use the Health Dashboard to check password strength");
                    ui.add_space(SPACING);
                }
                2 => {
                    ui.heading("⌨️ Quick Tips");
                    ui.add_space(SPACING * 2.0);
                    ui.label("Keyboard shortcuts (when vault is open):");
                    ui.add_space(SPACING / 2.0);
                    ui.label("• Ctrl+N - Create new entry");
                    ui.label("• Ctrl+F - Search entries");
                    ui.label("• Ctrl+L - Lock vault");
                    ui.label("• Ctrl+H - Health dashboard");
                    ui.label("• Escape - Go back");
                    ui.add_space(SPACING);
                }
                _ => {
                    ui.heading("🚀 You're Ready!");
                    ui.add_space(SPACING * 2.0);
                    ui.label("Start by creating a new vault or opening an existing one.");
                    ui.add_space(SPACING);
                    ui.label("Remember: Your master password cannot be recovered!");
                    ui.label("Choose something strong and memorable.");
                    ui.add_space(SPACING);
                }
            }
            
            ui.add_space(SPACING * 2.0);
            
            // Progress dots
            ui.horizontal(|ui| {
                for i in 0..4 {
                    let color = if i == *onboarding_step {
                        egui::Color32::from_rgb(70, 130, 180)
                    } else {
                        egui::Color32::from_gray(150)
                    };
                    ui.painter().circle_filled(
                        ui.cursor().min + egui::vec2(i as f32 * 15.0 + 7.0, 5.0),
                        5.0,
                        color,
                    );
                }
                ui.add_space(60.0);
            });
            
            ui.add_space(SPACING * 2.0);
            
            ui.horizontal(|ui| {
                let btn_width = 90.0;
                
                if *onboarding_step > 0 && ButtonWidgets::secondary(ui, "Back", [btn_width, BUTTON_HEIGHT]).clicked() {
                    *onboarding_step -= 1;
                }
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if *onboarding_step < 3 {
                        if ButtonWidgets::primary(ui, "Next", [btn_width, BUTTON_HEIGHT]).clicked() {
                            *onboarding_step += 1;
                        }
                        
                        if ButtonWidgets::secondary(ui, "Skip", [btn_width, BUTTON_HEIGHT]).clicked() {
                            *show_onboarding = false;
                        }
                    } else if ButtonWidgets::success(ui, "Get Started", [110.0, BUTTON_HEIGHT]).clicked() {
                        *show_onboarding = false;
                    }
                });
            });
        });
}
