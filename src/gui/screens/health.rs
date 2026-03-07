//! Health Dashboard Screen Module
//!
//! Password health analysis and recommendations.

use eframe::egui;
use super::super::types::Screen;
use super::super::theme;
use super::super::app::PassmanApp;

impl PassmanApp {
    /// Show password health dashboard
    pub fn show_health_dashboard(&mut self, ui: &mut egui::Ui) {
        let current_theme = self.current_theme;
        let border_color = theme::border_color(&current_theme);
        let frame_fill = theme::frame_fill(&current_theme);
        let muted_color = theme::muted_text_color(&current_theme);
        
        // ════════════════════════════════════════════════════════════════════
        // HEADER BAR
        // ════════════════════════════════════════════════════════════════════
        egui::Frame::none()
            .fill(theme::header_bg_color(&current_theme))
            .inner_margin(egui::Margin::symmetric(20.0, 14.0))
            .rounding(egui::Rounding::same(14.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🏥").size(22.0));
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("Health Dashboard").size(20.0).strong());
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let back_btn = egui::Button::new(
                            egui::RichText::new("← Back").size(13.0)
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .stroke(egui::Stroke::new(1.0, border_color))
                        .rounding(egui::Rounding::same(10.0))
                        .min_size(egui::vec2(80.0, 34.0));
                        
                        if ui.add(back_btn).clicked() {
                            self.current_screen = Screen::Main;
                        }
                    });
                });
            });
        
        ui.add_space(16.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.vertical_centered(|ui| {
            
            // Generate health summary if we have a vault
            if let Some(vault) = &self.vault {
                let reports = self.health_analyzer.analyze_vault(vault);
                let summary = self.health_analyzer.generate_summary(&reports);
                
                // Overall score card
                egui::Frame::none()
                    .fill(frame_fill)
                    .stroke(egui::Stroke::new(1.0, border_color))
                    .rounding(egui::Rounding::same(16.0))
                    .inner_margin(egui::Margin::same(24.0))
                    .show(ui, |ui| {
                        ui.set_width(440.0);
                        
                        ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::new("Overall Health").size(15.0).strong());
                            ui.add_space(10.0);
                            ui.label(egui::RichText::new(format!("{:.0}%", summary.score)).size(36.0).strong());
                            ui.add_space(8.0);
                            ui.add(egui::ProgressBar::new(summary.score as f32 / 100.0)
                                .text(format!("{:.1}%", summary.score)));
                        });
                        
                        ui.add_space(18.0);
                        
                        // Distribution badges
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 16.0;
                            ui.colored_label(egui::Color32::from_rgb(239, 68, 68), format!("● Critical {}", summary.critical));
                            ui.colored_label(egui::Color32::from_rgb(251, 191, 36), format!("● Warning {}", summary.warning));
                            ui.colored_label(egui::Color32::from_rgb(34, 197, 94), format!("● Good {}", summary.good));
                            ui.colored_label(egui::Color32::from_rgb(16, 185, 129), format!("● Excellent {}", summary.excellent));
                        });
                    });
                
                ui.add_space(16.0);
                
                // Entry details card
                egui::Frame::none()
                    .fill(frame_fill)
                    .stroke(egui::Stroke::new(1.0, border_color))
                    .rounding(egui::Rounding::same(16.0))
                    .inner_margin(egui::Margin::same(24.0))
                    .show(ui, |ui| {
                        ui.set_width(440.0);
                        
                        ui.label(egui::RichText::new("Entry Details").size(15.0).strong());
                        ui.add_space(12.0);
                        
                        egui::ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
                            for report in &reports {
                                egui::Frame::none()
                                    .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 20))
                                    .rounding(egui::Rounding::same(10.0))
                                    .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new(&report.entry_id).size(13.0).strong());
                                            
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                ui.label(egui::RichText::new(format!("{} days", report.age_days)).size(11.0).color(muted_color));
                                                ui.add_space(10.0);
                                                let (health_text, color) = match &report.health {
                                                    crate::utils::health::PasswordHealth::Excellent => ("Excellent", egui::Color32::from_rgb(16, 185, 129)),
                                                    crate::utils::health::PasswordHealth::Good => ("Good", egui::Color32::from_rgb(34, 197, 94)),
                                                    crate::utils::health::PasswordHealth::Warning { .. } => ("Warning", egui::Color32::from_rgb(251, 191, 36)),
                                                    crate::utils::health::PasswordHealth::Critical { .. } => ("Critical", egui::Color32::from_rgb(239, 68, 68)),
                                                };
                                                ui.label(egui::RichText::new(health_text).size(12.0).color(color));
                                            });
                                        });
                                    });
                                ui.add_space(6.0);
                            }
                        });
                    });
            } else {
                ui.add_space(40.0);
                ui.label(egui::RichText::new("📊").size(48.0));
                ui.add_space(12.0);
                ui.label(egui::RichText::new("No health data available").size(16.0).strong());
                ui.add_space(4.0);
                ui.label(egui::RichText::new("Add entries to analyze password health").size(13.0).color(muted_color));
            }
            });
        });
    }
}
