//! Welcome Screens Module
//!
//! Welcome, Init (create vault), and Login screens.

use eframe::egui;
use crate::vault::VaultManager;
use crate::config::get_config;
use super::super::types::{Screen, MessageType, SPACING, PADDING, INPUT_WIDTH, BUTTON_HEIGHT};
use super::super::app::PassmanApp;

impl PassmanApp {
    /// Show welcome/home screen
    pub fn show_welcome_screen(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading("Passman");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("❓ Help").clicked() {
                        self.show_onboarding = true;
                        self.onboarding_step = 0;
                    }
                });
            });
            ui.label("Password Manager");
        });
        ui.separator();
        ui.add_space(PADDING);

        ui.vertical_centered(|ui| {
            ui.add_space(SPACING * 2.0);

            ui.horizontal(|ui| {
                ui.label("Vault file:");
                ui.add(egui::TextEdit::singleline(&mut self.vault_file)
                    .desired_width(200.0));
            });
            ui.add_space(SPACING);

            let vault_exists = VaultManager::exists(Some(&self.vault_file));
            if vault_exists {
                if self.primary_button(ui, "Open Vault", [150.0, BUTTON_HEIGHT]).clicked() {
                    self.current_screen = Screen::Login;
                    self.clear_message();
                }
            } else if self.success_button(ui, "Create Vault", [150.0, BUTTON_HEIGHT]).clicked() {
                self.current_screen = Screen::Init;
                self.clear_message();
            }
            ui.add_space(SPACING);
            if self.secondary_button(ui, "Settings", [150.0, BUTTON_HEIGHT]).clicked() {
                self.current_screen = Screen::Settings;
            }
            
            // Error recovery section
            ui.add_space(SPACING * 3.0);
            ui.separator();
            ui.add_space(SPACING);
            
            ui.collapsing("🔧 Troubleshooting", |ui| {
                ui.add_space(SPACING / 2.0);
                ui.label("Having issues? Try these options:");
                ui.add_space(SPACING / 2.0);
                
                if ui.small_button("📂 Browse for vault file").clicked() {
                    self.toast_info("Enter the full path to your vault file above");
                }
                
                if ui.small_button("🔄 Reset to default vault location").clicked() {
                    let config = get_config();
                    self.vault_file = config.general.default_vault.clone();
                    self.toast_success("Reset to default vault location");
                }
                
                if ui.small_button("📋 Show vault file path").clicked() {
                    let path = std::path::Path::new(&self.vault_file);
                    let absolute = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
                    self.toast_info(format!("Vault: {}", absolute.display()));
                }
                
                ui.add_space(SPACING / 2.0);
                ui.label("⚠️ If you forgot your master password, the vault cannot be recovered.");
            });
        });
    }

    /// Show vault initialization screen
    pub fn show_init_screen(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Create New Vault");
        });
        ui.separator();
        ui.add_space(PADDING);

        ui.vertical_centered(|ui| {
            ui.add_space(SPACING * 2.0);
            
            ui.horizontal(|ui| {
                ui.label("Master Password:");
                ui.add(egui::TextEdit::singleline(&mut *self.init_password)
                    .password(true)
                    .desired_width(INPUT_WIDTH));
            });
            ui.add_space(SPACING);

            ui.horizontal(|ui| {
                ui.label("Confirm Password:");
                ui.add(egui::TextEdit::singleline(&mut *self.init_confirm)
                    .password(true)
                    .desired_width(INPUT_WIDTH));
            });
            ui.add_space(SPACING * 2.0);
            
            if self.success_button(ui, "Create", [150.0, BUTTON_HEIGHT]).clicked() {
                match self.init_vault() {
                    Ok(()) => {
                        self.show_message("Vault created successfully!".to_string(), MessageType::Success);
                    }
                    Err(e) => {
                        self.show_message(e, MessageType::Error);
                    }
                }
            }
            ui.add_space(SPACING);
            
            if self.secondary_button(ui, "Cancel", [150.0, BUTTON_HEIGHT]).clicked() {
                self.current_screen = Screen::Welcome;
                *self.init_password = String::new();
                *self.init_confirm = String::new();
                self.clear_message();
            }
        });
    }

    /// Show login screen
    pub fn show_login_screen(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Open Vault");
        });
        ui.separator();
        ui.add_space(PADDING);
        
        ui.vertical_centered(|ui| {
            ui.add_space(SPACING * 2.0);
            
            ui.horizontal(|ui| {
                ui.label("Master Password:");
                ui.add(egui::TextEdit::singleline(&mut *self.login_password)
                    .password(true)
                    .desired_width(INPUT_WIDTH));
            });
            ui.add_space(SPACING * 2.0);
            
            if self.primary_button(ui, "Open", [150.0, BUTTON_HEIGHT]).clicked() {
                match self.login() {
                    Ok(()) => {
                        self.show_message("Vault opened successfully!".to_string(), MessageType::Success);
                    }
                    Err(e) => {
                        self.show_message(e, MessageType::Error);
                    }
                }
            }
            ui.add_space(SPACING);
            
            if self.secondary_button(ui, "Cancel", [150.0, BUTTON_HEIGHT]).clicked() {
                self.current_screen = Screen::Welcome;
                *self.login_password = String::new();
                self.clear_message();
            }
        });
    }
}
