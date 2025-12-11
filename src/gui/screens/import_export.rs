//! Import/Export Screen Module
//!
//! Data import and export functionality.

use eframe::egui;
use crate::vault::VaultManager;
use crate::import_export::ImportExportManager;
use super::super::types::{Screen, ExportFormat, ImportFormat, SPACING, PADDING, INPUT_WIDTH, BUTTON_HEIGHT};
use super::super::app::PassmanApp;

impl PassmanApp {
    /// Show import/export screen
    pub fn show_import_export_screen(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Import/Export Data");
        });
        ui.separator();
        ui.add_space(PADDING);

        ui.vertical_centered(|ui| {
            ui.add_space(SPACING * 2.0);
            
            // Export section
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.heading("Export Vault");
                    ui.add_space(SPACING);
                    
                    ui.horizontal(|ui| {
                        ui.label("Export file path:");
                        ui.add(egui::TextEdit::singleline(&mut self.export_file_path)
                            .desired_width(INPUT_WIDTH)
                            .hint_text("e.g., passwords_backup.json"));
                    });
                    ui.add_space(SPACING);
                    
                    ui.horizontal(|ui| {
                        ui.label("Format:");
                        egui::ComboBox::from_label("")
                            .selected_text(match self.export_format {
                                ExportFormat::Json => "JSON",
                                ExportFormat::Csv => "CSV",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.export_format, ExportFormat::Json, "JSON");
                                ui.selectable_value(&mut self.export_format, ExportFormat::Csv, "CSV");
                            });
                    });
                    ui.add_space(SPACING);
                    
                    if self.success_button(ui, "Export", [120.0, BUTTON_HEIGHT]).clicked() {
                        if self.export_file_path.trim().is_empty() {
                            self.toast_error("Please specify export file path");
                        } else if let Some(vault) = &self.vault {
                            match self.export_format {
                                ExportFormat::Json => {
                                    match ImportExportManager::export_json(vault, &self.export_file_path) {
                                        Ok(()) => {
                                            self.toast_success("Data exported successfully!");
                                        }
                                        Err(e) => {
                                            self.toast_error(format!("Export failed: {}", e));
                                        }
                                    }
                                }
                                ExportFormat::Csv => {
                                    match ImportExportManager::export_csv(vault, &self.export_file_path) {
                                        Ok(()) => {
                                            self.toast_success("Data exported successfully!");
                                        }
                                        Err(e) => {
                                            self.toast_error(format!("Export failed: {}", e));
                                        }
                                    }
                                }
                            }
                        } else {
                            self.toast_error("No vault loaded");
                        }
                    }
                });
            });
            
            ui.add_space(SPACING * 2.0);
            
            // Import section
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.heading("Import Data");
                    ui.add_space(SPACING);
                    
                    ui.horizontal(|ui| {
                        ui.label("Import file path:");
                        ui.add(egui::TextEdit::singleline(&mut self.import_file_path)
                            .desired_width(INPUT_WIDTH)
                            .hint_text("e.g., passwords.json"));
                    });
                    ui.add_space(SPACING);
                    
                    ui.horizontal(|ui| {
                        ui.label("Format:");
                        egui::ComboBox::from_label("")
                            .selected_text(match self.import_format {
                                ImportFormat::Json => "JSON",
                                ImportFormat::Csv => "CSV",
                                ImportFormat::Chrome => "Chrome",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.import_format, ImportFormat::Json, "JSON");
                                ui.selectable_value(&mut self.import_format, ImportFormat::Csv, "CSV");
                                ui.selectable_value(&mut self.import_format, ImportFormat::Chrome, "Chrome");
                            });
                    });
                    ui.add_space(SPACING);
                    
                    ui.checkbox(&mut self.merge_on_import, "Merge with existing entries (otherwise replace all)");
                    ui.add_space(SPACING);
                    
                    if self.primary_button(ui, "Import", [120.0, BUTTON_HEIGHT]).clicked() {
                        if self.import_file_path.trim().is_empty() {
                            self.toast_error("Please specify import file path");
                        } else if self.vault.is_some() {
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
                                    // Reload the vault since import functions save it themselves
                                    match VaultManager::load(&self.master_password, Some(&self.vault_file)) {
                                        Ok(vault) => {
                                            self.vault = Some(vault);
                                            self.load_entries();
                                            self.toast_success("Data imported successfully!");
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
                        } else {
                            self.toast_error("No vault loaded");
                        }
                    }
                });
            });
            
            ui.add_space(SPACING * 2.0);
            
            if self.secondary_button(ui, "Back", [150.0, BUTTON_HEIGHT]).clicked() {
                self.current_screen = Screen::Main;
            }
        });
    }
}
