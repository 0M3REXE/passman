//! Health Dashboard Screen Module
//!
//! Password health analysis and recommendations.

use eframe::egui;
use super::super::types::{Screen, SPACING};
use super::super::theme;
use super::super::app::PassmanApp;

impl PassmanApp {
    /// Show password health dashboard
    pub fn show_health_dashboard(&mut self, ui: &mut egui::Ui) {
        let current_theme = self.current_theme.clone();
        let border_color = theme::border_color(&current_theme);
        
        // ════════════════════════════════════════════════════════════════════
        // HEADER BAR
        // ════════════════════════════════════════════════════════════════════
        egui::Frame::none()
            .fill(theme::header_bg_color(&current_theme))
            .inner_margin(egui::Margin::symmetric(16.0, 12.0))
            .rounding(egui::Rounding::same(10.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🏥").size(20.0));
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Health Dashboard").size(18.0).strong());
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let back_btn = egui::Button::new("Back")
                            .fill(egui::Color32::from_rgb(55, 65, 81))
                            .stroke(egui::Stroke::new(1.0, border_color))
                            .rounding(egui::Rounding::same(6.0))
                            .min_size(egui::vec2(70.0, 28.0));
                        
                        if ui.add(back_btn).clicked() {
                            self.current_screen = Screen::Main;
                        }
                    });
                });
            });
        
        ui.add_space(SPACING);

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.vertical_centered(|ui| {
            
            // Generate health summary if we have a vault
            if let Some(vault) = &self.vault {
                let reports = self.health_analyzer.analyze_vault(vault);
                let summary = self.health_analyzer.generate_summary(&reports);
                
                ui.label(format!("Overall Health: {:.1}%", summary.score));
                ui.add(egui::ProgressBar::new(summary.score as f32 / 100.0)
                    .text(format!("{:.1}%", summary.score)));
                
                ui.separator();
                
                // Show health distribution
                ui.horizontal(|ui| {
                    ui.label("Critical:");
                    ui.colored_label(egui::Color32::RED, format!("{}", summary.critical));
                    ui.label("Warning:");
                    ui.colored_label(egui::Color32::YELLOW, format!("{}", summary.warning));
                    ui.label("Good:");
                    ui.colored_label(egui::Color32::LIGHT_GREEN, format!("{}", summary.good));
                    ui.label("Excellent:");
                    ui.colored_label(egui::Color32::GREEN, format!("{}", summary.excellent));
                });
                
                ui.add_space(SPACING * 2.0);
                
                // Show individual entry health
                ui.label("Entry Details:");
                egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                    for report in &reports {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(&report.entry_id);
                                let health_text = match &report.health {
                                    crate::health::PasswordHealth::Excellent => "Excellent",
                                    crate::health::PasswordHealth::Good => "Good", 
                                    crate::health::PasswordHealth::Warning { .. } => "Warning",
                                    crate::health::PasswordHealth::Critical { .. } => "Critical",
                                };
                                let color = match &report.health {
                                    crate::health::PasswordHealth::Excellent => egui::Color32::GREEN,
                                    crate::health::PasswordHealth::Good => egui::Color32::LIGHT_GREEN,
                                    crate::health::PasswordHealth::Warning { .. } => egui::Color32::YELLOW,
                                    crate::health::PasswordHealth::Critical { .. } => egui::Color32::RED,
                                };
                                ui.colored_label(color, health_text);
                                ui.label(format!("Age: {} days", report.age_days));
                            });
                        });
                    }
                });
            } else {
                ui.label("No health data available. Please add entries to analyze.");
            }
            });
        });
    }
}
