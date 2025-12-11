//! Main Screen Module
//!
//! Main vault screen with entry list and search.

use eframe::egui;
use super::super::types::{Screen, SPACING};
use super::super::theme;
use super::super::app::PassmanApp;

impl PassmanApp {
    /// Show main vault screen
    pub fn show_main_screen(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // Header with title and action buttons
        ui.horizontal(|ui| {
            ui.heading("🔐 Password Vault");
            
            // Keyboard shortcuts hint
            ui.add_space(8.0);
            ui.label("⌨️").on_hover_text(
                "Keyboard Shortcuts:\n\
                • Ctrl+N - New entry\n\
                • Ctrl+F - Focus search\n\
                • Ctrl+L - Lock vault\n\
                • Ctrl+H - Health dashboard\n\
                • Ctrl+S - Settings\n\
                • Escape - Go back"
            );
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.secondary_button(ui, "🔒 Lock", [70.0, 30.0]).clicked() {
                    self.lock_vault();
                    self.clear_message();
                }
                
                if self.secondary_button(ui, "⚙️", [35.0, 30.0]).clicked() {
                    self.current_screen = Screen::Settings;
                }
                
                if self.primary_button(ui, "🏥 Health", [80.0, 30.0]).clicked() {
                    self.current_screen = Screen::HealthDashboard;
                }
                
                if self.secondary_button(ui, "📦 Export", [80.0, 30.0]).clicked() {
                    self.current_screen = Screen::ImportExport;
                }
                
                if self.success_button(ui, "➕ Add", [70.0, 30.0]).clicked() {
                    self.current_screen = Screen::AddEntry;
                    self.clear_add_form();
                }
            });
        });
        ui.separator();
        ui.add_space(SPACING);

        // Search bar with entry count and clear button
        ui.horizontal(|ui| {
            ui.label("🔍");
            let search_response = ui.add(egui::TextEdit::singleline(&mut self.search_query)
                .hint_text("Search entries... (Ctrl+F)")
                .desired_width(250.0));
            
            // Request focus from keyboard shortcut
            if self.request_search_focus {
                search_response.request_focus();
                self.request_search_focus = false;
            }
            
            // Clear search button
            if !self.search_query.is_empty() {
                if self.secondary_button(ui, "✕", [25.0, 25.0]).clicked() {
                    self.search_query.clear();
                }
            }
            
            ui.add_space(SPACING * 2.0);
            
            // Entry count
            let filtered_count = self.filter_entries().len();
            let total_count = self.entries.len();
            if self.search_query.is_empty() {
                ui.label(format!("{} entries", total_count));
            } else {
                ui.label(format!("{} of {} entries", filtered_count, total_count));
            }
        });
        ui.add_space(SPACING);

        // Entries list
        self.render_entry_list(ui, ctx);
    }

    /// Render the entry list
    fn render_entry_list(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
            let filtered_entries: Vec<(String, crate::model::Entry)> = self.filter_entries()
                .into_iter()
                .map(|(id, entry)| (id.clone(), entry.clone()))
                .collect();
            
            if filtered_entries.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    if self.search_query.is_empty() {
                        ui.label("📭 No entries yet");
                        ui.add_space(SPACING);
                        ui.label("Click 'Add' to create your first entry");
                    } else {
                        ui.label("🔍 No matching entries");
                        ui.add_space(SPACING);
                        ui.label(format!("No entries match \"{}\"", self.search_query));
                    }
                });
            } else {
                for (id, entry) in filtered_entries.iter() {
                    self.render_entry_card(ui, ctx, id, entry);
                    ui.add_space(SPACING);
                }
            }
        });
    }

    /// Render a single entry card
    fn render_entry_card(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, id: &str, entry: &crate::model::Entry) {
        let frame_fill = theme::frame_fill(&self.current_theme);
        
        egui::Frame::none()
            .fill(frame_fill)
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::same(12.0))
            .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Entry info (left side)
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.strong(format!("🔑 {}", id));
                    });
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label("👤");
                        ui.label(&entry.username);
                    });
                    ui.horizontal(|ui| {
                        ui.label("🔒");
                        if *self.show_password.get(id).unwrap_or(&false) {
                            // Display password as non-selectable label with monospace font
                            ui.add(egui::Label::new(
                                egui::RichText::new(&entry.password)
                                    .monospace()
                                    .color(egui::Color32::from_rgb(255, 180, 100))
                            ).selectable(false));
                        } else {
                            ui.label("••••••••••••");
                        }
                    });
                    if let Some(note) = &entry.note {
                        if !note.is_empty() {
                            ui.horizontal(|ui| {
                                ui.label("📝");
                                ui.label(note);
                            });
                        }
                    }
                });
                
                // Action buttons (right side)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.danger_button(ui, "🗑", [35.0, 30.0]).clicked() {
                        self.pending_delete = Some(id.to_string());
                    }
                    
                    if self.primary_button(ui, "📋 Copy", [70.0, 30.0]).clicked() {
                        match self.secure_clipboard.copy_password(&entry.password) {
                            Ok(()) => {
                                let timeout = self.clipboard_clear_secs;
                                self.toast_success(format!("Password copied! Auto-clear in {}s", timeout));
                            }
                            Err(_) => {
                                ctx.output_mut(|o| o.copied_text = entry.password.clone());
                                self.toast_info("Password copied (standard clipboard)");
                            }
                        }
                    }
                    
                    if self.success_button(ui, "✏️", [35.0, 30.0]).clicked() {
                        self.start_edit_entry(id);
                    }
                    
                    let eye_text = if *self.show_password.get(id).unwrap_or(&false) { "🙈" } else { "👁" };
                    if self.secondary_button(ui, eye_text, [35.0, 30.0]).clicked() {
                        let current_shown_state = self.show_password.entry(id.to_string()).or_insert(false);
                        *current_shown_state = !*current_shown_state;
                    }
                });
            });
        });
    }
}
