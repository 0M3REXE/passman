//! Settings Screen Module
//!
//! Application settings and configuration.

use eframe::egui;
use crate::security::vault::VaultManager;
use super::super::types::Screen;
use super::super::theme;
use super::super::widgets;
use super::super::app::PassmanApp;

impl PassmanApp {
    /// Show settings screen
    pub fn show_settings_screen(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        let current_theme = self.current_theme;
        let muted_color = theme::muted_text_color(&current_theme);
        let frame_fill = theme::frame_fill(&current_theme);
        let border_color = theme::border_color(&current_theme);
        let dim_color = theme::dim_text_color(&current_theme);
        
        // ════════════════════════════════════════════════════════════════════
        // HEADER BAR
        // ════════════════════════════════════════════════════════════════════
        egui::Frame::none()
            .fill(theme::header_bg_color(&current_theme))
            .inner_margin(egui::Margin::symmetric(20.0, 14.0))
            .rounding(egui::Rounding::same(14.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("⚙").size(22.0));
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("Settings").size(20.0).strong());
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let back_btn = egui::Button::new(
                            egui::RichText::new("← Back").size(13.0)
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .stroke(egui::Stroke::new(1.0, border_color))
                        .rounding(egui::Rounding::same(10.0))
                        .min_size(egui::vec2(80.0, 34.0));
                        
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
        
        ui.add_space(16.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                let card_width = 440.0;
                let field_width = 320.0;
                
                // ════════════════════════════════════════════════════════════════
                // VAULT SECTION
                // ════════════════════════════════════════════════════════════════
                egui::Frame::none()
                    .fill(frame_fill)
                    .stroke(egui::Stroke::new(1.0, border_color))
                    .rounding(egui::Rounding::same(16.0))
                    .inner_margin(egui::Margin::same(24.0))
                    .show(ui, |ui| {
                        ui.set_width(card_width);
                        
                        ui.label(egui::RichText::new("📁 Vault").size(15.0).strong());
                        ui.add_space(14.0);
                        
                        // Current vault file
                        ui.label(egui::RichText::new("Current vault").size(13.0).strong());
                        ui.add_space(6.0);
                        ui.add_sized(
                            egui::vec2(field_width, 36.0),
                            egui::TextEdit::singleline(&mut self.vault_file)
                                .interactive(false)
                                .margin(egui::Margin::symmetric(14.0, 8.0))
                        );
                        
                        ui.add_space(16.0);
                        
                        // Available vaults
                        ui.label(
                            egui::RichText::new("Available vault files")
                                .size(12.0)
                                .color(muted_color)
                        );
                        ui.add_space(8.0);
                        
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 30))
                            .rounding(egui::Rounding::same(10.0))
                            .inner_margin(egui::Margin::same(12.0))
                            .show(ui, |ui| {
                                egui::ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
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
                                                    
                                                    ui.label(
                                                        egui::RichText::new(&vault_filename).size(13.0)
                                                    );
                                                    
                                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                        if is_current {
                                                            ui.label(
                                                                egui::RichText::new("● current")
                                                                    .size(11.0)
                                                                    .color(egui::Color32::from_rgb(34, 197, 94))
                                                            );
                                                        } else {
                                                            let select_btn = egui::Button::new(
                                                                egui::RichText::new("Select").size(11.0)
                                                            )
                                                            .rounding(egui::Rounding::same(8.0))
                                                            .min_size(egui::vec2(56.0, 26.0));
                                                            
                                                            if ui.add(select_btn).clicked() {
                                                                self.vault_file = vault_filename.clone();
                                                                self.toast_info(format!("Vault file set to '{}'. Please reopen.", self.vault_file));
                                                                self.current_screen = Screen::Welcome;
                                                            }
                                                        }
                                                    });
                                                });
                                                ui.add_space(2.0);
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
                        .rounding(egui::Rounding::same(16.0))
                        .inner_margin(egui::Margin::same(24.0))
                        .show(ui, |ui| {
                            ui.set_width(card_width);
                            
                            egui::CollapsingHeader::new(
                                egui::RichText::new("🔑 Change Master Password").size(15.0).strong()
                            )
                            .default_open(false)
                            .show(ui, |ui| {
                                ui.add_space(14.0);
                                
                                let pw_field_width = 280.0;
                                let pw_field_height = 36.0;
                                
                                // Current password
                                ui.label(egui::RichText::new("Current Password").size(13.0).strong());
                                ui.add_space(6.0);
                                ui.add_sized(
                                    egui::vec2(pw_field_width, pw_field_height),
                                    egui::TextEdit::singleline(&mut *self.change_current_password)
                                        .password(true)
                                        .hint_text(egui::RichText::new("Enter current password").color(dim_color))
                                        .margin(egui::Margin::symmetric(14.0, 8.0))
                                );
                                ui.add_space(12.0);
                                
                                // New password
                                ui.label(egui::RichText::new("New Password").size(13.0).strong());
                                ui.add_space(6.0);
                                ui.add_sized(
                                    egui::vec2(pw_field_width, pw_field_height),
                                    egui::TextEdit::singleline(&mut *self.change_new_password)
                                        .password(!self.show_password_change)
                                        .hint_text(egui::RichText::new("Enter new password").color(dim_color))
                                        .margin(egui::Margin::symmetric(14.0, 8.0))
                                );
                                
                                // Strength indicator
                                if !self.change_new_password.is_empty() {
                                    ui.add_space(6.0);
                                    let password_clone = self.change_new_password.to_string();
                                    widgets::show_password_strength_indicator(ui, &password_clone);
                                }
                                ui.add_space(12.0);
                                
                                // Confirm password
                                ui.label(egui::RichText::new("Confirm Password").size(13.0).strong());
                                ui.add_space(6.0);
                                ui.add_sized(
                                    egui::vec2(pw_field_width, pw_field_height),
                                    egui::TextEdit::singleline(&mut *self.change_confirm_password)
                                        .password(!self.show_password_change)
                                        .hint_text(egui::RichText::new("Confirm new password").color(dim_color))
                                        .margin(egui::Margin::symmetric(14.0, 8.0))
                                );
                                ui.add_space(10.0);
                                
                                ui.checkbox(&mut self.show_password_change, "Show passwords");
                                
                                ui.add_space(18.0);
                                
                                let change_btn = egui::Button::new(
                                    egui::RichText::new("Change Password")
                                        .size(14.0)
                                        .color(egui::Color32::WHITE)
                                )
                                .fill(egui::Color32::from_rgb(79, 120, 247))
                                .rounding(egui::Rounding::same(10.0))
                                .min_size(egui::vec2(160.0, 40.0));
                                
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
                    
                    ui.add_space(16.0);
                }
                
                // ════════════════════════════════════════════════════════════════
                // ABOUT SECTION
                // ════════════════════════════════════════════════════════════════
                egui::Frame::none()
                    .fill(frame_fill)
                    .stroke(egui::Stroke::new(1.0, border_color))
                    .rounding(egui::Rounding::same(16.0))
                    .inner_margin(egui::Margin::same(24.0))
                    .show(ui, |ui| {
                        ui.set_width(card_width);
                        
                        ui.label(egui::RichText::new("ℹ About").size(15.0).strong());
                        ui.add_space(14.0);
                        
                        ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::new("Passman").size(18.0).strong());
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                                    .size(12.0)
                                    .color(muted_color)
                            );
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new("A secure password manager")
                                    .size(13.0)
                                    .color(muted_color)
                            );
                            ui.add_space(14.0);
                            
                            ui.label(
                                egui::RichText::new("Created by 0M3REXE")
                                    .size(13.0)
                            );
                            ui.add_space(8.0);
                            
                            if ui.link(
                                egui::RichText::new("github.com/0M3REXE/passman")
                                    .color(theme::accent_color(&current_theme))
                            ).clicked() {
                                let _ = open::that("https://github.com/0M3REXE/passman");
                            }
                        });
                    });
                
                ui.add_space(24.0);
            });
        });
    }
}
