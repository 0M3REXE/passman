//! Welcome Screens Module
//!
//! Welcome, Init (create vault), and Login screens.

use eframe::egui;
use crate::security::vault::VaultManager;
use super::super::types::Screen;
use super::super::theme;
use super::super::app::PassmanApp;

impl PassmanApp {
    /// Show welcome/home screen
    pub fn show_welcome_screen(&mut self, ui: &mut egui::Ui) {
        let current_theme = self.current_theme;
        let muted_color = theme::muted_text_color(&current_theme);
        let frame_fill = theme::frame_fill(&current_theme);
        let border_color = theme::border_color(&current_theme);
        let dim_color = theme::dim_text_color(&current_theme);
        
        ui.vertical_centered(|ui| {
            ui.add_space(36.0);
            
            // ════════════════════════════════════════════════════════════════
            // LOGO / BRANDING
            // ════════════════════════════════════════════════════════════════
            ui.label(egui::RichText::new("🔐").size(56.0));
            ui.add_space(12.0);
            ui.label(egui::RichText::new("Passman").size(32.0).strong());
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Secure Password Manager")
                    .size(14.0)
                    .color(muted_color)
            );
            
            ui.add_space(32.0);
            
            // ════════════════════════════════════════════════════════════════
            // MAIN CARD
            // ════════════════════════════════════════════════════════════════
            egui::Frame::none()
                .fill(frame_fill)
                .stroke(egui::Stroke::new(1.0, border_color))
                .rounding(egui::Rounding::same(2.0))
                .inner_margin(egui::Margin::same(32.0))
                .show(ui, |ui| {
                    ui.set_width(320.0);
                    
                    // Vault file selection
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("Vault Location")
                                .size(13.0)
                                .strong()
                                .color(muted_color)
                        );
                    });
                    ui.add_space(12.0);
                    
                    ui.vertical_centered(|ui| {
                        let total_width = 280.0;
                        let browse_btn_size = 40.0;
                        let gap = 6.0;
                        let field_width = total_width - browse_btn_size - gap;
                        let field_height = 38.0;
                        
                        ui.allocate_ui_with_layout(
                            egui::vec2(total_width, field_height),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                ui.add_sized(
                                    egui::vec2(field_width, field_height),
                                    egui::TextEdit::singleline(&mut self.vault_file)
                                        .hint_text(
                                            egui::RichText::new("vault.dat")
                                                .color(dim_color)
                                        )
                                        .margin(egui::Margin::symmetric(12.0, 8.0))
                                );
                                
                                if ui.add_sized(
                                    egui::vec2(browse_btn_size, field_height),
                                    egui::Button::new("...")
                                        .fill(egui::Color32::from_rgb(42, 43, 52))
                                        .stroke(egui::Stroke::new(1.0, border_color))
                                        .rounding(egui::Rounding::same(2.0))
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
                        let btn_width = 280.0;
                        let btn_height = 42.0;
                        
                        if vault_exists {
                            // Open existing vault (primary)
                            let open_btn = egui::Button::new(
                                egui::RichText::new("Open Vault")
                                    .size(15.0)
                                    .color(egui::Color32::WHITE)
                            )
                            .fill(egui::Color32::from_rgb(79, 120, 247))
                            .rounding(egui::Rounding::same(2.0))
                            .min_size(egui::vec2(btn_width, btn_height));
                            
                            if ui.add(open_btn).clicked() {
                                self.current_screen = Screen::Login;
                            }
                            
                            ui.add_space(10.0);
                            
                            // Create new vault (secondary)
                            let create_btn = egui::Button::new(
                                egui::RichText::new("Create New Vault").size(13.0)
                            )
                            .fill(egui::Color32::TRANSPARENT)
                            .stroke(egui::Stroke::new(1.0, border_color))
                            .rounding(egui::Rounding::same(2.0))
                            .min_size(egui::vec2(btn_width, btn_height));
                            
                            if ui.add(create_btn).clicked() {
                                self.current_screen = Screen::Init;
                            }
                        } else {
                            // Create new vault (primary)
                            let create_btn = egui::Button::new(
                                egui::RichText::new("Create Vault")
                                    .size(15.0)
                                    .color(egui::Color32::WHITE)
                            )
                            .fill(egui::Color32::from_rgb(34, 197, 94))
                            .rounding(egui::Rounding::same(2.0))
                            .min_size(egui::vec2(btn_width, btn_height));
                            
                            if ui.add(create_btn).clicked() {
                                self.current_screen = Screen::Init;
                            }
                            
                            ui.add_space(10.0);
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
                    egui::Button::new(
                        egui::RichText::new("Settings").size(11.0).color(muted_color)
                    )
                    .fill(egui::Color32::TRANSPARENT)
                    .stroke(egui::Stroke::NONE)
                ).clicked() {
                    self.current_screen = Screen::Settings;
                }
                
                ui.add_space(8.0);
                
                // Help button
                if ui.add(
                    egui::Button::new(
                        egui::RichText::new("? Help").size(11.0).color(muted_color)
                    )
                    .fill(egui::Color32::TRANSPARENT)
                    .stroke(egui::Stroke::NONE)
                ).clicked() {
                    self.show_onboarding = true;
                    self.onboarding_step = 0;
                }
                
                ui.add_space(8.0);
                
                // Troubleshooting menu
                ui.menu_button(
                    egui::RichText::new("Troubleshoot").size(11.0).color(muted_color),
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
                            egui::RichText::new("(!) Forgot password = lost vault")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(251, 191, 36))
                        );
                    }
                );
            });
            
            // Credit section at bottom
            ui.add_space(16.0);
            ui.label(
                egui::RichText::new("Created by 0M3REXE")
                    .size(10.0)
                    .color(egui::Color32::from_rgb(80, 82, 96))
            );
            ui.add_space(2.0);
            if ui.add(
                egui::Button::new(
                    egui::RichText::new("github.com/0M3REXE/passman")
                        .size(10.0)
                        .color(egui::Color32::from_rgb(99, 138, 255))
                )
                .fill(egui::Color32::TRANSPARENT)
                .stroke(egui::Stroke::NONE)
            ).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                let _ = open::that("https://github.com/0M3REXE/passman");
            }
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                    .size(9.0)
                    .color(egui::Color32::from_rgb(60, 62, 72))
            );
        });
    }

    /// Show vault initialization screen
    pub fn show_init_screen(&mut self, ui: &mut egui::Ui) {
        let current_theme = self.current_theme;
        let muted_color = theme::muted_text_color(&current_theme);
        let frame_fill = theme::frame_fill(&current_theme);
        let border_color = theme::border_color(&current_theme);
        let dim_color = theme::dim_text_color(&current_theme);
        
        ui.vertical_centered(|ui| {
            ui.add_space(24.0);
            
            // Header
            ui.label(egui::RichText::new("Create New Vault").size(24.0).strong());
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Set up a secure password to protect your vault")
                    .size(12.0)
                    .color(muted_color)
            );
            
            ui.add_space(24.0);
            
            // Form card
            egui::Frame::none()
                .fill(frame_fill)
                .stroke(egui::Stroke::new(1.0, border_color))
                .rounding(egui::Rounding::same(2.0))
                .inner_margin(egui::Margin::same(28.0))
                .show(ui, |ui| {
                    ui.set_width(320.0);
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    
                    let field_width = 280.0;
                    let field_height = 38.0;
                    
                    // Master password
                    ui.label(egui::RichText::new("Master Password").size(12.0).strong());
                    ui.add_space(6.0);
                    ui.add_sized(
                        egui::vec2(field_width, field_height),
                        egui::TextEdit::singleline(&mut *self.init_password)
                            .password(true)
                            .hint_text(
                                egui::RichText::new("Enter a strong password")
                                    .color(dim_color)
                            )
                            .margin(egui::Margin::symmetric(12.0, 8.0))
                    );
                    
                    ui.add_space(16.0);
                    
                    // Confirm password
                    ui.label(egui::RichText::new("Confirm Password").size(12.0).strong());
                    ui.add_space(6.0);
                    ui.add_sized(
                        egui::vec2(field_width, field_height),
                        egui::TextEdit::singleline(&mut *self.init_confirm)
                            .password(true)
                            .hint_text(
                                egui::RichText::new("Re-enter your password")
                                    .color(dim_color)
                            )
                            .margin(egui::Margin::symmetric(12.0, 8.0))
                    );
                    
                    // Password strength indicator
                    if !self.init_password.is_empty() {
                        ui.add_space(12.0);
                        self.show_password_strength_indicator(ui, &self.init_password.clone());
                    }
                    
                    ui.add_space(20.0);
                    
                    // Buttons
                    ui.vertical_centered(|ui| {
                        let btn_width = 280.0;
                        
                        let create_btn = egui::Button::new(
                            egui::RichText::new("Create Vault")
                                .size(14.0)
                                .color(egui::Color32::WHITE)
                        )
                        .fill(egui::Color32::from_rgb(34, 197, 94))
                        .rounding(egui::Rounding::same(2.0))
                        .min_size(egui::vec2(btn_width, 42.0));
                        
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
                        
                        ui.add_space(10.0);
                        
                        let back_btn = egui::Button::new(
                            egui::RichText::new("Back").size(12.0)
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .stroke(egui::Stroke::new(1.0, border_color))
                        .rounding(egui::Rounding::same(2.0))
                        .min_size(egui::vec2(btn_width, 42.0));
                        
                        if ui.add(back_btn).clicked() {
                            self.current_screen = Screen::Welcome;
                            *self.init_password = String::new();
                            *self.init_confirm = String::new();
                        }
                    });
                    }); // end with_layout
                });
            
            ui.add_space(16.0);
            
            // Security tip
            ui.label(
                egui::RichText::new("Tip: Use a unique password you don't use elsewhere")
                    .size(11.0)
                    .color(muted_color)
            );
        });
    }

    /// Show login screen
    pub fn show_login_screen(&mut self, ui: &mut egui::Ui) {
        let current_theme = self.current_theme;
        let muted_color = theme::muted_text_color(&current_theme);
        let frame_fill = theme::frame_fill(&current_theme);
        let border_color = theme::border_color(&current_theme);
        let dim_color = theme::dim_text_color(&current_theme);
        
        ui.vertical_centered(|ui| {
            ui.add_space(32.0);
            
            // Header
            ui.label(egui::RichText::new("🔐").size(48.0));
            ui.add_space(10.0);
            ui.label(egui::RichText::new("Welcome Back").size(26.0).strong());
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Enter your master password to unlock")
                    .size(13.0)
                    .color(muted_color)
            );
            
            ui.add_space(28.0);
            
            // Form card
            egui::Frame::none()
                .fill(frame_fill)
                .stroke(egui::Stroke::new(1.0, border_color))
                .rounding(egui::Rounding::same(2.0))
                .inner_margin(egui::Margin::same(32.0))
                .show(ui, |ui| {
                    ui.set_width(320.0);
                    
                    let btn_width = 280.0;
                    let field_height = 42.0;
                    
                    // Master password
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("Master Password")
                                .size(13.0)
                                .strong()
                        );
                    });
                    ui.add_space(10.0);
                    
                    ui.vertical_centered(|ui| {
                        ui.add_sized(
                            egui::vec2(btn_width, field_height),
                            egui::TextEdit::singleline(&mut *self.login_password)
                                .password(true)
                                .hint_text(
                                    egui::RichText::new("Enter your password")
                                        .color(dim_color)
                                )
                                .margin(egui::Margin::symmetric(14.0, 10.0))
                        );
                    });
                    
                    ui.add_space(22.0);
                    
                    // Buttons
                    ui.vertical_centered(|ui| {
                        let open_btn = egui::Button::new(
                            egui::RichText::new("Unlock")
                                .size(15.0)
                                .color(egui::Color32::WHITE)
                        )
                        .fill(egui::Color32::from_rgb(79, 120, 247))
                        .rounding(egui::Rounding::same(2.0))
                        .min_size(egui::vec2(btn_width, 44.0));
                        
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
                        
                        ui.add_space(10.0);
                        
                        let back_btn = egui::Button::new(
                            egui::RichText::new("Back").size(13.0)
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .stroke(egui::Stroke::new(1.0, border_color))
                        .rounding(egui::Rounding::same(2.0))
                        .min_size(egui::vec2(btn_width, 44.0));
                        
                        if ui.add(back_btn).clicked() {
                            self.current_screen = Screen::Welcome;
                            *self.login_password = String::new();
                        }
                    });
                });
            
            // Vault info
            ui.add_space(20.0);
            ui.label(
                egui::RichText::new(format!("Vault: {}", self.vault_file))
                    .size(11.0)
                    .color(egui::Color32::from_rgb(70, 72, 84))
            );
        });
    }
}
