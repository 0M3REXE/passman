//! Health Dashboard Screen Module
//!
//! Password health analysis and recommendations.

use eframe::egui;
use super::super::types::{Screen, SPACING, PADDING, BUTTON_HEIGHT};
use super::super::app::PassmanApp;

impl PassmanApp {
    /// Show password health dashboard
    pub fn show_health_dashboard(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Password Health Dashboard");
        });
        ui.separator();
        ui.add_space(PADDING);

        ui.vertical_centered(|ui| {
            ui.add_space(SPACING * 2.0);
            
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
            
            ui.add_space(SPACING * 2.0);
            
            if self.secondary_button(ui, "Back", [150.0, BUTTON_HEIGHT]).clicked() {
                self.current_screen = Screen::Main;
            }
        });
    }
}
