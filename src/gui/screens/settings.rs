//! Settings Screen Module
//!
//! Application settings and configuration.

use eframe::egui;
use crate::vault::VaultManager;
use super::super::types::{Screen, SPACING};
use super::super::theme;
use super::super::widgets;
use super::super::app::PassmanApp;

impl PassmanApp {
    /// Show settings screen
    pub fn show_settings_screen(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        let current_theme = self.current_theme.clone();
        let muted_color = theme::muted_text_color(&current_theme);
        let frame_fill = theme::frame_fill(&current_theme);
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
                    ui.label(egui::RichText::new("⚙").size(20.0));
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Settings").size(18.0).strong());
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let back_btn = egui::Button::new("Back")
                            .fill(egui::Color32::from_rgb(55, 65, 81))
                            .stroke(egui::Stroke::new(1.0, border_color))
                            .rounding(egui::Rounding::same(6.0))
                            .min_size(egui::vec2(70.0, 28.0));
                        
                        if ui.add(back_btn).clicked() {
                            if self.vault.is_some() {
                                self.current_screen = Screen::Main;
                            } else {
                                self.current_screen = Screen::Welcome;
                            }
                        }
                    });
                });
            });
        
        ui.add_space(SPACING);

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                let card_width = 400.0;
                let field_width = 280.0;
                
                // ════════════════════════════════════════════════════════════════
                // VAULT SECTION
                // ════════════════════════════════════════════════════════════════
                egui::Frame::none()
                    .fill(frame_fill)
                    .stroke(egui::Stroke::new(1.0, border_color))
                    .rounding(egui::Rounding::same(12.0))
                    .inner_margin(egui::Margin::same(20.0))
                    .show(ui, |ui| {
                        ui.set_width(card_width);
                        
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Vault").size(14.0).strong());
                        });
                        
                        ui.add_space(12.0);
                        
                        // Current vault file
                        ui.horizontal(|ui| {
                            ui.label("Current:");
                            ui.add_space(8.0);
                            ui.add_sized(
                                egui::vec2(field_width, 24.0),
                                egui::TextEdit::singleline(&mut self.vault_file)
                                    .interactive(false)
                            );
                        });
                        
                        ui.add_space(12.0);
                        
                        // Available vaults
                        ui.label(
                            egui::RichText::new("Available vault files:")
                                .size(12.0)
                                .color(muted_color)
                        );
                        ui.add_space(6.0);
                        
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 30))
                            .rounding(egui::Rounding::same(6.0))
                            .inner_margin(egui::Margin::same(8.0))
                            .show(ui, |ui| {
                                egui::ScrollArea::vertical().max_height(80.0).show(ui, |ui| {
                                    if let Ok(vaults) = VaultManager::list_vaults() {
                                        if vaults.is_empty() {
                                            ui.label(
                                                egui::RichText::new("No vault files found")
                                                    .size(12.0)
                                                    .color(muted_color)
                                            );
                                        } else {
                                            for vault_filename in vaults {
                                                ui.horizontal(|ui| {
                                                    let is_current = self.vault_file == vault_filename;
                                                    
                                                    ui.label(&vault_filename);
                                                    
                                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                        if is_current {
                                                            ui.label(
                                                                egui::RichText::new("current")
                                                                    .size(11.0)
                                                                    .color(egui::Color32::from_rgb(34, 197, 94))
                                                            );
                                                        } else {
                                                            let select_btn = egui::Button::new(
                                                                egui::RichText::new("Select").size(11.0)
                                                            )
                                                            .rounding(egui::Rounding::same(4.0))
                                                            .min_size(egui::vec2(50.0, 22.0));
                                                            
                                                            if ui.add(select_btn).clicked() {
                                                                self.vault_file = vault_filename.clone();
                                                                self.toast_info(format!("Vault file set to '{}'. Please reopen.", self.vault_file));
                                                                self.current_screen = Screen::Welcome;
                                                            }
                                                        }
                                                    });
                                                });
                                            }
                                        }
                                    } else {
                                        ui.label("Could not read vault files.");
                                    }
                                });
                            });
                    });
                
                ui.add_space(16.0);
                
                // ════════════════════════════════════════════════════════════════
                // PASSWORD CHANGE SECTION
                // ════════════════════════════════════════════════════════════════
                if self.vault.is_some() {
                    egui::Frame::none()
                        .fill(frame_fill)
                        .stroke(egui::Stroke::new(1.0, border_color))
                        .rounding(egui::Rounding::same(12.0))
                        .inner_margin(egui::Margin::same(20.0))
                        .show(ui, |ui| {
                            ui.set_width(card_width);
                            
                            egui::CollapsingHeader::new(
                                egui::RichText::new("Change Master Password").size(14.0).strong()
                            )
                            .default_open(false)
                            .show(ui, |ui| {
                                ui.add_space(12.0);
                                
                                let label_width = 120.0;
                                
                                // Current password
                                ui.horizontal(|ui| {
                                    ui.allocate_ui_with_layout(
                                        egui::vec2(label_width, 24.0),
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| { ui.label("Current:"); }
                                    );
                                    ui.add_sized(
                                        egui::vec2(200.0, 24.0),
                                        egui::TextEdit::singleline(&mut *self.change_current_password)
                                            .password(true)
                                    );
                                });
                                
                                ui.add_space(8.0);
                                
                                // New password
                                ui.horizontal(|ui| {
                                    ui.allocate_ui_with_layout(
                                        egui::vec2(label_width, 24.0),
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| { ui.label("New:"); }
                                    );
                                    ui.add_sized(
                                        egui::vec2(200.0, 24.0),
                                        egui::TextEdit::singleline(&mut *self.change_new_password)
                                            .password(!self.show_password_change)
                                    );
                                });
                                
                                // Strength indicator
                                if !self.change_new_password.is_empty() {
                                    ui.horizontal(|ui| {
                                        ui.add_space(label_width + 8.0);
                                        let password_clone = self.change_new_password.to_string();
                                        widgets::show_password_strength_indicator(ui, &password_clone);
                                    });
                                }
                                
                                ui.add_space(8.0);
                                
                                // Confirm password
                                ui.horizontal(|ui| {
                                    ui.allocate_ui_with_layout(
                                        egui::vec2(label_width, 24.0),
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| { ui.label("Confirm:"); }
                                    );
                                    ui.add_sized(
                                        egui::vec2(200.0, 24.0),
                                        egui::TextEdit::singleline(&mut *self.change_confirm_password)
                                            .password(!self.show_password_change)
                                    );
                                });
                                
                                ui.add_space(8.0);
                                
                                ui.horizontal(|ui| {
                                    ui.add_space(label_width + 8.0);
                                    ui.checkbox(&mut self.show_password_change, "Show passwords");
                                });
                                
                                ui.add_space(16.0);
                                
                                ui.horizontal(|ui| {
                                    ui.add_space(label_width + 8.0);
                                    
                                    let change_btn = egui::Button::new(
                                        egui::RichText::new("Change Password").color(egui::Color32::WHITE)
                                    )
                                    .fill(egui::Color32::from_rgb(59, 130, 246))
                                    .rounding(egui::Rounding::same(8.0))
                                    .min_size(egui::vec2(140.0, 32.0));
                                    
                                    if ui.add(change_btn).clicked() {
                                        if self.change_current_password.is_empty() {
                                            self.toast_error("Current password is required");
                                        } else if self.change_new_password.is_empty() {
                                            self.toast_error("New password is required");
                                        } else if self.change_new_password.len() < 8 {
                                            self.toast_error("New password must be at least 8 characters");
                                        } else if self.change_new_password.as_str() != self.change_confirm_password.as_str() {
                                            self.toast_error("New passwords do not match");
                                        } else if self.change_current_password.as_str() != self.master_password.as_str() {
                                            self.toast_error("Current password is incorrect");
                                        } else {
                                            match VaultManager::change_password(
                                                &self.change_current_password,
                                                &self.change_new_password,
                                                Some(&self.vault_file)
                                            ) {
                                                Ok(()) => {
                                                    *self.master_password = self.change_new_password.to_string();
                                                    *self.change_current_password = String::new();
                                                    *self.change_new_password = String::new();
                                                    *self.change_confirm_password = String::new();
                                                    self.toast_success("Master password changed successfully!");
                                                }
                                                Err(e) => {
                                                    self.toast_error(format!("Failed to change password: {}", e));
                                                }
                                            }
                                        }
                                    }
                                });
                            });
                        });
                    
                    ui.add_space(16.0);
                }
                
                ui.add_space(SPACING * 2.0);
            });
        });
    }
}
