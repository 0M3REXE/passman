//! Import/Export Screen Module
//!
//! Data import and export functionality with native file dialogs.

use eframe::egui;
use crate::vault::VaultManager;
use crate::import_export::ImportExportManager;
use super::super::types::{Screen, ExportFormat, ImportFormat, SPACING};
use super::super::theme;
use super::super::app::PassmanApp;

impl PassmanApp {
    /// Show import/export screen
    pub fn show_import_export_screen(&mut self, ui: &mut egui::Ui) {
        let current_theme = self.current_theme.clone();
        let header_bg = theme::header_bg_color(&current_theme);
        let frame_fill = theme::frame_fill(&current_theme);
        let border_color = theme::border_color(&current_theme);
        let muted_color = theme::muted_text_color(&current_theme);
        
        // ════════════════════════════════════════════════════════════════════
        // HEADER
        // ════════════════════════════════════════════════════════════════════
        egui::Frame::none()
            .fill(header_bg)
            .inner_margin(egui::Margin::symmetric(16.0, 12.0))
            .rounding(egui::Rounding::same(10.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("📦").size(24.0));
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Import / Export").size(20.0).strong());
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let back_btn = egui::Button::new("Back")
                            .fill(egui::Color32::from_rgb(55, 65, 81))
                            .stroke(egui::Stroke::new(1.0, border_color))
                            .rounding(egui::Rounding::same(6.0))
                            .min_size(egui::vec2(70.0, 28.0));
                        
                        if ui.add(back_btn).clicked() {
                            self.current_screen = Screen::Main;
                        }
                    });
                });
            });
        
        ui.add_space(SPACING * 2.0);
        
        // Two-column layout for export and import
        ui.columns(2, |columns| {
            // ════════════════════════════════════════════════════════════════
            // EXPORT SECTION (Left Column)
            // ════════════════════════════════════════════════════════════════
            columns[0].vertical(|ui| {
                egui::Frame::none()
                    .fill(frame_fill)
                    .stroke(egui::Stroke::new(1.0, border_color))
                    .rounding(egui::Rounding::same(12.0))
                    .inner_margin(egui::Margin::same(20.0))
                    .show(ui, |ui| {
                        // Section header
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("📤").size(20.0));
                            ui.add_space(8.0);
                            ui.label(egui::RichText::new("Export Vault").size(16.0).strong());
                        });
                        
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new("Save your passwords to a file").size(12.0).color(muted_color));
                        
                        ui.add_space(SPACING * 1.5);
                        ui.separator();
                        ui.add_space(SPACING);
                        
                        // Format selection
                        ui.label(egui::RichText::new("Format").size(13.0).strong());
                        ui.add_space(4.0);
                        
                        ui.horizontal(|ui| {
                            ui.selectable_value(&mut self.export_format, ExportFormat::Json, "📄 JSON");
                            ui.add_space(8.0);
                            ui.selectable_value(&mut self.export_format, ExportFormat::Csv, "📊 CSV");
                        });
                        
                        ui.add_space(SPACING);
                        
                        // File path with browse button
                        ui.label(egui::RichText::new("Destination").size(13.0).strong());
                        ui.add_space(4.0);
                        
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut self.export_file_path)
                                    .hint_text("Select file location...")
                                    .desired_width(ui.available_width() - 90.0)
                            );
                            
                            if self.secondary_button(ui, "📁 Browse", [80.0, 28.0]).clicked() {
                                let extension = match self.export_format {
                                    ExportFormat::Json => "json",
                                    ExportFormat::Csv => "csv",
                                };
                                
                                let filter_name = match self.export_format {
                                    ExportFormat::Json => "JSON files",
                                    ExportFormat::Csv => "CSV files",
                                };
                                
                                if let Some(path) = rfd::FileDialog::new()
                                    .set_title("Export Passwords")
                                    .add_filter(filter_name, &[extension])
                                    .add_filter("All files", &["*"])
                                    .set_file_name(&format!("passwords_backup.{}", extension))
                                    .save_file()
                                {
                                    self.export_file_path = path.display().to_string();
                                }
                            }
                        });
                        
                        ui.add_space(SPACING * 1.5);
                        
                        // Export button
                        ui.vertical_centered(|ui| {
                            let button = egui::Button::new(
                                egui::RichText::new("⬇ Export").size(14.0).color(egui::Color32::WHITE)
                            )
                            .fill(egui::Color32::from_rgb(34, 197, 94))
                            .rounding(egui::Rounding::same(8.0))
                            .min_size(egui::vec2(140.0, 40.0));
                            
                            if ui.add(button).clicked() {
                                self.do_export();
                            }
                        });
                        
                        ui.add_space(SPACING);
                        
                        // Info text
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new("⚠ Exported files are NOT encrypted")
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(251, 191, 36))
                            );
                        });
                    });
            });
            
            // ════════════════════════════════════════════════════════════════
            // IMPORT SECTION (Right Column)
            // ════════════════════════════════════════════════════════════════
            columns[1].vertical(|ui| {
                egui::Frame::none()
                    .fill(frame_fill)
                    .stroke(egui::Stroke::new(1.0, border_color))
                    .rounding(egui::Rounding::same(12.0))
                    .inner_margin(egui::Margin::same(20.0))
                    .show(ui, |ui| {
                        // Section header
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("📥").size(20.0));
                            ui.add_space(8.0);
                            ui.label(egui::RichText::new("Import Data").size(16.0).strong());
                        });
                        
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new("Load passwords from a file").size(12.0).color(muted_color));
                        
                        ui.add_space(SPACING * 1.5);
                        ui.separator();
                        ui.add_space(SPACING);
                        
                        // Format selection
                        ui.label(egui::RichText::new("Format").size(13.0).strong());
                        ui.add_space(4.0);
                        
                        ui.horizontal(|ui| {
                            ui.selectable_value(&mut self.import_format, ImportFormat::Json, "📄 JSON");
                            ui.add_space(4.0);
                            ui.selectable_value(&mut self.import_format, ImportFormat::Csv, "📊 CSV");
                            ui.add_space(4.0);
                            ui.selectable_value(&mut self.import_format, ImportFormat::Chrome, "🌐 Chrome");
                        });
                        
                        ui.add_space(SPACING);
                        
                        // File path with browse button
                        ui.label(egui::RichText::new("Source File").size(13.0).strong());
                        ui.add_space(4.0);
                        
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut self.import_file_path)
                                    .hint_text("Select file to import...")
                                    .desired_width(ui.available_width() - 90.0)
                            );
                            
                            if self.secondary_button(ui, "📁 Browse", [80.0, 28.0]).clicked() {
                                let (filter_name, extensions): (&str, Vec<&str>) = match self.import_format {
                                    ImportFormat::Json => ("JSON files", vec!["json"]),
                                    ImportFormat::Csv => ("CSV files", vec!["csv"]),
                                    ImportFormat::Chrome => ("CSV files", vec!["csv"]),
                                };
                                
                                if let Some(path) = rfd::FileDialog::new()
                                    .set_title("Import Passwords")
                                    .add_filter(filter_name, &extensions)
                                    .add_filter("All files", &["*"])
                                    .pick_file()
                                {
                                    self.import_file_path = path.display().to_string();
                                }
                            }
                        });
                        
                        ui.add_space(SPACING);
                        
                        // Merge option
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut self.merge_on_import, "");
                            ui.label("Merge with existing entries");
                        });
                        ui.label(
                            egui::RichText::new(if self.merge_on_import {
                                "New entries will be added, existing ones kept"
                            } else {
                                "⚠ All existing entries will be replaced"
                            })
                            .size(11.0)
                            .color(if self.merge_on_import { muted_color } else { egui::Color32::from_rgb(251, 191, 36) })
                        );
                        
                        ui.add_space(SPACING * 1.5);
                        
                        // Import button
                        ui.vertical_centered(|ui| {
                            let button = egui::Button::new(
                                egui::RichText::new("⬆ Import").size(14.0).color(egui::Color32::WHITE)
                            )
                            .fill(egui::Color32::from_rgb(59, 130, 246))
                            .rounding(egui::Rounding::same(8.0))
                            .min_size(egui::vec2(140.0, 40.0));
                            
                            if ui.add(button).clicked() {
                                self.do_import();
                            }
                        });
                        
                        ui.add_space(SPACING);
                        
                        // Format help
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new("Supports Passman JSON, CSV, and Chrome exports")
                                    .size(11.0)
                                    .color(muted_color)
                            );
                        });
                    });
            });
        });
    }
    
    /// Execute export operation
    fn do_export(&mut self) {
        if self.export_file_path.trim().is_empty() {
            self.toast_error("Please select a destination file");
            return;
        }
        
        let Some(vault) = &self.vault else {
            self.toast_error("No vault loaded");
            return;
        };
        
        let result = match self.export_format {
            ExportFormat::Json => ImportExportManager::export_json(vault, &self.export_file_path),
            ExportFormat::Csv => ImportExportManager::export_csv(vault, &self.export_file_path),
        };
        
        match result {
            Ok(()) => {
                self.toast_success(format!("Exported to {}", self.export_file_path));
                self.export_file_path.clear();
            }
            Err(e) => {
                self.toast_error(format!("Export failed: {}", e));
            }
        }
    }
    
    /// Execute import operation
    fn do_import(&mut self) {
        if self.import_file_path.trim().is_empty() {
            self.toast_error("Please select a file to import");
            return;
        }
        
        if self.vault.is_none() {
            self.toast_error("No vault loaded");
            return;
        }
        
        let result = match self.import_format {
            ImportFormat::Json => {
                ImportExportManager::import_json(&self.import_file_path, &self.master_password, Some(&self.vault_file), self.merge_on_import)
            }
            ImportFormat::Csv => {
                ImportExportManager::import_csv(&self.import_file_path, &self.master_password, Some(&self.vault_file), self.merge_on_import)
            }
            ImportFormat::Chrome => {
                ImportExportManager::import_browser(&self.import_file_path, &self.master_password, Some(&self.vault_file), "chrome", self.merge_on_import)
            }
        };
        
        match result {
            Ok(()) => {
                // Reload the vault
                match VaultManager::load(&self.master_password, Some(&self.vault_file)) {
                    Ok(vault) => {
                        let count = vault.entries.len();
                        self.vault = Some(vault);
                        self.load_entries();
                        self.toast_success(format!("Imported successfully! {} entries total", count));
                        self.import_file_path.clear();
                    }
                    Err(e) => {
                        self.toast_error(format!("Import succeeded but reload failed: {}", e));
                    }
                }
            }
            Err(e) => {
                self.toast_error(format!("Import failed: {}", e));
            }
        }
    }
}
