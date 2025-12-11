//! Entry Screens Module
//!
//! Add and Edit entry screens.

use eframe::egui;
use super::super::types::{Screen, SPACING, PADDING, INPUT_WIDTH, BUTTON_HEIGHT};
use super::super::widgets;
use super::super::app::PassmanApp;

impl PassmanApp {
    /// Show add entry screen
    pub fn show_add_entry_screen(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Add New Entry");
        });
        ui.separator();
        ui.add_space(PADDING);
        
        ui.vertical_centered(|ui| {
            ui.add_space(SPACING * 2.0);

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
                                let pw_response = ui.add(egui::TextEdit::singleline(&mut self.add_password)
                                    .desired_width(INPUT_WIDTH - 80.0)
                                    .password(!self.add_show_password));
                                if pw_response.changed() {
                                    self.clear_form_error("add_password");
                                }
                                
                                let eye_text = if self.add_show_password { "🙈" } else { "👁" };
                                if self.secondary_button(ui, eye_text, [40.0, 30.0]).clicked() {
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
            
            if self.success_button(ui, "Add Entry", [150.0, BUTTON_HEIGHT]).clicked() {
                if self.validate_add_entry() {
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
            }
            ui.add_space(SPACING);
            if self.secondary_button(ui, "Cancel", [150.0, BUTTON_HEIGHT]).clicked() {
                self.current_screen = Screen::Main;
                self.clear_add_form();
            }
        });
    }

    /// Show edit entry screen
    pub fn show_edit_entry_screen(&mut self, ui: &mut egui::Ui, id: &str) {
        ui.vertical(|ui| {
            ui.heading(format!("Edit Entry: {}", id));
        });
        ui.separator();
        ui.add_space(PADDING);

        ui.vertical_centered(|ui| {
            ui.add_space(SPACING * 2.0);
            
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
                                let pw_response = ui.add(egui::TextEdit::singleline(&mut self.edit_password)
                                    .desired_width(INPUT_WIDTH - 80.0)
                                    .password(!self.edit_show_password));
                                if pw_response.changed() {
                                    self.clear_form_error("edit_password");
                                }
                                
                                let eye_text = if self.edit_show_password { "🙈" } else { "👁" };
                                if self.secondary_button(ui, eye_text, [40.0, 30.0]).clicked() {
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
            
            ui.horizontal(|ui| {
                if self.success_button(ui, "Update Entry", [150.0, BUTTON_HEIGHT]).clicked() {
                    if self.validate_edit_entry() {
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
                }
                
                if self.secondary_button(ui, "Cancel", [150.0, BUTTON_HEIGHT]).clicked() {
                    self.current_screen = Screen::Main;
                    self.clear_edit_form();
                    self.clear_form_errors();
                }
            });
        });
    }
}
