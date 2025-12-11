//! Welcome Screens Module
//!
//! Welcome, Init (create vault), and Login screens.

use eframe::egui;
use crate::vault::VaultManager;
use super::super::types::Screen;
use super::super::theme;
use super::super::app::PassmanApp;

impl PassmanApp {
    /// Show welcome/home screen
    pub fn show_welcome_screen(&mut self, ui: &mut egui::Ui) {
        let current_theme = self.current_theme.clone();
        let muted_color = theme::muted_text_color(&current_theme);
        let frame_fill = theme::frame_fill(&current_theme);
        let border_color = theme::border_color(&current_theme);
        
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            
            // ════════════════════════════════════════════════════════════════
            // LOGO / BRANDING
            // ════════════════════════════════════════════════════════════════
            ui.label(egui::RichText::new("🔐").size(64.0));
            ui.add_space(12.0);
            ui.label(egui::RichText::new("Passman").size(32.0).strong());
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Secure Password Manager")
                    .size(14.0)
                    .color(muted_color)
            );
            
            ui.add_space(40.0);
            
            // ════════════════════════════════════════════════════════════════
            // MAIN CARD
            // ════════════════════════════════════════════════════════════════
            egui::Frame::none()
                .fill(frame_fill)
                .stroke(egui::Stroke::new(1.0, border_color))
                .rounding(egui::Rounding::same(16.0))
                .inner_margin(egui::Margin::same(32.0))
                .show(ui, |ui| {
                    ui.set_width(320.0);
                    
                    // Vault file selection
                    ui.vertical_centered(|ui| {
                        ui.label(egui::RichText::new("Vault Location").size(13.0).strong());
                    });
                    ui.add_space(8.0);
                    
                    ui.vertical_centered(|ui| {
                        let btn_width = 260.0;
                        let field_height = 32.0;
                        let browse_btn_size = 36.0;
                        let gap = 4.0;
                        let field_width = btn_width - browse_btn_size - gap;
                        
                        ui.allocate_ui_with_layout(
                            egui::vec2(btn_width, field_height),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                ui.add_sized(
                                    egui::vec2(field_width, field_height),
                                    egui::TextEdit::singleline(&mut self.vault_file)
                                        .hint_text("vault.dat")
                                );
                                
                                if ui.add_sized(
                                    egui::vec2(browse_btn_size, field_height),
                                    egui::Button::new("📁")
                                        .fill(egui::Color32::TRANSPARENT)
                                        .stroke(egui::Stroke::new(1.0, border_color))
                                        .rounding(egui::Rounding::same(6.0))
                                ).on_hover_text("Browse for vault file").clicked() {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .set_title("Select Vault File")
                                        .add_filter("Vault files", &["dat"])
                                        .add_filter("All files", &["*"])
                                        .pick_file()
                                    {
                                        self.vault_file = path.display().to_string();
                                    }
                                }
                            }
                        );
                    });
                    
                    ui.add_space(24.0);
                    
                    // Action buttons
                    let vault_exists = VaultManager::exists(Some(&self.vault_file));
                    
                    ui.vertical_centered(|ui| {
                        let btn_width = 260.0;
                        let btn_height = 44.0;
                        
                        if vault_exists {
                            // Open existing vault (primary)
                            let open_btn = egui::Button::new(
                                egui::RichText::new("Open Vault").size(14.0).color(egui::Color32::WHITE)
                            )
                            .fill(egui::Color32::from_rgb(59, 130, 246))
                            .rounding(egui::Rounding::same(10.0))
                            .min_size(egui::vec2(btn_width, btn_height));
                            
                            if ui.add(open_btn).clicked() {
                                self.current_screen = Screen::Login;
                            }
                            
                            ui.add_space(12.0);
                            
                            // Create new vault (secondary)
                            let create_btn = egui::Button::new(
                                egui::RichText::new("Create New Vault").size(14.0)
                            )
                            .fill(egui::Color32::TRANSPARENT)
                            .stroke(egui::Stroke::new(1.0, border_color))
                            .rounding(egui::Rounding::same(10.0))
                            .min_size(egui::vec2(btn_width, btn_height));
                            
                            if ui.add(create_btn).clicked() {
                                self.current_screen = Screen::Init;
                            }
                        } else {
                            // Create new vault (primary)
                            let create_btn = egui::Button::new(
                                egui::RichText::new("Create Vault").size(14.0).color(egui::Color32::WHITE)
                            )
                            .fill(egui::Color32::from_rgb(34, 197, 94))
                            .rounding(egui::Rounding::same(10.0))
                            .min_size(egui::vec2(btn_width, btn_height));
                            
                            if ui.add(create_btn).clicked() {
                                self.current_screen = Screen::Init;
                            }
                            
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new("No vault found at this location")
                                    .size(11.0)
                                    .color(muted_color)
                            );
                        }
                    });
                });
            
            ui.add_space(16.0);
            
            // ════════════════════════════════════════════════════════════════
            // BOTTOM ACTIONS
            // ════════════════════════════════════════════════════════════════
            ui.horizontal(|ui| {
                // Settings button
                if ui.add(
                    egui::Button::new(egui::RichText::new("⚙ Settings").size(12.0).color(muted_color))
                        .fill(egui::Color32::TRANSPARENT)
                        .stroke(egui::Stroke::NONE)
                ).clicked() {
                    self.current_screen = Screen::Settings;
                }
                
                ui.add_space(12.0);
                
                // Help button
                if ui.add(
                    egui::Button::new(egui::RichText::new("❓ Help").size(12.0).color(muted_color))
                        .fill(egui::Color32::TRANSPARENT)
                        .stroke(egui::Stroke::NONE)
                ).clicked() {
                    self.show_onboarding = true;
                    self.onboarding_step = 0;
                }
                
                ui.add_space(12.0);
                
                // Troubleshooting menu
                ui.menu_button(
                    egui::RichText::new("🔧 Troubleshoot").size(12.0).color(muted_color),
                    |ui| {
                        if ui.button("Reset to default vault").clicked() {
                            self.vault_file = "vault.dat".to_string();
                            self.toast_success("Reset to default vault location");
                            ui.close_menu();
                        }
                        if ui.button("Show full path").clicked() {
                            let path = std::path::Path::new(&self.vault_file);
                            let absolute = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
                            self.toast_info(format!("Vault: {}", absolute.display()));
                            ui.close_menu();
                        }
                        ui.separator();
                        ui.label(
                            egui::RichText::new("⚠ Forgot password = lost vault")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(251, 191, 36))
                        );
                    }
                );
            });
            
            // Version info at bottom
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("v1.0.0")
                    .size(11.0)
                    .color(egui::Color32::from_rgb(80, 80, 85))
            );
        });
    }

    /// Show vault initialization screen
    pub fn show_init_screen(&mut self, ui: &mut egui::Ui) {
        let current_theme = self.current_theme.clone();
        let muted_color = theme::muted_text_color(&current_theme);
        let frame_fill = theme::frame_fill(&current_theme);
        let border_color = theme::border_color(&current_theme);
        
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            
            // Header
            ui.label(egui::RichText::new("✨").size(48.0));
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Create New Vault").size(24.0).strong());
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Set up a secure password to protect your vault")
                    .size(13.0)
                    .color(muted_color)
            );
            
            ui.add_space(32.0);
            
            // Form card
            egui::Frame::none()
                .fill(frame_fill)
                .stroke(egui::Stroke::new(1.0, border_color))
                .rounding(egui::Rounding::same(16.0))
                .inner_margin(egui::Margin::same(32.0))
                .show(ui, |ui| {
                    ui.set_width(320.0);
                    
                    // Master password
                    ui.label(egui::RichText::new("Master Password").size(13.0).strong());
                    ui.add_space(6.0);
                    ui.add(
                        egui::TextEdit::singleline(&mut *self.init_password)
                            .password(true)
                            .hint_text("Enter a strong password")
                            .desired_width(280.0)
                    );
                    
                    ui.add_space(16.0);
                    
                    // Confirm password
                    ui.label(egui::RichText::new("Confirm Password").size(13.0).strong());
                    ui.add_space(6.0);
                    ui.add(
                        egui::TextEdit::singleline(&mut *self.init_confirm)
                            .password(true)
                            .hint_text("Re-enter your password")
                            .desired_width(280.0)
                    );
                    
                    // Password strength indicator
                    if !self.init_password.is_empty() {
                        ui.add_space(12.0);
                        self.show_password_strength_indicator(ui, &self.init_password.clone());
                    }
                    
                    ui.add_space(24.0);
                    
                    // Buttons
                    ui.vertical_centered(|ui| {
                        let create_btn = egui::Button::new(
                            egui::RichText::new("Create Vault").size(14.0).color(egui::Color32::WHITE)
                        )
                        .fill(egui::Color32::from_rgb(34, 197, 94))
                        .rounding(egui::Rounding::same(10.0))
                        .min_size(egui::vec2(260.0, 44.0));
                        
                        if ui.add(create_btn).clicked() {
                            match self.init_vault() {
                                Ok(()) => {
                                    self.toast_success("Vault created successfully!");
                                }
                                Err(e) => {
                                    self.toast_error(e);
                                }
                            }
                        }
                        
                        ui.add_space(12.0);
                        
                        if ui.add(
                            egui::Button::new(egui::RichText::new("← Back").color(muted_color))
                                .fill(egui::Color32::TRANSPARENT)
                                .stroke(egui::Stroke::NONE)
                        ).clicked() {
                            self.current_screen = Screen::Welcome;
                            *self.init_password = String::new();
                            *self.init_confirm = String::new();
                        }
                    });
                });
            
            ui.add_space(24.0);
            
            // Security tip
            ui.label(
                egui::RichText::new("💡 Use a unique password you don't use elsewhere")
                    .size(12.0)
                    .color(muted_color)
            );
        });
    }

    /// Show login screen
    pub fn show_login_screen(&mut self, ui: &mut egui::Ui) {
        let current_theme = self.current_theme.clone();
        let muted_color = theme::muted_text_color(&current_theme);
        let frame_fill = theme::frame_fill(&current_theme);
        let border_color = theme::border_color(&current_theme);
        
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            
            // Header
            ui.label(egui::RichText::new("🔐").size(48.0));
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Welcome Back").size(24.0).strong());
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Enter your master password to unlock")
                    .size(13.0)
                    .color(muted_color)
            );
            
            ui.add_space(32.0);
            
            // Form card
            egui::Frame::none()
                .fill(frame_fill)
                .stroke(egui::Stroke::new(1.0, border_color))
                .rounding(egui::Rounding::same(16.0))
                .inner_margin(egui::Margin::same(32.0))
                .show(ui, |ui| {
                    ui.set_width(320.0);
                    
                    // Master password
                    ui.label(egui::RichText::new("Master Password").size(13.0).strong());
                    ui.add_space(6.0);
                    
                    let password_input = ui.add(
                        egui::TextEdit::singleline(&mut *self.login_password)
                            .password(true)
                            .hint_text("Enter your password")
                            .desired_width(280.0)
                    );
                    
                    // Submit on Enter
                    if password_input.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        match self.login() {
                            Ok(()) => {
                                self.toast_success("Vault opened successfully!");
                            }
                            Err(e) => {
                                self.toast_error(e);
                            }
                        }
                    }
                    
                    ui.add_space(24.0);
                    
                    // Buttons
                    ui.vertical_centered(|ui| {
                        let open_btn = egui::Button::new(
                            egui::RichText::new("🔓  Unlock").size(14.0).color(egui::Color32::WHITE)
                        )
                        .fill(egui::Color32::from_rgb(59, 130, 246))
                        .rounding(egui::Rounding::same(10.0))
                        .min_size(egui::vec2(260.0, 44.0));
                        
                        if ui.add(open_btn).clicked() {
                            match self.login() {
                                Ok(()) => {
                                    self.toast_success("Vault opened successfully!");
                                }
                                Err(e) => {
                                    self.toast_error(e);
                                }
                            }
                        }
                        
                        ui.add_space(12.0);
                        
                        if ui.add(
                            egui::Button::new(egui::RichText::new("← Back").color(muted_color))
                                .fill(egui::Color32::TRANSPARENT)
                                .stroke(egui::Stroke::NONE)
                        ).clicked() {
                            self.current_screen = Screen::Welcome;
                            *self.login_password = String::new();
                        }
                    });
                });
            
            // Vault info
            ui.add_space(24.0);
            ui.label(
                egui::RichText::new(format!("📁 {}", self.vault_file))
                    .size(11.0)
                    .color(egui::Color32::from_rgb(80, 80, 85))
            );
        });
    }
}
