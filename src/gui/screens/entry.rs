//! Entry Screens Module
//!
//! Add and Edit entry screens.

use eframe::egui;
use super::super::types::Screen;
use super::super::theme;
use super::super::widgets;
use super::super::app::PassmanApp;

impl PassmanApp {
    /// Show add entry screen
    pub fn show_add_entry_screen(&mut self, ui: &mut egui::Ui) {
        let current_theme = self.current_theme;
        let border_color = theme::border_color(&current_theme);
        let frame_fill = theme::frame_fill(&current_theme);
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
                    ui.label(egui::RichText::new("➕").size(22.0));
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("Add New Entry").size(20.0).strong());
                    
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
                            self.clear_add_form();
                        }
                    });
                });
            });
        
        ui.add_space(16.0);
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                // Form card
                egui::Frame::none()
                    .fill(frame_fill)
                    .stroke(egui::Stroke::new(1.0, border_color))
                    .rounding(egui::Rounding::same(16.0))
                    .inner_margin(egui::Margin::same(28.0))
                    .show(ui, |ui| {
                        ui.set_width(400.0);
                        let field_width = 370.0;
                        let field_height = 38.0;
                        
                        // Entry ID
                        ui.label(egui::RichText::new("Entry ID").size(13.0).strong());
                        ui.add_space(6.0);
                        let id_response = ui.add_sized(
                            egui::vec2(field_width, field_height),
                            egui::TextEdit::singleline(&mut self.add_id)
                                .hint_text(egui::RichText::new("e.g., gmail, work").color(dim_color))
                                .margin(egui::Margin::symmetric(14.0, 10.0))
                        );
                        if id_response.changed() {
                            self.clear_form_error("add_id");
                        }
                        self.show_field_error(ui, "add_id");
                        ui.add_space(14.0);

                        // Username
                        ui.label(egui::RichText::new("Username").size(13.0).strong());
                        ui.add_space(6.0);
                        let username_response = ui.add_sized(
                            egui::vec2(field_width, field_height),
                            egui::TextEdit::singleline(&mut self.add_username)
                                .hint_text(egui::RichText::new("Username or email").color(dim_color))
                                .margin(egui::Margin::symmetric(14.0, 10.0))
                        );
                        if username_response.changed() {
                            self.clear_form_error("add_username");
                        }
                        self.show_field_error(ui, "add_username");
                        ui.add_space(14.0);

                        // Generate password checkbox
                        ui.checkbox(&mut self.generate_password, "Generate secure password");
                        ui.add_space(10.0);

                        if self.generate_password {
                            ui.label(egui::RichText::new("Password Length").size(13.0).strong());
                            ui.add_space(6.0);
                            ui.add(egui::Slider::new(&mut self.password_length, 8..=64)
                                .text("characters"));
                            ui.add_space(14.0);
                        } else {
                            // Password field
                            ui.label(egui::RichText::new("Password").size(13.0).strong());
                            ui.add_space(6.0);
                            ui.horizontal(|ui| {
                                let eye_width = 42.0;
                                let gap = 8.0;
                                let pw_width = field_width - eye_width - gap;
                                
                                let pw_response = ui.add_sized(
                                    egui::vec2(pw_width, field_height),
                                    egui::TextEdit::singleline(&mut self.add_password)
                                        .password(!self.add_show_password)
                                        .margin(egui::Margin::symmetric(14.0, 10.0))
                                );
                                if pw_response.changed() {
                                    self.clear_form_error("add_password");
                                }
                                
                                let eye_text = if self.add_show_password { "🙈" } else { "👁" };
                                let eye_btn = egui::Button::new(
                                    egui::RichText::new(eye_text).size(16.0)
                                )
                                .rounding(egui::Rounding::same(8.0))
                                .min_size(egui::vec2(eye_width, field_height));
                                
                                if ui.add(eye_btn).clicked() {
                                    self.add_show_password = !self.add_show_password;
                                }
                            });
                            self.show_field_error(ui, "add_password");
                            
                            // Password strength indicator
                            if !self.add_password.is_empty() {
                                ui.add_space(6.0);
                                ui.scope(|ui| {
                                    let password = self.add_password.clone();
                                    widgets::show_password_strength_indicator(ui, &password);
                                });
                            }
                            ui.add_space(14.0);
                        }

                        // Note
                        ui.label(egui::RichText::new("Note").size(13.0).strong());
                        ui.add_space(6.0);
                        ui.add_sized(
                            egui::vec2(field_width, 80.0),
                            egui::TextEdit::multiline(&mut self.add_note)
                                .hint_text(egui::RichText::new("Optional notes").color(dim_color))
                                .margin(egui::Margin::symmetric(14.0, 10.0))
                        );
                        
                        ui.add_space(24.0);
                        
                        // Submit button
                        ui.vertical_centered(|ui| {
                            let submit_btn = egui::Button::new(
                                egui::RichText::new("✓ Add Entry")
                                    .size(15.0)
                                    .color(egui::Color32::WHITE)
                            )
                            .fill(egui::Color32::from_rgb(34, 197, 94))
                            .rounding(egui::Rounding::same(10.0))
                            .min_size(egui::vec2(200.0, 44.0));
                            
                            if ui.add(submit_btn).clicked() && self.validate_add_entry() {
                                match self.add_entry() {
                                    Ok(()) => {
                                        self.toast_success("Entry added successfully!");
                                        self.clear_form_errors();
                                    }
                                    Err(e) => {
                                        self.toast_error(e);
                                    }
                                }
                            }
                        });
                    });
                
                ui.add_space(24.0);
            });
        });
    }

    /// Show edit entry screen
    pub fn show_edit_entry_screen(&mut self, ui: &mut egui::Ui, id: &str) {
        let current_theme = self.current_theme;
        let border_color = theme::border_color(&current_theme);
        let frame_fill = theme::frame_fill(&current_theme);
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
                    ui.label(egui::RichText::new("✏").size(22.0));
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(format!("Edit: {}", id)).size(20.0).strong());
                    
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
                            self.clear_edit_form();
                            self.clear_form_errors();
                        }
                    });
                });
            });
        
        ui.add_space(16.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                // Form card
                egui::Frame::none()
                    .fill(frame_fill)
                    .stroke(egui::Stroke::new(1.0, border_color))
                    .rounding(egui::Rounding::same(16.0))
                    .inner_margin(egui::Margin::same(28.0))
                    .show(ui, |ui| {
                        ui.set_width(400.0);
                        let field_width = 370.0;
                        let field_height = 38.0;
                        
                        // Username
                        ui.label(egui::RichText::new("Username").size(13.0).strong());
                        ui.add_space(6.0);
                        let username_response = ui.add_sized(
                            egui::vec2(field_width, field_height),
                            egui::TextEdit::singleline(&mut self.edit_username)
                                .hint_text(egui::RichText::new("Username or email").color(dim_color))
                                .margin(egui::Margin::symmetric(14.0, 10.0))
                        );
                        if username_response.changed() {
                            self.clear_form_error("edit_username");
                        }
                        self.show_field_error(ui, "edit_username");
                        ui.add_space(14.0);

                        // Generate password checkbox
                        ui.checkbox(&mut self.edit_generate_password, "Generate new password");
                        ui.add_space(10.0);

                        if self.edit_generate_password {
                            ui.label(egui::RichText::new("Password Length").size(13.0).strong());
                            ui.add_space(6.0);
                            ui.add(egui::Slider::new(&mut self.password_length, 8..=64)
                                .text("characters"));
                            ui.add_space(14.0);
                        } else {
                            // Password field
                            ui.label(egui::RichText::new("Password").size(13.0).strong());
                            ui.add_space(6.0);
                            ui.horizontal(|ui| {
                                let eye_width = 42.0;
                                let gap = 8.0;
                                let pw_width = field_width - eye_width - gap;
                                
                                let pw_response = ui.add_sized(
                                    egui::vec2(pw_width, field_height),
                                    egui::TextEdit::singleline(&mut self.edit_password)
                                        .password(!self.edit_show_password)
                                        .margin(egui::Margin::symmetric(14.0, 10.0))
                                );
                                if pw_response.changed() {
                                    self.clear_form_error("edit_password");
                                }
                                
                                let eye_text = if self.edit_show_password { "🙈" } else { "👁" };
                                let eye_btn = egui::Button::new(
                                    egui::RichText::new(eye_text).size(16.0)
                                )
                                .rounding(egui::Rounding::same(8.0))
                                .min_size(egui::vec2(eye_width, field_height));
                                
                                if ui.add(eye_btn).clicked() {
                                    self.edit_show_password = !self.edit_show_password;
                                }
                            });
                            self.show_field_error(ui, "edit_password");

                            // Password strength indicator
                            if !self.edit_password.is_empty() {
                                ui.add_space(6.0);
                                ui.scope(|ui| {
                                    let password = self.edit_password.clone();
                                    widgets::show_password_strength_indicator(ui, &password);
                                });
                            }
                            ui.add_space(14.0);
                        }

                        // Note
                        ui.label(egui::RichText::new("Note").size(13.0).strong());
                        ui.add_space(6.0);
                        ui.add_sized(
                            egui::vec2(field_width, 80.0),
                            egui::TextEdit::multiline(&mut self.edit_note)
                                .hint_text(egui::RichText::new("Optional notes").color(dim_color))
                                .margin(egui::Margin::symmetric(14.0, 10.0))
                        );
                        
                        ui.add_space(24.0);
                        
                        // Submit button
                        ui.vertical_centered(|ui| {
                            let submit_btn = egui::Button::new(
                                egui::RichText::new("✓ Update Entry")
                                    .size(15.0)
                                    .color(egui::Color32::WHITE)
                            )
                            .fill(egui::Color32::from_rgb(34, 197, 94))
                            .rounding(egui::Rounding::same(10.0))
                            .min_size(egui::vec2(200.0, 44.0));
                            
                            if ui.add(submit_btn).clicked() && self.validate_edit_entry() {
                                match self.update_entry() {
                                    Ok(()) => {
                                        self.toast_success("Entry updated successfully!");
                                        self.clear_form_errors();
                                    }
                                    Err(e) => {
                                        self.toast_error(e);
                                    }
                                }
                            }
                        });
                    });
                
                ui.add_space(24.0);
            });
        });
    }
}
