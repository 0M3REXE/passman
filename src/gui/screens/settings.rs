//! Settings Screen Module
//!
//! Application settings and configuration.

use eframe::egui;
use crate::vault::VaultManager;
use super::super::types::{Screen, MessageType, SPACING, PADDING, INPUT_WIDTH, BUTTON_HEIGHT};
use super::super::theme;
use super::super::widgets;
use super::super::app::PassmanApp;

impl PassmanApp {
    /// Show settings screen
    pub fn show_settings_screen(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.vertical(|ui| {
            ui.heading("Settings");
        });
        ui.separator();
        ui.add_space(PADDING);

        ui.vertical_centered(|ui| {
            ui.add_space(SPACING * 2.0);
            
            // Theme Toggle Section
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label("🎨 Theme:");
                    ui.add_space(SPACING);
                    
                    let theme_text = match self.current_theme {
                        super::super::types::Theme::Dark => "🌙 Dark",
                        super::super::types::Theme::Light => "☀️ Light",
                    };
                    
                    if self.primary_button(ui, theme_text, [120.0, BUTTON_HEIGHT]).clicked() {
                        self.current_theme = self.current_theme.toggle();
                        theme::apply_theme(&self.current_theme, ctx);
                        
                        // Save theme preference to config
                        {
                            let mut config = crate::config::get_config_mut();
                            config.ui.theme = self.current_theme.name().to_string();
                        }
                        let _ = crate::config::save_config();
                    }
                });
            });
            ui.add_space(SPACING);
            
            ui.horizontal(|ui| {
                ui.label("Vault file:");
                ui.add(egui::TextEdit::singleline(&mut self.vault_file)
                    .desired_width(INPUT_WIDTH));
            });
            ui.add_space(SPACING);

            ui.label("Available vault files:");
            egui::ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
                if let Ok(vaults) = VaultManager::list_vaults() {
                    if vaults.is_empty() {
                        ui.label("No other vault files found in this directory.");
                    } else {
                        for vault_filename in vaults {
                            ui.horizontal(|ui| {
                                ui.label(&vault_filename);
                                if self.vault_file != vault_filename {
                                    if self.primary_button(ui, "Select", [80.0, 28.0]).clicked() {
                                        self.vault_file = vault_filename.clone();
                                        self.show_message(format!("Vault file set to '{}'. Please reopen.", self.vault_file), MessageType::Info);
                                        self.current_screen = Screen::Welcome;
                                    }
                                } else {
                                    ui.label("(current)");
                                }
                            });
                        }
                    }
                } else {
                    ui.label("Could not read vault files.");
                }
            });
            ui.add_space(SPACING * 2.0);
            
            // Password Change Section
            ui.separator();
            ui.add_space(SPACING);
            
            if self.vault.is_some() {
                ui.collapsing("🔐 Change Master Password", |ui| {
                    ui.add_space(SPACING);
                    
                    ui.horizontal(|ui| {
                        ui.label("Current Password:");
                        ui.add(egui::TextEdit::singleline(&mut *self.change_current_password)
                            .password(true)
                            .desired_width(INPUT_WIDTH - 100.0));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("New Password:");
                        ui.add(egui::TextEdit::singleline(&mut *self.change_new_password)
                            .password(!self.show_password_change)
                            .desired_width(INPUT_WIDTH - 100.0));
                    });
                    
                    // Visual password strength indicator for new password
                    if !self.change_new_password.is_empty() {
                        let password_clone = self.change_new_password.to_string();
                        widgets::show_password_strength_indicator(ui, &password_clone);
                    }
                    
                    ui.horizontal(|ui| {
                        ui.label("Confirm Password:");
                        ui.add(egui::TextEdit::singleline(&mut *self.change_confirm_password)
                            .password(!self.show_password_change)
                            .desired_width(INPUT_WIDTH - 100.0));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.show_password_change, "Show passwords");
                    });
                    
                    ui.add_space(SPACING);
                    
                    if self.primary_button(ui, "Change Password", [150.0, BUTTON_HEIGHT]).clicked() {
                        // Validate inputs
                        if self.change_current_password.is_empty() {
                            self.show_message("Current password is required".into(), MessageType::Error);
                        } else if self.change_new_password.is_empty() {
                            self.show_message("New password is required".into(), MessageType::Error);
                        } else if self.change_new_password.len() < 8 {
                            self.show_message("New password must be at least 8 characters".into(), MessageType::Error);
                        } else if self.change_new_password.as_str() != self.change_confirm_password.as_str() {
                            self.show_message("New passwords do not match".into(), MessageType::Error);
                        } else if self.change_current_password.as_str() != self.master_password.as_str() {
                            self.show_message("Current password is incorrect".into(), MessageType::Error);
                        } else {
                            // Attempt to change password
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
                                    self.show_message("Master password changed successfully!".into(), MessageType::Success);
                                }
                                Err(e) => {
                                    self.show_message(format!("Failed to change password: {}", e), MessageType::Error);
                                }
                            }
                        }
                    }
                });
            } else {
                ui.label("Unlock a vault to change the master password.");
            }
            
            ui.add_space(SPACING * 2.0);
            
            if self.secondary_button(ui, "Back", [150.0, BUTTON_HEIGHT]).clicked() {
                self.current_screen = Screen::Welcome;
            }
        });
    }
}
