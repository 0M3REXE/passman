//! Main Screen Module
//!
//! Main vault screen with entry list and search.

use eframe::egui;
use super::super::types::{Screen, SPACING};
use super::super::theme;
use super::super::widgets;
use super::super::app::PassmanApp;

impl PassmanApp {
    /// Show main vault screen
    pub fn show_main_screen(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // Clone theme to avoid borrow issues
        let current_theme = self.current_theme.clone();
        
        // ════════════════════════════════════════════════════════════════════
        // HEADER BAR
        // ════════════════════════════════════════════════════════════════════
        egui::Frame::none()
            .fill(theme::header_bg_color(&current_theme))
            .inner_margin(egui::Margin::symmetric(16.0, 12.0))
            .rounding(egui::Rounding::same(10.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Title section
                    ui.label(egui::RichText::new("🔐").size(24.0));
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Password Vault").size(20.0).strong());
                    
                    // Keyboard shortcuts hint
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("⌨")
                            .size(14.0)
                            .color(theme::muted_text_color(&current_theme))
                    ).on_hover_text(
                        "Keyboard Shortcuts:\n\
                        • Ctrl+N - New entry\n\
                        • Ctrl+F - Focus search\n\
                        • Ctrl+L - Lock vault\n\
                        • Ctrl+H - Health dashboard\n\
                        • Ctrl+S - Settings\n\
                        • Escape - Go back"
                    );
                    
                    // Right-aligned buttons
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        
                        // Lock button
                        if self.secondary_button(ui, "Lock", [65.0, 32.0]).clicked() {
                            self.lock_vault();
                            self.toast_info("Vault locked".to_string());
                        }
                        
                        // Settings
                        if self.secondary_button(ui, "⚙", [36.0, 32.0]).clicked() {
                            self.current_screen = Screen::Settings;
                        }
                        
                        ui.add_space(4.0);
                        
                        // Health dashboard
                        if self.primary_button(ui, "Health", [70.0, 32.0]).clicked() {
                            self.current_screen = Screen::HealthDashboard;
                        }
                        
                        // Export
                        if self.secondary_button(ui, "Export", [70.0, 32.0]).clicked() {
                            self.current_screen = Screen::ImportExport;
                        }
                        
                        ui.add_space(4.0);
                        
                        // Add button (prominent)
                        if self.success_button(ui, "+ Add", [65.0, 32.0]).clicked() {
                            self.current_screen = Screen::AddEntry;
                            self.clear_add_form();
                        }
                    });
                });
            });
        
        ui.add_space(SPACING);
        
        // Get colors upfront
        let search_bg = theme::search_bg_color(&current_theme);
        let border_col = theme::border_color(&current_theme);
        let muted_col = theme::muted_text_color(&current_theme);
        let frame_col = theme::frame_fill(&current_theme);
        
        // ════════════════════════════════════════════════════════════════════
        // SEARCH BAR
        // ════════════════════════════════════════════════════════════════════
        ui.horizontal(|ui| {
            // Styled search bar
            let search_response = egui::Frame::none()
                .fill(search_bg)
                .rounding(egui::Rounding::same(8.0))
                .stroke(egui::Stroke::new(1.0, border_col))
                .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("🔍").size(14.0).color(muted_col));
                        ui.add_space(6.0);
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut self.search_query)
                                .hint_text("Search entries... (Ctrl+F)")
                                .frame(false)
                                .desired_width(220.0)
                        );
                        response
                    }).inner
                }).inner;
            
            // Request focus from keyboard shortcut
            if self.request_search_focus {
                search_response.request_focus();
                self.request_search_focus = false;
            }
            
            // Clear search button
            if !self.search_query.is_empty() {
                if ui.add(
                    egui::Button::new("✕")
                        .fill(egui::Color32::TRANSPARENT)
                        .stroke(egui::Stroke::NONE)
                ).clicked() {
                    self.search_query.clear();
                }
            }
            
            ui.add_space(SPACING * 2.0);
            
            // Entry count badge
            let filtered_count = self.filter_entries().len();
            let total_count = self.entries.len();
            let count_text = if self.search_query.is_empty() {
                format!("{} entries", total_count)
            } else {
                format!("{} of {}", filtered_count, total_count)
            };
            
            egui::Frame::none()
                .fill(frame_col)
                .rounding(egui::Rounding::same(12.0))
                .inner_margin(egui::Margin::symmetric(10.0, 4.0))
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(count_text).size(12.0).color(muted_col));
                });
        });
        
        ui.add_space(SPACING);
        
        // ════════════════════════════════════════════════════════════════════
        // ENTRIES LIST
        // ════════════════════════════════════════════════════════════════════
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
                // Empty state
                if self.search_query.is_empty() {
                    widgets::empty_state(
                        ui,
                        "📭",
                        "No entries yet",
                        "Click '+ Add' to create your first password entry"
                    );
                } else {
                    widgets::empty_state(
                        ui,
                        "🔍",
                        "No matching entries",
                        &format!("No entries match \"{}\"", self.search_query)
                    );
                }
            } else {
                for (id, entry) in filtered_entries.iter() {
                    self.render_entry_card(ui, ctx, id, entry);
                    ui.add_space(8.0);
                }
            }
        });
    }

    /// Render a single entry card
    fn render_entry_card(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, id: &str, entry: &crate::model::Entry) {
        // Get all theme colors upfront to avoid borrow issues
        let current_theme = self.current_theme.clone();
        let frame_fill = theme::frame_fill(&current_theme);
        let border_color = theme::border_color(&current_theme);
        let muted_col = theme::muted_text_color(&current_theme);
        
        let password_str = entry.password_str();
        let strength_score = widgets::calculate_password_score(password_str);
        let strength_color = widgets::strength_color(strength_score);
        
        // Clone data we need for the closure
        let username = entry.username.clone();
        let note = entry.note.clone();
        let show_pwd = *self.show_password.get(id).unwrap_or(&false);
        let password_display = password_str.to_string();
        let id_owned = id.to_string();
        
        egui::Frame::none()
            .fill(frame_fill)
            .rounding(egui::Rounding::same(12.0))
            .stroke(egui::Stroke::new(1.0, border_color))
            .inner_margin(egui::Margin::same(0.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // ─────────────────────────────────────────────────────────
                    // LEFT ACCENT BAR (password strength indicator)
                    // ─────────────────────────────────────────────────────────
                    let (accent_rect, _) = ui.allocate_exact_size(
                        egui::vec2(4.0, 80.0),
                        egui::Sense::hover()
                    );
                    ui.painter().rect_filled(
                        accent_rect,
                        egui::Rounding {
                            nw: 12.0,
                            sw: 12.0,
                            ne: 0.0,
                            se: 0.0,
                        },
                        strength_color
                    );
                    
                    // ─────────────────────────────────────────────────────────
                    // CONTENT AREA
                    // ─────────────────────────────────────────────────────────
                    ui.add_space(12.0);
                    
                    ui.vertical(|ui| {
                        ui.add_space(10.0);
                        
                        // Entry title with strength dots
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(format!("🔑 {}", id_owned)).size(15.0).strong());
                            ui.add_space(8.0);
                            widgets::paint_strength_dots(ui, strength_score);
                        });
                        
                        ui.add_space(6.0);
                        
                        // Username row
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("👤").size(12.0));
                            ui.add_space(4.0);
                            ui.label(egui::RichText::new(&username).color(muted_col));
                        });
                        
                        // Password row
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("🔒").size(12.0));
                            ui.add_space(4.0);
                            if show_pwd {
                                ui.add(egui::Label::new(
                                    egui::RichText::new(&password_display)
                                        .monospace()
                                        .color(egui::Color32::from_rgb(251, 191, 36))
                                ).selectable(false));
                            } else {
                                ui.label(egui::RichText::new("••••••••••••").color(muted_col));
                            }
                        });
                        
                        // Note (if exists)
                        if let Some(ref note_text) = note {
                            if !note_text.is_empty() {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("📝").size(12.0));
                                    ui.add_space(4.0);
                                    let display_note = if note_text.len() > 40 {
                                        format!("{}...", &note_text[..40])
                                    } else {
                                        note_text.clone()
                                    };
                                    ui.label(egui::RichText::new(display_note).size(12.0).color(muted_col));
                                });
                            }
                        }
                        
                        ui.add_space(10.0);
                    });
                    
                    // ─────────────────────────────────────────────────────────
                    // ACTION BUTTONS (right side)
                    // ─────────────────────────────────────────────────────────
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(12.0);
                        ui.spacing_mut().item_spacing.x = 6.0;
                        
                        // Delete button
                        if self.danger_button(ui, "🗑", [36.0, 36.0]).clicked() {
                            self.pending_delete = Some(id.to_string());
                        }
                        
                        // Copy button
                        if self.primary_button(ui, "📋 Copy", [75.0, 36.0]).clicked() {
                            match self.secure_clipboard.copy_password(&password_display) {
                                Ok(()) => {
                                    let timeout = self.clipboard_clear_secs;
                                    self.toast_success(format!("Password copied! Auto-clear in {}s", timeout));
                                }
                                Err(_) => {
                                    ctx.output_mut(|o| o.copied_text = password_display.clone());
                                    self.toast_info("Password copied (standard clipboard)");
                                }
                            }
                        }
                        
                        // Edit button
                        if self.success_button(ui, "✏", [36.0, 36.0]).clicked() {
                            self.start_edit_entry(id);
                        }
                        
                        // Show/hide password button
                        let eye_icon = if show_pwd { "🙈" } else { "👁" };
                        if self.secondary_button(ui, eye_icon, [36.0, 36.0]).clicked() {
                            let current = self.show_password.entry(id.to_string()).or_insert(false);
                            *current = !*current;
                        }
                    });
                });
            });
    }
}
