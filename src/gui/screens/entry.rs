//! Entry Screens Module
//!
//! Add and Edit entry screens.

use eframe::egui;
use super::super::types::{Screen, SPACING, INPUT_WIDTH, BUTTON_HEIGHT};
use super::super::theme;
use super::super::widgets;
use super::super::app::PassmanApp;

impl PassmanApp {
    /// Show add entry screen
    pub fn show_add_entry_screen(&mut self, ui: &mut egui::Ui) {
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
                    ui.label(egui::RichText::new("➕").size(20.0));
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Add New Entry").size(18.0).strong());
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let back_btn = egui::Button::new("Back")
                            .fill(egui::Color32::from_rgb(55, 65, 81))
                            .stroke(egui::Stroke::new(1.0, border_color))
                            .rounding(egui::Rounding::same(6.0))
                            .min_size(egui::vec2(70.0, 28.0));
                        
                        if ui.add(back_btn).clicked() {
                            self.current_screen = Screen::Main;
                            self.clear_add_form();
                        }
                    });
                });
            });
        
        ui.add_space(SPACING);
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(SPACING);

                egui::Grid::new("add_entry_grid")
                    .num_columns(2)
                    .spacing([SPACING * 2.0, SPACING])
                    .striped(false)
                .show(ui, |ui| {
                    ui.label("Entry ID:");
                    ui.vertical(|ui| {
                        let id_response = ui.add(egui::TextEdit::singleline(&mut self.add_id)
                            .desired_width(INPUT_WIDTH)
                            .hint_text("e.g., gmail, work"));
                        if id_response.changed() {
                            self.clear_form_error("add_id");
                        }
                        self.show_field_error(ui, "add_id");
                    });
                    ui.end_row();

                    ui.label("Username:");
                    ui.vertical(|ui| {
                        let username_response = ui.add(egui::TextEdit::singleline(&mut self.add_username)
                            .desired_width(INPUT_WIDTH)
                            .hint_text("Username or email"));
                        if username_response.changed() {
                            self.clear_form_error("add_username");
                        }
                        self.show_field_error(ui, "add_username");
                    });
                    ui.end_row();

                    ui.label("");
                    ui.checkbox(&mut self.generate_password, "Generate secure password");
                    ui.end_row();

                    if self.generate_password {
                        ui.label("Length:");
                        ui.add(egui::Slider::new(&mut self.password_length, 8..=64)
                            .text("characters"));
                        ui.end_row();
                    } else {
                        ui.label("Password:");
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                let field_height = 24.0;
                                let btn_width = 40.0;
                                let gap = 8.0;
                                let field_width = INPUT_WIDTH - btn_width - gap;
                                
                                let pw_response = ui.add_sized(
                                    egui::vec2(field_width, field_height),
                                    egui::TextEdit::singleline(&mut self.add_password)
                                        .password(!self.add_show_password)
                                );
                                if pw_response.changed() {
                                    self.clear_form_error("add_password");
                                }
                                
                                let eye_text = if self.add_show_password { "🙈" } else { "👁" };
                                if ui.add_sized(
                                    egui::vec2(btn_width, field_height),
                                    egui::Button::new(eye_text)
                                ).clicked() {
                                    self.add_show_password = !self.add_show_password;
                                }
                            });
                            self.show_field_error(ui, "add_password");
                        });
                        ui.end_row();
                        
                        // Visual password strength indicator
                        if !self.add_password.is_empty() {
                            ui.label("");
                            ui.scope(|ui| {
                                let password = self.add_password.clone();
                                widgets::show_password_strength_indicator(ui, &password);
                            });
                            ui.end_row();
                        }
                    }

                    ui.label("Note:");
                    ui.add(egui::TextEdit::multiline(&mut self.add_note)
                        .desired_width(INPUT_WIDTH)
                        .desired_rows(3)
                        .hint_text("Optional notes"));
                    ui.end_row();
                });

                ui.add_space(SPACING * 2.0);
                
                if self.success_button(ui, "Add Entry", [150.0, BUTTON_HEIGHT]).clicked() && self.validate_add_entry() {
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
                
                ui.add_space(SPACING * 2.0);
            });
        });
    }

    /// Show edit entry screen
    pub fn show_edit_entry_screen(&mut self, ui: &mut egui::Ui, id: &str) {
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
                    ui.label(egui::RichText::new("✏").size(20.0));
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(format!("Edit: {}", id)).size(18.0).strong());
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let back_btn = egui::Button::new("Back")
                            .fill(egui::Color32::from_rgb(55, 65, 81))
                            .stroke(egui::Stroke::new(1.0, border_color))
                            .rounding(egui::Rounding::same(6.0))
                            .min_size(egui::vec2(70.0, 28.0));
                        
                        if ui.add(back_btn).clicked() {
                            self.current_screen = Screen::Main;
                            self.clear_edit_form();
                            self.clear_form_errors();
                        }
                    });
                });
            });
        
        ui.add_space(SPACING);

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(SPACING);
            
                egui::Grid::new("edit_entry_grid")
                .num_columns(2)
                .spacing([SPACING * 2.0, SPACING])
                .striped(false)
                .show(ui, |ui| {
                    ui.label("Username:");
                    ui.vertical(|ui| {
                        let username_response = ui.add(egui::TextEdit::singleline(&mut self.edit_username)
                            .desired_width(INPUT_WIDTH)
                            .hint_text("Username or email"));
                        if username_response.changed() {
                            self.clear_form_error("edit_username");
                        }
                        self.show_field_error(ui, "edit_username");
                    });
                    ui.end_row();

                    ui.label("");
                    ui.checkbox(&mut self.edit_generate_password, "Generate new password");
                    ui.end_row();

                    if self.edit_generate_password {
                        ui.label("Length:");
                        ui.add(egui::Slider::new(&mut self.password_length, 8..=64)
                            .text("characters"));
                        ui.end_row();
                    } else {
                        ui.label("Password:");
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                let field_height = 24.0;
                                let btn_width = 40.0;
                                let gap = 8.0;
                                let field_width = INPUT_WIDTH - btn_width - gap;
                                
                                let pw_response = ui.add_sized(
                                    egui::vec2(field_width, field_height),
                                    egui::TextEdit::singleline(&mut self.edit_password)
                                        .password(!self.edit_show_password)
                                );
                                if pw_response.changed() {
                                    self.clear_form_error("edit_password");
                                }
                                
                                let eye_text = if self.edit_show_password { "🙈" } else { "👁" };
                                if ui.add_sized(
                                    egui::vec2(btn_width, field_height),
                                    egui::Button::new(eye_text)
                                ).clicked() {
                                    self.edit_show_password = !self.edit_show_password;
                                }
                            });
                            self.show_field_error(ui, "edit_password");
                        });
                        ui.end_row();

                        // Visual password strength indicator
                        if !self.edit_password.is_empty() {
                            ui.label("");
                            ui.scope(|ui| {
                                let password = self.edit_password.clone();
                                widgets::show_password_strength_indicator(ui, &password);
                            });
                            ui.end_row();
                        }
                    }

                    ui.label("Note:");
                    ui.add(egui::TextEdit::multiline(&mut self.edit_note)
                        .desired_width(INPUT_WIDTH)
                        .desired_rows(3)
                        .hint_text("Optional notes"));
                    ui.end_row();
                });

                ui.add_space(SPACING * 2.0);
                
                if self.success_button(ui, "Update Entry", [150.0, BUTTON_HEIGHT]).clicked() && self.validate_edit_entry() {
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
                
                ui.add_space(SPACING * 2.0);
            });
        });
    }
}
