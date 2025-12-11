//! Application State Module
//!
//! Core PassmanApp struct and state management.

#![allow(dead_code)]

use eframe::egui;
use std::collections::HashMap;
use std::time::Instant;
use zeroize::Zeroizing;

use crate::model::{Entry, Vault};
use crate::vault::{VaultManager, SecurityManager};
use crate::utils::generate_password;
use crate::health::PasswordHealthAnalyzer;
use crate::secure_clipboard::SecureClipboard;
use crate::config::get_config;

use super::types::*;
use super::theme;
use super::toasts;
use super::overlays;
use super::widgets;

/// Main application state
pub struct PassmanApp {
    // App state
    pub current_screen: Screen,
    pub vault: Option<Vault>,
    pub vault_file: String,
    pub master_password: Zeroizing<String>,
    
    // Security state
    pub security_manager: SecurityManager,
    pub secure_clipboard: SecureClipboard,
    pub last_activity: Option<Instant>,
    pub lock_timeout_secs: u64,
    pub clipboard_clear_secs: u64,
    
    // UI state
    pub show_password: HashMap<String, bool>,
    pub entries: Vec<(String, Entry)>,
    
    // Form fields
    pub init_password: Zeroizing<String>,
    pub init_confirm: Zeroizing<String>,
    pub login_password: Zeroizing<String>,
    pub add_id: String,
    pub add_username: String,
    pub add_password: String,
    pub add_note: String,
    pub generate_password: bool,
    pub add_show_password: bool,
    pub password_length: usize,
    
    // Form validation errors
    pub form_errors: HashMap<String, String>,
    
    // Edit entry fields
    pub edit_id: String,
    pub edit_username: String,
    pub edit_password: String,
    pub edit_note: String,
    pub edit_generate_password: bool,
    pub edit_show_password: bool,
    
    // Confirmation dialog
    pub pending_delete: Option<String>,
    
    // Search and filtering
    pub search_query: String,
    
    // Password strength
    pub password_strength: String,
    pub password_suggestions: Vec<String>,
    
    // Health dashboard
    pub health_analyzer: PasswordHealthAnalyzer,
    
    // Import/Export fields
    pub export_file_path: String,
    pub import_file_path: String,
    pub export_format: ExportFormat,
    pub import_format: ImportFormat,
    pub merge_on_import: bool,
    
    // Password change fields
    pub change_current_password: Zeroizing<String>,
    pub change_new_password: Zeroizing<String>,
    pub change_confirm_password: Zeroizing<String>,
    pub show_password_change: bool,
    
    // Theme
    pub current_theme: Theme,
    
    // Keyboard shortcut state
    pub request_search_focus: bool,
    
    // Loading state
    pub is_loading: bool,
    pub loading_message: String,
    
    // Onboarding
    pub show_onboarding: bool,
    pub onboarding_step: u8,
    
    // Toast notifications
    pub toasts: Vec<Toast>,
}

impl Default for PassmanApp {
    fn default() -> Self {
        Self {
            current_screen: Screen::default(),
            vault: None,
            vault_file: String::new(),
            master_password: Zeroizing::new(String::new()),
            security_manager: SecurityManager::new(),
            secure_clipboard: SecureClipboard::new(),
            last_activity: None,
            lock_timeout_secs: 0,
            clipboard_clear_secs: 30,
            show_password: HashMap::new(),
            entries: Vec::new(),
            init_password: Zeroizing::new(String::new()),
            init_confirm: Zeroizing::new(String::new()),
            login_password: Zeroizing::new(String::new()),
            add_id: String::new(),
            add_username: String::new(),
            add_password: String::new(),
            add_note: String::new(),
            generate_password: false,
            add_show_password: false,
            password_length: 16,
            form_errors: HashMap::new(),
            edit_id: String::new(),
            edit_username: String::new(),
            edit_password: String::new(),
            edit_note: String::new(),
            edit_generate_password: false,
            edit_show_password: false,
            pending_delete: None,
            search_query: String::new(),
            password_strength: String::new(),
            password_suggestions: Vec::new(),
            health_analyzer: PasswordHealthAnalyzer::new(),
            export_file_path: String::new(),
            import_file_path: String::new(),
            export_format: ExportFormat::default(),
            import_format: ImportFormat::default(),
            merge_on_import: false,
            change_current_password: Zeroizing::new(String::new()),
            change_new_password: Zeroizing::new(String::new()),
            change_confirm_password: Zeroizing::new(String::new()),
            show_password_change: false,
            current_theme: Theme::default(),
            request_search_focus: false,
            is_loading: false,
            loading_message: String::new(),
            show_onboarding: false,
            onboarding_step: 0,
            toasts: Vec::new(),
        }
    }
}

impl PassmanApp {
    /// Create new application with configuration
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let config = get_config();
        
        // Dark theme only
        let initial_theme = Theme::Dark;
        
        let vault_exists = std::path::Path::new(&config.general.default_vault).exists();

        let app = Self {
            vault_file: config.general.default_vault.clone(),
            password_length: config.password.default_length,
            lock_timeout_secs: config.security.lock_timeout_secs,
            clipboard_clear_secs: config.security.clipboard_timeout_secs,
            secure_clipboard: SecureClipboard::with_timeout(config.security.clipboard_timeout_secs),
            current_theme: initial_theme,
            show_onboarding: !vault_exists,
            ..Default::default()
        };
        
        theme::apply_theme(&app.current_theme, &cc.egui_ctx);
        
        app
    }

    // === Toast Methods ===
    
    pub fn add_toast(&mut self, message: impl Into<String>, toast_type: ToastType) {
        self.toasts.push(Toast::new(message, toast_type));
    }
    
    pub fn toast_success(&mut self, message: impl Into<String>) {
        self.add_toast(message, ToastType::Success);
    }
    
    pub fn toast_error(&mut self, message: impl Into<String>) {
        self.add_toast(message, ToastType::Error);
    }
    
    pub fn toast_info(&mut self, message: impl Into<String>) {
        self.add_toast(message, ToastType::Info);
    }
    
    #[allow(dead_code)]
    pub fn toast_warning(&mut self, message: impl Into<String>) {
        self.add_toast(message, ToastType::Warning);
    }
    
    fn cleanup_toasts(&mut self) {
        self.toasts.retain(|t| !t.is_expired());
    }
    
    // === Form Error Methods ===
    
    pub fn set_form_error(&mut self, field: &str, message: impl Into<String>) {
        self.form_errors.insert(field.to_string(), message.into());
    }
    
    pub fn clear_form_error(&mut self, field: &str) {
        self.form_errors.remove(field);
    }
    
    pub fn clear_form_errors(&mut self) {
        self.form_errors.clear();
    }
    
    pub fn show_field_error(&self, ui: &mut egui::Ui, field: &str) {
        widgets::show_field_error(ui, &self.form_errors, field);
    }
    
    // === Loading Methods ===
    
    #[allow(dead_code)]
    pub fn start_loading(&mut self, message: impl Into<String>) {
        self.is_loading = true;
        self.loading_message = message.into();
    }
    
    #[allow(dead_code)]
    pub fn stop_loading(&mut self) {
        self.is_loading = false;
        self.loading_message.clear();
    }
    
    // === Validation Methods ===
    
    pub fn validate_add_entry(&mut self) -> bool {
        self.clear_form_errors();
        let mut is_valid = true;
        
        if self.add_id.trim().is_empty() {
            self.set_form_error("add_id", "Entry ID is required");
            is_valid = false;
        } else if self.vault.as_ref().is_some_and(|v| v.entries.contains_key(&self.add_id)) {
            self.set_form_error("add_id", "Entry ID already exists");
            is_valid = false;
        }
        
        if self.add_username.trim().is_empty() {
            self.set_form_error("add_username", "Username is required");
            is_valid = false;
        }
        
        if !self.generate_password && self.add_password.trim().is_empty() {
            self.set_form_error("add_password", "Password is required");
            is_valid = false;
        }
        
        is_valid
    }
    
    pub fn validate_edit_entry(&mut self) -> bool {
        self.clear_form_errors();
        let mut is_valid = true;
        
        if self.edit_username.trim().is_empty() {
            self.set_form_error("edit_username", "Username is required");
            is_valid = false;
        }
        
        if !self.edit_generate_password && self.edit_password.trim().is_empty() {
            self.set_form_error("edit_password", "Password is required");
            is_valid = false;
        }
        
        is_valid
    }
    
    // === Vault Operations ===
    
    pub fn lock_vault(&mut self) {
        self.vault = None;
        *self.master_password = String::new();
        self.entries.clear();
        self.show_password.clear();
        self.last_activity = None;
        self.current_screen = Screen::Welcome;
        let _ = self.secure_clipboard.clear_now();
    }
    
    pub fn load_entries(&mut self) {
        if let Some(vault) = &self.vault {
            self.entries = vault.list_entries()
                .into_iter()
                .filter_map(|id| {
                    vault.get_entry(id).map(|entry| (id.clone(), entry.clone()))
                })
                .collect();
            self.entries.sort_by(|a, b| a.0.cmp(&b.0));
        }
    }

    pub fn filter_entries(&self) -> Vec<&(String, Entry)> {
        if self.search_query.is_empty() {
            self.entries.iter().collect()
        } else {
            self.entries
                .iter()
                .filter(|(id, entry)| {
                    id.to_lowercase().contains(&self.search_query.to_lowercase())
                        || entry.username.to_lowercase().contains(&self.search_query.to_lowercase())
                })
                .collect()
        }
    }

    pub fn init_vault(&mut self) -> Result<(), String> {
        if self.init_password.as_str() != self.init_confirm.as_str() {
            return Err("Passwords do not match!".into());
        }

        if self.init_password.len() < 8 {
            return Err("Password must be at least 8 characters long!".into());
        }

        VaultManager::init(&self.init_password, Some(&self.vault_file))
            .map_err(|e| e.to_string())?;

        *self.master_password = self.init_password.to_string();
        self.vault = Some(Vault::new());
        self.load_entries();
        self.current_screen = Screen::Main;
        *self.init_password = String::new();
        *self.init_confirm = String::new();

        Ok(())
    }

    pub fn login(&mut self) -> Result<(), String> {
        if self.login_password.trim().is_empty() {
            return Err("Please enter your master password".into());
        }
        
        if self.security_manager.is_locked_out() {
            let remaining = self.security_manager.lockout_remaining_secs();
            return Err(format!("Account locked. Please wait {} seconds.", remaining));
        }

        match VaultManager::load(&self.login_password, Some(&self.vault_file)) {
            Ok(vault) => {
                self.security_manager.record_successful_login();
                *self.master_password = self.login_password.to_string();
                self.vault = Some(vault);
                self.load_entries();
                self.current_screen = Screen::Main;
                *self.login_password = String::new();
                self.last_activity = Some(Instant::now());
                Ok(())
            }
            Err(e) => {
                self.security_manager.record_failed_attempt();
                *self.login_password = String::new();
                
                if self.security_manager.is_locked_out() {
                    let remaining = self.security_manager.lockout_remaining_secs();
                    Err(format!("Too many failed attempts. Locked for {} seconds.", remaining))
                } else {
                    let remaining_attempts = self.security_manager.remaining_attempts();
                    Err(format!("{} ({} attempts remaining)", e, remaining_attempts))
                }
            }
        }
    }

    pub fn add_entry(&mut self) -> Result<(), String> {
        if let Some(vault) = &mut self.vault {
            if self.add_id.trim().is_empty() {
                return Err("Entry ID cannot be empty!".into());
            }
            if self.add_username.trim().is_empty() {
                return Err("Username cannot be empty!".into());
            }
            if !self.generate_password && self.add_password.trim().is_empty() {
                return Err("Password cannot be empty!".into());
            }

            if vault.get_entry(&self.add_id).is_some() {
                return Err(format!("Entry '{}' already exists!", self.add_id));
            }

            let password = if self.generate_password {
                generate_password(self.password_length)
            } else {
                self.add_password.clone()
            };

            let note = if self.add_note.is_empty() {
                None
            } else {
                Some(self.add_note.clone())
            };

            let entry = Entry::new(self.add_username.clone(), password, note);
            vault.add_entry(self.add_id.clone(), entry);

            VaultManager::save(vault, &self.master_password, Some(&self.vault_file))
                .map_err(|e| e.to_string())?;

            self.load_entries();
            self.current_screen = Screen::Main;
            self.clear_add_form();

            Ok(())
        } else {
            Err("No vault loaded".into())
        }
    }

    pub fn remove_entry(&mut self, id: &str) -> Result<(), String> {
        if let Some(vault) = &mut self.vault {
            vault.remove_entry(id).ok_or("Entry not found")?;

            VaultManager::save(vault, &self.master_password, Some(&self.vault_file))
                .map_err(|e| e.to_string())?;

            self.load_entries();
            Ok(())
        } else {
            Err("No vault loaded".into())
        }
    }

    pub fn clear_add_form(&mut self) {
        self.add_id.clear();
        self.add_username.clear();
        self.add_password.clear();
        self.add_note.clear();
        self.generate_password = false;
        self.add_show_password = false;
        self.password_strength.clear();
        self.password_suggestions.clear();
    }

    pub fn start_edit_entry(&mut self, id: &str) {
        if let Some(vault) = &self.vault {
            if let Some(entry) = vault.get_entry(id) {
                self.edit_id = id.to_string();
                self.edit_username = entry.username.clone();
                self.edit_password = entry.password_str().to_string();
                self.edit_note = entry.note.clone().unwrap_or_default();
                self.current_screen = Screen::EditEntry(id.to_string());
            }
        }
    }

    pub fn update_entry(&mut self) -> Result<(), String> {
        if let Some(vault) = &mut self.vault {
            if self.edit_username.trim().is_empty() {
                return Err("Username cannot be empty!".into());
            }
            if !self.edit_generate_password && self.edit_password.trim().is_empty() {
                return Err("Password cannot be empty!".into());
            }

            let password = if self.edit_generate_password {
                generate_password(self.password_length)
            } else {
                self.edit_password.clone()
            };

            let note = if self.edit_note.trim().is_empty() {
                None
            } else {
                Some(self.edit_note.clone())
            };

            if let Some(existing_entry) = vault.get_entry(&self.edit_id) {
                let updated_entry = Entry {
                    username: self.edit_username.clone(),
                    password: password.into(),
                    note,
                    created_at: existing_entry.created_at,
                    modified_at: chrono::Utc::now(),
                    tags: existing_entry.tags.clone(),
                    url: existing_entry.url.clone(),
                    totp_secret: existing_entry.totp_secret.clone(),
                };
                
                vault.add_entry(self.edit_id.clone(), updated_entry);
            } else {
                return Err("Entry not found".into());
            }

            VaultManager::save(vault, &self.master_password, Some(&self.vault_file))
                .map_err(|e| e.to_string())?;

            self.load_entries();
            self.current_screen = Screen::Main;
            self.clear_edit_form();
            Ok(())
        } else {
            Err("No vault loaded".into())
        }
    }

    pub fn clear_edit_form(&mut self) {
        self.edit_id.clear();
        self.edit_username.clear();
        self.edit_password.clear();
        self.edit_note.clear();
        self.edit_generate_password = false;
        self.edit_show_password = false;
        self.password_strength.clear();
        self.password_suggestions.clear();
    }
    
    // === Keyboard Shortcuts ===
    
    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            if self.vault.is_some() {
                // Ctrl+N - New entry
                if i.modifiers.ctrl && i.key_pressed(egui::Key::N) && self.current_screen == Screen::Main {
                    self.current_screen = Screen::AddEntry;
                    self.clear_add_form();
                }
                
                // Ctrl+F - Focus search
                if i.modifiers.ctrl && i.key_pressed(egui::Key::F) && self.current_screen == Screen::Main {
                    self.request_search_focus = true;
                }
                
                // Ctrl+L - Lock vault
                if i.modifiers.ctrl && i.key_pressed(egui::Key::L) {
                    self.lock_vault();
                    self.toast_info("Vault locked".to_string());
                }
                
                // Ctrl+H - Health dashboard
                if i.modifiers.ctrl && i.key_pressed(egui::Key::H) && self.current_screen == Screen::Main {
                    self.current_screen = Screen::HealthDashboard;
                }
                
                // Ctrl+S - Settings
                if i.modifiers.ctrl && i.key_pressed(egui::Key::S) && self.current_screen == Screen::Main {
                    self.current_screen = Screen::Settings;
                }
            }
            
            // Escape - Go back
            if i.key_pressed(egui::Key::Escape) {
                match &self.current_screen {
                    Screen::AddEntry | Screen::EditEntry(_) | Screen::Settings | 
                    Screen::HealthDashboard | Screen::ImportExport => {
                        self.current_screen = Screen::Main;
                    }
                    _ => {}
                }
            }
        });
    }
    
    // === Button Helpers ===
    
    pub fn primary_button(&self, ui: &mut egui::Ui, text: &str, size: [f32; 2]) -> egui::Response {
        widgets::ButtonWidgets::primary(ui, text, size)
    }

    pub fn secondary_button(&self, ui: &mut egui::Ui, text: &str, size: [f32; 2]) -> egui::Response {
        widgets::ButtonWidgets::secondary(ui, text, size)
    }

    pub fn success_button(&self, ui: &mut egui::Ui, text: &str, size: [f32; 2]) -> egui::Response {
        widgets::ButtonWidgets::success(ui, text, size)
    }

    pub fn danger_button(&self, ui: &mut egui::Ui, text: &str, size: [f32; 2]) -> egui::Response {
        widgets::ButtonWidgets::danger(ui, text, size)
    }

    pub fn show_password_strength_indicator(&self, ui: &mut egui::Ui, password: &str) {
        widgets::show_password_strength_indicator(ui, password);
    }
}

/// eframe App implementation
impl eframe::App for PassmanApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for session timeout
        if self.vault.is_some() && self.lock_timeout_secs > 0 {
            if let Some(last) = self.last_activity {
                if last.elapsed().as_secs() >= self.lock_timeout_secs {
                    self.lock_vault();
                    self.toast_info(format!("Session timed out after {} seconds of inactivity", self.lock_timeout_secs));
                }
            }
        }
        
        // Update last activity on any input
        if ctx.input(|i| i.pointer.any_click() || i.key_pressed(egui::Key::Enter) || !i.keys_down.is_empty()) {
            self.last_activity = Some(Instant::now());
        }
        
        // Handle keyboard shortcuts
        self.handle_keyboard_shortcuts(ctx);
        
        // Clean up expired toasts
        self.cleanup_toasts();
        
        let panel_fill = theme::panel_fill(&self.current_theme);
        
        egui::CentralPanel::default()
            .frame(egui::Frame::none()
                .inner_margin(PADDING)
                .fill(panel_fill))
            .show(ctx, |ui| {
                match self.current_screen.clone() {
                    Screen::Welcome => self.show_welcome_screen(ui),
                    Screen::Init => self.show_init_screen(ui),
                    Screen::Login => self.show_login_screen(ui),
                    Screen::Main => self.show_main_screen(ui, ctx),
                    Screen::AddEntry => self.show_add_entry_screen(ui),
                    Screen::EditEntry(id) => self.show_edit_entry_screen(ui, &id),
                    Screen::Settings => self.show_settings_screen(ui, ctx),
                    Screen::HealthDashboard => self.show_health_dashboard(ui),
                    Screen::ImportExport => self.show_import_export_screen(ui),
                }
            });
        
        // Render overlays
        overlays::render_loading_overlay(ctx, self.is_loading, &self.loading_message);
        overlays::render_onboarding(ctx, &mut self.show_onboarding, &mut self.onboarding_step);
        
        // Handle confirmation dialog
        if self.pending_delete.is_some() {
            let entry_id = self.pending_delete.clone().unwrap();
            let mut should_delete = false;
            let mut should_cancel = false;
            
            // Modal background overlay
            egui::Area::new(egui::Id::new("confirm_overlay"))
                .anchor(egui::Align2::LEFT_TOP, egui::vec2(0.0, 0.0))
                .order(egui::Order::Middle)
                .show(ctx, |ui| {
                    let screen_rect = ctx.screen_rect();
                    ui.painter().rect_filled(
                        screen_rect,
                        0.0,
                        egui::Color32::from_black_alpha(150),
                    );
                });
            
            // Dialog window
            egui::Window::new("⚠️ Confirm Delete")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    ui.add_space(SPACING);
                    ui.label(format!("Are you sure you want to delete '{}'?", entry_id));
                    ui.add_space(SPACING);
                    ui.label("This action cannot be undone.");
                    ui.add_space(SPACING * 2.0);
                    
                    ui.horizontal(|ui| {
                        if self.danger_button(ui, "Delete", [100.0, BUTTON_HEIGHT]).clicked() {
                            should_delete = true;
                        }
                        
                        ui.add_space(SPACING);
                        
                        if self.secondary_button(ui, "Cancel", [100.0, BUTTON_HEIGHT]).clicked() {
                            should_cancel = true;
                        }
                    });
                });
            
            if should_delete {
                match self.remove_entry(&entry_id) {
                    Ok(()) => {
                        self.toast_success(format!("Entry '{}' deleted", entry_id));
                    }
                    Err(e) => {
                        self.toast_error(e);
                    }
                }
                self.pending_delete = None;
            } else if should_cancel {
                self.pending_delete = None;
            }
        }
        
        toasts::render_toasts(ctx, &self.toasts);
    }
}
