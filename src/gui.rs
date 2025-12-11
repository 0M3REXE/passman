use eframe::egui;
use crate::model::{Entry, Vault};
use crate::vault::{VaultManager, SecurityManager};
use crate::utils::*;
use crate::health::{PasswordHealthAnalyzer, HealthSummary};
use crate::import_export::ImportExportManager;
use crate::secure_clipboard::SecureClipboard;
use crate::config::{Config, get_config};
use std::collections::HashMap;
use std::time::Instant;
use zeroize::Zeroizing;

// Simple UI Constants
const BUTTON_HEIGHT: f32 = 36.0;
const INPUT_WIDTH: f32 = 300.0;
const SPACING: f32 = 10.0;
const PADDING: f32 = 20.0;

#[derive(Default)]
pub struct PassmanApp {
    // App state
    current_screen: Screen,
    vault: Option<Vault>,
    vault_file: String,
    master_password: Zeroizing<String>,
    
    // Security state
    security_manager: SecurityManager,
    secure_clipboard: SecureClipboard,
    last_activity: Option<Instant>,
    lock_timeout_secs: u64,
    clipboard_clear_secs: u64,
    
    // UI state
    show_password: HashMap<String, bool>,
    entries: Vec<(String, Entry)>,
      // Form fields
    init_password: Zeroizing<String>,
    init_confirm: Zeroizing<String>,
    login_password: Zeroizing<String>,
    add_id: String,
    add_username: String,
    add_password: String,
    add_note: String,
    generate_password: bool,
    password_length: usize,
      // Edit entry fields
    edit_id: String,
    edit_username: String,
    edit_password: String,
    edit_note: String,
    edit_generate_password: bool,
    edit_show_password: bool,
    
    // Search and filtering
    search_query: String,
    
    // Messages
    message: String,
    message_type: MessageType,    // Password strength
    password_strength: String,
    password_suggestions: Vec<String>,    // Health dashboard
    health_analyzer: PasswordHealthAnalyzer,
    #[allow(dead_code)]
    health_summary: Option<HealthSummary>,
    
    // Import/Export fields
    export_file_path: String,
    import_file_path: String,
    export_format: ExportFormat,
    import_format: ImportFormat,
    merge_on_import: bool,
    
    // Password change fields
    change_current_password: Zeroizing<String>,
    change_new_password: Zeroizing<String>,
    change_confirm_password: Zeroizing<String>,
    show_password_change: bool,
}

#[derive(Default, PartialEq, Clone, Copy)]
enum ExportFormat {
    #[default]
    Json,
    Csv,
}

#[derive(Default, PartialEq, Clone, Copy)]
enum ImportFormat {
    #[default]
    Json,
    Csv,
    Chrome,
}

#[derive(Default, PartialEq)]
#[allow(dead_code)]
enum Screen {
    #[default]
    Welcome,
    Init,    Login,
    Main,
    AddEntry,
    EditEntry(String),
    Settings,
    HealthDashboard, // New screen for password health
    ImportExport,    // New screen for import/export
}

#[derive(Default, PartialEq)]
#[allow(dead_code)]
enum MessageType {
    #[default]
    None,
    Success,
    Error,
    Info,
}

impl PassmanApp {    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Simple clean style with dark background and white text
        let mut style = (*cc.egui_ctx.style()).clone();
        
        // Dark theme with white text
        style.visuals.dark_mode = true;
        style.visuals.override_text_color = Some(egui::Color32::WHITE);
        style.visuals.window_fill = egui::Color32::from_rgb(32, 33, 36);
        style.visuals.panel_fill = egui::Color32::from_rgb(32, 33, 36);
        
        // Ensure white text for all elements - override_text_color handles this globally.
        // style.visuals.widgets.noninteractive.fg_stroke.color = egui::Color32::WHITE; // Redundant
        // style.visuals.widgets.inactive.fg_stroke.color = egui::Color32::WHITE;    // Redundant
        // style.visuals.widgets.hovered.fg_stroke.color = egui::Color32::WHITE;     // Redundant
        // style.visuals.widgets.active.fg_stroke.color = egui::Color32::WHITE;      // Redundant
        
        // Dark backgrounds
        style.visuals.faint_bg_color = egui::Color32::from_rgb(45, 46, 49);
        style.visuals.code_bg_color = egui::Color32::from_rgb(45, 46, 49);
        
        // Minimal rounded corners
        style.visuals.window_rounding = egui::Rounding::same(6.0);
        style.visuals.widgets.noninteractive.rounding = egui::Rounding::same(4.0);
        style.visuals.widgets.inactive.rounding = egui::Rounding::same(4.0);
        style.visuals.widgets.hovered.rounding = egui::Rounding::same(4.0);
        style.visuals.widgets.active.rounding = egui::Rounding::same(4.0);
        
        // Default widget fill colors for interactive elements (e.g., checkboxes)
        // Buttons will override these with their specific helper functions.
        // Using slightly different shades of grey from panel_fill for subtlety.
        style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(50, 52, 56); // Darker grey for inactive state background
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(60, 62, 66);  // Slightly lighter grey for hovered state background
        style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(70, 72, 76);   // Even lighter grey for active state background
        // Note: fg_stroke.color for these states is already globally set to WHITE.
        
        // Clean input fields with dark backgrounds and visible borders
        style.visuals.extreme_bg_color = egui::Color32::from_rgb(45, 46, 49);
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(45, 46, 49);
        style.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100));
        style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(120, 120, 120));
        style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.5, egui::Color32::from_rgb(150, 150, 150));
        style.visuals.widgets.active.bg_stroke = egui::Stroke::new(1.5, egui::Color32::from_rgb(70, 130, 180));
        style.visuals.selection.bg_fill = egui::Color32::from_rgb(100, 150, 255);
        
        // Simple spacing
        style.spacing.item_spacing = egui::vec2(SPACING, SPACING);
        style.spacing.button_padding = egui::vec2(12.0, 8.0);
        
        // Apply the initial style (which includes visuals, spacing, etc.)
        // cc.egui_ctx.set_visuals(style.visuals.clone()); // Redundant if set_style is called with the same style object
        cc.egui_ctx.set_style(style);
        
        // Load configuration
        let config = get_config();

        Self {
            vault_file: config.general.default_vault.clone(),
            password_length: config.password.default_length,
            master_password: Zeroizing::new(String::new()),
            init_password: Zeroizing::new(String::new()),
            init_confirm: Zeroizing::new(String::new()),
            login_password: Zeroizing::new(String::new()),
            export_file_path: String::new(),
            import_file_path: String::new(),
            export_format: ExportFormat::Json,
            import_format: ImportFormat::Json,
            merge_on_import: false,
            // Security settings from config
            lock_timeout_secs: config.security.lock_timeout_secs,
            clipboard_clear_secs: config.security.clipboard_timeout_secs,
            secure_clipboard: SecureClipboard::with_timeout(config.security.clipboard_timeout_secs),
            security_manager: SecurityManager::new(),
            last_activity: None,
            ..Default::default()
        }
    }

    fn show_message(&mut self, message: String, msg_type: MessageType) {
        self.message = message;
        self.message_type = msg_type;
    }

    fn clear_message(&mut self) {
        self.message.clear();
        self.message_type = MessageType::None;
    }

    /// Lock the vault and clear sensitive data
    fn lock_vault(&mut self) {
        self.vault = None;
        *self.master_password = String::new();
        self.entries.clear();
        self.show_password.clear();
        self.last_activity = None;
        self.current_screen = Screen::Welcome;
        
        // Clear clipboard for security
        let _ = self.secure_clipboard.clear_now();
    }

    fn load_entries(&mut self) {
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

    fn filter_entries(&self) -> Vec<&(String, Entry)> {
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
    }    fn init_vault(&mut self) -> Result<(), String> {
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
    }    fn login(&mut self) -> Result<(), String> {
        // Check if locked out
        if self.security_manager.is_locked_out() {
            let remaining = self.security_manager.lockout_remaining_secs();
            return Err(format!("Account locked. Please wait {} seconds.", remaining));
        }

        match VaultManager::load(&self.login_password, Some(&self.vault_file)) {
            Ok(vault) => {
                // Successful login
                self.security_manager.record_successful_login();
                *self.master_password = self.login_password.to_string();
                self.vault = Some(vault);
                self.load_entries();
                self.current_screen = Screen::Main;
                *self.login_password = String::new();
                // Start session timer
                self.last_activity = Some(Instant::now());
                Ok(())
            }
            Err(e) => {
                // Failed login - record attempt
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
    }    fn add_entry(&mut self) -> Result<(), String> {
        if let Some(vault) = &mut self.vault {
            // Validation: check for empty required fields
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

    fn remove_entry(&mut self, id: &str) -> Result<(), String> {
        if let Some(vault) = &mut self.vault {
            vault.remove_entry(id).ok_or("Entry not found")?;

            VaultManager::save(vault, &self.master_password, Some(&self.vault_file))
                .map_err(|e| e.to_string())?;

            self.load_entries();
            Ok(())
        } else {
            Err("No vault loaded".into())
        }
    }    fn clear_add_form(&mut self) {
        self.add_id.clear();
        self.add_username.clear();
        self.add_password.clear();
        self.add_note.clear();
        self.generate_password = false;
        self.password_strength.clear();
        self.password_suggestions.clear();
    }

    // Edit entry methods
    fn start_edit_entry(&mut self, id: &str) {
        if let Some(vault) = &self.vault {
            if let Some(entry) = vault.get_entry(id) {
                self.edit_id = id.to_string();
                self.edit_username = entry.username.clone();
                self.edit_password = entry.password.clone();
                self.edit_note = entry.note.clone().unwrap_or_default();
                self.current_screen = Screen::EditEntry(id.to_string());
            }
        }
    }    fn update_entry(&mut self) -> Result<(), String> {
        if let Some(vault) = &mut self.vault {
            // Validation: check for empty required fields
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

            // Get the existing entry to preserve metadata
            if let Some(existing_entry) = vault.get_entry(&self.edit_id) {
                let updated_entry = Entry {
                    username: self.edit_username.clone(),
                    password,
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
    }fn clear_edit_form(&mut self) {
        self.edit_id.clear();
        self.edit_username.clear();
        self.edit_password.clear();
        self.edit_note.clear();
        self.edit_generate_password = false;
        self.edit_show_password = false;
        self.password_strength.clear();
        self.password_suggestions.clear();
    }
}

impl eframe::App for PassmanApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for session timeout (only when vault is unlocked)
        if self.vault.is_some() && self.lock_timeout_secs > 0 {
            if let Some(last) = self.last_activity {
                if last.elapsed().as_secs() >= self.lock_timeout_secs {
                    // Session timed out - lock the vault
                    self.lock_vault();
                    self.show_message(
                        format!("Session timed out after {} seconds of inactivity", self.lock_timeout_secs),
                        MessageType::Info
                    );
                }
            }
        }
        
        // Update last activity on any input
        if ctx.input(|i| i.pointer.any_click() || i.key_pressed(egui::Key::Enter) || !i.keys_down.is_empty()) {
            self.last_activity = Some(Instant::now());
        }
        
        egui::CentralPanel::default()
            .frame(egui::Frame::none()
                .inner_margin(PADDING)
                .fill(egui::Color32::from_rgb(32, 33, 36)))
            .show(ctx, |ui| {
                // Colorful message display
                if !self.message.is_empty() {
                    let color = match self.message_type {
                        MessageType::Success => egui::Color32::from_rgb(40, 167, 69),
                        MessageType::Error => egui::Color32::from_rgb(220, 53, 69),
                        MessageType::Info => egui::Color32::from_rgb(23, 162, 184),
                        MessageType::None => egui::Color32::from_rgb(108, 117, 125),
                    };
                    
                    ui.horizontal(|ui| {
                        ui.colored_label(color, &self.message);
                        if self.secondary_button(ui, "×", [28.0, 28.0]).clicked() { // Adjusted size from [25.0, 25.0]
                            self.clear_message();
                        }
                    });
                    ui.add_space(SPACING);
                }

                match self.current_screen {
                    Screen::Welcome => self.show_welcome_screen(ui),
                    Screen::Init => self.show_init_screen(ui),
                    Screen::Login => self.show_login_screen(ui),
                    Screen::Main => self.show_main_screen(ui, ctx), // Pass ctx here
                    Screen::AddEntry => self.show_add_entry_screen(ui),
                    Screen::EditEntry(ref id) => {
                        let id = id.clone();
                        self.show_edit_entry_screen(ui, &id);
                    },                    Screen::Settings => self.show_settings_screen(ui),
                    Screen::HealthDashboard => self.show_health_dashboard(ui),
                    Screen::ImportExport => self.show_import_export_screen(ui),
                }
            });
    }
}

// Simple UI Screen implementations
impl PassmanApp {
    fn show_welcome_screen(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| { // Changed from vertical_centered
            ui.heading("Passman");
            ui.label("Password Manager");
        });
        ui.separator();
        ui.add_space(PADDING); // Add some space after the header

        ui.vertical_centered(|ui| { // Keep centering for the main content body
            ui.add_space(SPACING * 2.0); // Reduced initial top space

            ui.horizontal(|ui| {
                ui.label("Vault file:");
                ui.add(egui::TextEdit::singleline(&mut self.vault_file)
                    .desired_width(200.0));
            });
            ui.add_space(SPACING);

            let vault_exists = VaultManager::exists(Some(&self.vault_file));
            if vault_exists {
                if self.primary_button(ui, "Open Vault", [150.0, BUTTON_HEIGHT]).clicked() {
                    self.current_screen = Screen::Login;
                    self.clear_message();
                }
            } else {
                if self.success_button(ui, "Create Vault", [150.0, BUTTON_HEIGHT]).clicked() {
                    self.current_screen = Screen::Init;
                    self.clear_message();
                }
            }
            ui.add_space(SPACING);
            if self.secondary_button(ui, "Settings", [150.0, BUTTON_HEIGHT]).clicked() { // Adjusted width for consistency
                self.current_screen = Screen::Settings;
            }
        });
    }

    fn show_init_screen(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Create New Vault");
        });
        ui.separator();
        ui.add_space(PADDING);        ui.vertical_centered(|ui| {
            ui.add_space(SPACING * 2.0); // Adjusted top spacing
            
            ui.horizontal(|ui| {
                ui.label("Master Password:");
                ui.add(egui::TextEdit::singleline(&mut *self.init_password)
                    .password(true)
                    .desired_width(INPUT_WIDTH));
            });
            ui.add_space(SPACING);

            ui.horizontal(|ui| {
                ui.label("Confirm Password:");
                ui.add(egui::TextEdit::singleline(&mut *self.init_confirm)
                    .password(true)
                    .desired_width(INPUT_WIDTH));
            });
            ui.add_space(SPACING * 2.0);
            
            if self.success_button(ui, "Create", [150.0, BUTTON_HEIGHT]).clicked() { // Adjusted width
                match self.init_vault() {
                    Ok(()) => {
                        self.show_message("Vault created successfully!".to_string(), MessageType::Success);
                    }
                    Err(e) => {
                        self.show_message(e, MessageType::Error);
                    }                }
            }
            ui.add_space(SPACING); // Space between stacked buttons
            
            if self.secondary_button(ui, "Cancel", [150.0, BUTTON_HEIGHT]).clicked() { // Adjusted width
                self.current_screen = Screen::Welcome;
                *self.init_password = String::new();
                *self.init_confirm = String::new();
                self.clear_message();
            }
        });
    }

    fn show_login_screen(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| { // Align heading to the top-left
            ui.heading("Open Vault");
        });
        ui.separator(); // Add a separator line
        ui.add_space(PADDING); // Add padding after the separator
        
        ui.vertical_centered(|ui| { // Center the rest of the content
            ui.add_space(SPACING * 2.0); // Adjust top spacing as needed
            
            ui.horizontal(|ui| {
                ui.label("Master Password:");
                ui.add(egui::TextEdit::singleline(&mut *self.login_password)
                    .password(true)
                    .desired_width(INPUT_WIDTH));
            });
            ui.add_space(SPACING * 2.0);
            
            if self.primary_button(ui, "Open", [150.0, BUTTON_HEIGHT]).clicked() { // Adjusted width
                match self.login() {
                    Ok(()) => {
                        self.show_message("Vault opened successfully!".to_string(), MessageType::Success);
                    }
                    Err(e) => {
                        self.show_message(e, MessageType::Error);
                    }
                }
            }
            ui.add_space(SPACING); // Space between stacked buttons
            
            if self.secondary_button(ui, "Cancel", [150.0, BUTTON_HEIGHT]).clicked() { // Adjusted width
                self.current_screen = Screen::Welcome;
                *self.login_password = String::new();                self.clear_message();
            }
        });
    }

    fn show_main_screen(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) { // Add ctx as parameter        // Simple header
        ui.horizontal(|ui| {
            ui.heading("Password Vault");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.secondary_button(ui, "Lock", [60.0, 30.0]).clicked() {
                    self.current_screen = Screen::Welcome;
                    self.vault = None;
                    *self.master_password = String::new();
                    self.clear_message();
                }
                
                if self.secondary_button(ui, "Settings", [70.0, 30.0]).clicked() {
                    self.current_screen = Screen::Settings;
                }
                  if self.primary_button(ui, "Health", [60.0, 30.0]).clicked() {
                    self.current_screen = Screen::HealthDashboard;
                }
                
                if self.secondary_button(ui, "Import/Export", [100.0, 30.0]).clicked() {
                    self.current_screen = Screen::ImportExport;
                }
                
                if self.success_button(ui, "Add Entry", [80.0, 30.0]).clicked() {
                    self.current_screen = Screen::AddEntry;
                    self.clear_add_form();
                }
            });
        });
        ui.separator();
        ui.add_space(SPACING);

        // Simple search
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.add(egui::TextEdit::singleline(&mut self.search_query)
                .hint_text("Type to search...")
                .desired_width(300.0));
        });
        ui.add_space(SPACING);

        // Simple entries list
        egui::ScrollArea::vertical().show(ui, |ui| {
            let filtered_entries: Vec<(String, Entry)> = self.filter_entries()
                .into_iter()
                .map(|(id, entry)| (id.clone(), entry.clone()))
                .collect();
            
            if filtered_entries.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.label("No entries found");
                    if self.search_query.is_empty() {
                        ui.label("Click 'Add Entry' to get started");
                    }
                });
            } else {
                for (id, entry) in filtered_entries.iter() {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.strong(id);
                                ui.horizontal(|ui| {
                                    ui.label("User:");
                                    ui.label(&entry.username);
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Pass:");
                                    if *self.show_password.get(id).unwrap_or(&false) {
                                        ui.label(&entry.password);
                                    } else {
                                        ui.label("••••••••");
                                    }
                                });
                                if let Some(note) = &entry.note {
                                    if !note.is_empty() {
                                        ui.horizontal(|ui| {
                                            ui.label("Note:");
                                            ui.label(note);
                                        });
                                    }
                                }
                            });
                              ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if self.danger_button(ui, "Delete", [60.0, 30.0]).clicked() {
                                    match self.remove_entry(id) {
                                        Ok(()) => {
                                            self.show_message(format!("Entry '{}' deleted", id), MessageType::Success);
                                        }
                                        Err(e) => {
                                            self.show_message(e, MessageType::Error);
                                        }
                                    }                                }
                                
                                if self.primary_button(ui, "Copy", [60.0, 30.0]).clicked() {
                                    // Use secure clipboard with auto-clear
                                    match self.secure_clipboard.copy_password(&entry.password) {
                                        Ok(()) => {
                                            let timeout = self.clipboard_clear_secs;
                                            self.show_message(
                                                format!("Password copied! Clipboard will clear in {}s", timeout), 
                                                MessageType::Info
                                            );
                                        }
                                        Err(_) => {
                                            // Fallback to egui clipboard
                                            ctx.output_mut(|o| o.copied_text = entry.password.clone());
                                            self.show_message(
                                                format!("Password for '{}' copied (secure clipboard unavailable)", id), 
                                                MessageType::Info
                                            );
                                        }
                                    }
                                }
                                
                                if self.success_button(ui, "Edit", [60.0, 30.0]).clicked() {
                                    self.start_edit_entry(id);
                                }
                                
                                let eye_text = if *self.show_password.get(id).unwrap_or(&false) { "Hide" } else { "Show" };
                                if self.secondary_button(ui, eye_text, [60.0, 30.0]).clicked() {
                                    let current_shown_state = self.show_password.entry(id.clone()).or_insert(false);
                                    *current_shown_state = !*current_shown_state;
                                }
                            });
                        });                    });
                    ui.add_space(SPACING);
                }
            }
        });
    }

    fn show_add_entry_screen(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Add New Entry");
        });
        ui.separator();
        ui.add_space(PADDING);
        
        ui.vertical_centered(|ui| {
            ui.add_space(SPACING * 2.0); // Consistent top spacing after header

            // Use a grid layout for aligned labels and input fields
            egui::Grid::new("add_entry_grid")
                .num_columns(2)
                .spacing([SPACING * 2.0, SPACING]) // More horizontal spacing between label and field
                .striped(false)
                .show(ui, |ui| {
                    ui.label("Entry ID:");
                    ui.add(egui::TextEdit::singleline(&mut self.add_id)
                        .desired_width(INPUT_WIDTH)
                        .hint_text("e.g., gmail, work"));
                    ui.end_row();

                    ui.label("Username:");
                    ui.add(egui::TextEdit::singleline(&mut self.add_username)
                        .desired_width(INPUT_WIDTH)
                        .hint_text("Username or email"));
                    ui.end_row();

                    // Span the checkbox across two columns for better alignment or keep it in one for consistency
                    ui.label(""); // Empty label for spacing, or adjust colspan
                    ui.checkbox(&mut self.generate_password, "Generate secure password");
                    ui.end_row();

                    if self.generate_password {
                        ui.label("Length:");
                        ui.add(egui::Slider::new(&mut self.password_length, 8..=64)
                            .text("characters"));
                        ui.end_row();
                    } else {
                        ui.label("Password:");
                        if ui.add(egui::TextEdit::singleline(&mut self.add_password)
                            .desired_width(INPUT_WIDTH)
                            .password(true))
                            .changed() {
                            self.update_password_strength();
                        }
                        ui.end_row();
                        
                        if !self.password_strength.is_empty() {
                            ui.label("Strength:");
                            ui.label(&self.password_strength);
                            ui.end_row();

                            if !self.password_suggestions.is_empty() {
                                ui.label(""); // For grid alignment
                                ui.collapsing("Show Suggestions", |ui| {
                                    for suggestion in &self.password_suggestions {
                                        ui.label(format!("- {}", suggestion));
                                    }
                                });
                                ui.end_row();
                            }
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
            
            if self.success_button(ui, "Add Entry", [150.0, BUTTON_HEIGHT]).clicked() { // Adjusted width
                match self.add_entry() {
                    Ok(()) => {
                        self.show_message("Entry added successfully!".to_string(), MessageType::Success);
                    }
                    Err(e) => {
                        self.show_message(e, MessageType::Error);
                    }
                }
            }
            ui.add_space(SPACING); // Space between stacked buttons
            if self.secondary_button(ui, "Cancel", [150.0, BUTTON_HEIGHT]).clicked() { // Adjusted width
                self.current_screen = Screen::Main;
                self.clear_add_form();
            }
        });
    }    fn show_edit_entry_screen(&mut self, ui: &mut egui::Ui, id: &str) {
        ui.vertical(|ui| {
            ui.heading(format!("Edit Entry: {}", id));
        });
        ui.separator();
        ui.add_space(PADDING);

        ui.vertical_centered(|ui| {
            ui.add_space(SPACING * 2.0);
            
            // Use a grid layout for aligned labels and input fields
            egui::Grid::new("edit_entry_grid")
                .num_columns(2)
                .spacing([SPACING * 2.0, SPACING])
                .striped(false)
                .show(ui, |ui| {
                    ui.label("Username:");
                    ui.add(egui::TextEdit::singleline(&mut self.edit_username)
                        .desired_width(INPUT_WIDTH)
                        .hint_text("Username or email"));
                    ui.end_row();

                    ui.label(""); // Empty label for checkbox alignment
                    ui.checkbox(&mut self.edit_generate_password, "Generate new password");
                    ui.end_row();

                    if self.edit_generate_password {
                        ui.label("Length:");
                        ui.add(egui::Slider::new(&mut self.password_length, 8..=64)
                            .text("characters"));
                        ui.end_row();
                    } else {
                        ui.label("Password:");
                        ui.horizontal(|ui| {
                            if ui.add(egui::TextEdit::singleline(&mut self.edit_password)
                                .desired_width(INPUT_WIDTH - 80.0)
                                .password(!self.edit_show_password))
                                .changed() {
                                self.update_password_strength();
                            }
                            
                            let eye_text = if self.edit_show_password { "Hide" } else { "Show" };
                            if self.secondary_button(ui, eye_text, [60.0, 30.0]).clicked() {
                                self.edit_show_password = !self.edit_show_password;
                            }
                        });
                        ui.end_row();

                        if !self.password_strength.is_empty() {
                            ui.label("Strength:");
                            ui.label(&self.password_strength);
                            ui.end_row();

                            if !self.password_suggestions.is_empty() {
                                ui.label(""); // For grid alignment
                                ui.collapsing("Show Suggestions", |ui| {
                                    for suggestion in &self.password_suggestions {
                                        ui.label(format!("- {}", suggestion));
                                    }
                                });
                                ui.end_row();
                            }
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
                    match self.update_entry() {
                        Ok(()) => {
                            self.show_message("Entry updated successfully!".to_string(), MessageType::Success);
                        }
                        Err(e) => {
                            self.show_message(e, MessageType::Error);
                        }
                    }
                }
                
                if self.secondary_button(ui, "Cancel", [150.0, BUTTON_HEIGHT]).clicked() {
                    self.current_screen = Screen::Main;
                    self.clear_edit_form();
                }
            });
        });
    }

    fn show_settings_screen(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Settings");
        });
        ui.separator();
        ui.add_space(PADDING);

        ui.vertical_centered(|ui| {
            ui.add_space(SPACING * 2.0); // Adjusted top spacing
            
            ui.horizontal(|ui| {
                ui.label("Vault file:");
                ui.add(egui::TextEdit::singleline(&mut self.vault_file)
                    .desired_width(INPUT_WIDTH));
            });
            ui.add_space(SPACING);

            ui.label("Available vault files:");
            // Use a ScrollArea if the list of vaults can be long
            egui::ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
                if let Ok(vaults) = VaultManager::list_vaults() {
                    if vaults.is_empty() {
                        ui.label("No other vault files found in this directory.");
                    } else {
                        for vault_filename in vaults {
                            // Ensure we don't list the currently active vault_file as an option to "select" again if it's already the one.
                            // Or, visually indicate which one is active. For simplicity, just list them.
                            ui.horizontal(|ui| {
                                ui.label(&vault_filename);
                                // Make the button smaller and perhaps only show if it's not the current vault_file
                                if self.vault_file != vault_filename {
                                    if self.primary_button(ui, "Select", [80.0, 28.0]).clicked() {
                                        self.vault_file = vault_filename.clone();
                                        // Optionally, provide feedback or auto-navigate
                                        self.show_message(format!("Vault file set to '{}'. Please reopen.", self.vault_file), MessageType::Info);
                                        self.current_screen = Screen::Welcome; // Go to welcome to reopen or reinit
                                    }
                                } else {
                                    ui.label("(current)");
                                }
                            });
                        }
                    }
                } else {
                    ui.label("Could not read vault files.");
                }
            });
            ui.add_space(SPACING * 2.0);
            
            // Password Change Section
            ui.separator();
            ui.add_space(SPACING);
            
            if self.vault.is_some() {
                ui.collapsing("🔐 Change Master Password", |ui| {
                    ui.add_space(SPACING);
                    
                    ui.horizontal(|ui| {
                        ui.label("Current Password:");
                        ui.add(egui::TextEdit::singleline(&mut *self.change_current_password)
                            .password(true)
                            .desired_width(INPUT_WIDTH - 100.0));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("New Password:");
                        ui.add(egui::TextEdit::singleline(&mut *self.change_new_password)
                            .password(!self.show_password_change)
                            .desired_width(INPUT_WIDTH - 100.0));
                    });
                    
                    // Show password strength for new password
                    if !self.change_new_password.is_empty() {
                        let (strength, suggestions) = analyze_password_strength(&self.change_new_password);
                        ui.horizontal(|ui| {
                            ui.label("Strength:");
                            let color = match &strength {
                                PasswordStrength::VeryWeak | PasswordStrength::Weak => egui::Color32::RED,
                                PasswordStrength::Fair => egui::Color32::YELLOW,
                                PasswordStrength::Good => egui::Color32::LIGHT_GREEN,
                                PasswordStrength::Strong => egui::Color32::GREEN,
                            };
                            ui.colored_label(color, strength.to_string());
                        });
                        if !suggestions.is_empty() {
                            for suggestion in &suggestions {
                                ui.small(format!("• {}", suggestion));
                            }
                        }
                    }
                    
                    ui.horizontal(|ui| {
                        ui.label("Confirm Password:");
                        ui.add(egui::TextEdit::singleline(&mut *self.change_confirm_password)
                            .password(!self.show_password_change)
                            .desired_width(INPUT_WIDTH - 100.0));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.show_password_change, "Show passwords");
                    });
                    
                    ui.add_space(SPACING);
                    
                    if self.primary_button(ui, "Change Password", [150.0, BUTTON_HEIGHT]).clicked() {
                        // Validate inputs
                        if self.change_current_password.is_empty() {
                            self.show_message("Current password is required".into(), MessageType::Error);
                        } else if self.change_new_password.is_empty() {
                            self.show_message("New password is required".into(), MessageType::Error);
                        } else if self.change_new_password.len() < 8 {
                            self.show_message("New password must be at least 8 characters".into(), MessageType::Error);
                        } else if self.change_new_password.as_str() != self.change_confirm_password.as_str() {
                            self.show_message("New passwords do not match".into(), MessageType::Error);
                        } else if self.change_current_password.as_str() != self.master_password.as_str() {
                            self.show_message("Current password is incorrect".into(), MessageType::Error);
                        } else {
                            // Attempt to change password
                            match VaultManager::change_password(
                                &self.change_current_password,
                                &self.change_new_password,
                                Some(&self.vault_file)
                            ) {
                                Ok(()) => {
                                    *self.master_password = self.change_new_password.to_string();
                                    *self.change_current_password = String::new();
                                    *self.change_new_password = String::new();
                                    *self.change_confirm_password = String::new();
                                    self.show_message("Master password changed successfully!".into(), MessageType::Success);
                                }
                                Err(e) => {
                                    self.show_message(format!("Failed to change password: {}", e), MessageType::Error);
                                }
                            }
                        }
                    }
                });
            } else {
                ui.label("Unlock a vault to change the master password.");
            }
            
            ui.add_space(SPACING * 2.0);
            
            if self.secondary_button(ui, "Back", [150.0, BUTTON_HEIGHT]).clicked() { // Adjusted width
                self.current_screen = Screen::Welcome;
            }
        });
    }

    fn show_health_dashboard(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Password Health Dashboard");
        });
        ui.separator();
        ui.add_space(PADDING);        ui.vertical_centered(|ui| {
            ui.add_space(SPACING * 2.0); // Adjusted top spacing
            
            // Generate health summary if we have a vault
            if let Some(vault) = &self.vault {
                let reports = self.health_analyzer.analyze_vault(vault);
                let summary = self.health_analyzer.generate_summary(&reports);
                
                ui.label(format!("Overall Health: {:.1}%", summary.score));
                ui.add(egui::ProgressBar::new(summary.score as f32 / 100.0)
                    .text(format!("{:.1}%", summary.score)));
                
                ui.separator();
                
                // Show health distribution
                ui.horizontal(|ui| {
                    ui.label("Critical:");
                    ui.colored_label(egui::Color32::RED, format!("{}", summary.critical));
                    ui.label("Warning:");
                    ui.colored_label(egui::Color32::YELLOW, format!("{}", summary.warning));
                    ui.label("Good:");
                    ui.colored_label(egui::Color32::LIGHT_GREEN, format!("{}", summary.good));
                    ui.label("Excellent:");
                    ui.colored_label(egui::Color32::GREEN, format!("{}", summary.excellent));
                });
                
                ui.add_space(SPACING * 2.0);
                
                // Show individual entry health
                ui.label("Entry Details:");
                egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                    for report in &reports {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(&report.entry_id);
                                let health_text = match &report.health {
                                    crate::health::PasswordHealth::Excellent => "Excellent",
                                    crate::health::PasswordHealth::Good => "Good", 
                                    crate::health::PasswordHealth::Warning { .. } => "Warning",
                                    crate::health::PasswordHealth::Critical { .. } => "Critical",
                                };
                                let color = match &report.health {
                                    crate::health::PasswordHealth::Excellent => egui::Color32::GREEN,
                                    crate::health::PasswordHealth::Good => egui::Color32::LIGHT_GREEN,
                                    crate::health::PasswordHealth::Warning { .. } => egui::Color32::YELLOW,
                                    crate::health::PasswordHealth::Critical { .. } => egui::Color32::RED,
                                };
                                ui.colored_label(color, health_text);
                                ui.label(format!("Age: {} days", report.age_days));
                            });
                        });
                    }
                });
            } else {
                ui.label("No health data available. Please add entries to analyze.");
            }
            
            ui.add_space(SPACING * 2.0);
            
            if self.secondary_button(ui, "Back", [150.0, BUTTON_HEIGHT]).clicked() { // Adjusted width
                self.current_screen = Screen::Main;
            }
        });
    }

    fn show_import_export_screen(&mut self, ui: &mut egui::Ui) {
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
                            self.show_message("Please specify export file path".to_string(), MessageType::Error);
                        } else if let Some(vault) = &self.vault {
                            match self.export_format {
                                ExportFormat::Json => {
                                    match ImportExportManager::export_json(vault, &self.export_file_path) {
                                        Ok(()) => {
                                            self.show_message("Data exported successfully!".to_string(), MessageType::Success);
                                        }
                                        Err(e) => {
                                            self.show_message(format!("Export failed: {}", e), MessageType::Error);
                                        }
                                    }
                                }
                                ExportFormat::Csv => {
                                    match ImportExportManager::export_csv(vault, &self.export_file_path) {
                                        Ok(()) => {
                                            self.show_message("Data exported successfully!".to_string(), MessageType::Success);
                                        }
                                        Err(e) => {
                                            self.show_message(format!("Export failed: {}", e), MessageType::Error);
                                        }
                                    }
                                }
                            }
                        } else {
                            self.show_message("No vault loaded".to_string(), MessageType::Error);
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
                            self.show_message("Please specify import file path".to_string(), MessageType::Error);
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
                                            self.show_message("Data imported successfully!".to_string(), MessageType::Success);
                                        }
                                        Err(e) => {
                                            self.show_message(format!("Import succeeded but reload failed: {}", e), MessageType::Error);
                                        }
                                    }
                                }
                                Err(e) => {
                                    self.show_message(format!("Import failed: {}", e), MessageType::Error);
                                }
                            }
                        } else {
                            self.show_message("No vault loaded".to_string(), MessageType::Error);
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

    // Button helper methods
    fn primary_button(&self, ui: &mut egui::Ui, text: &str, size: [f32; 2]) -> egui::Response {
        ui.add_sized(size, egui::Button::new(text)
            .fill(egui::Color32::from_rgb(70, 130, 180)) // Steel blue
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 150, 200))))
    }

    fn secondary_button(&self, ui: &mut egui::Ui, text: &str, size: [f32; 2]) -> egui::Response {
        ui.add_sized(size, egui::Button::new(text)
            .fill(egui::Color32::from_rgb(108, 117, 125)) // Gray
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(138, 147, 155))))
    }

    fn success_button(&self, ui: &mut egui::Ui, text: &str, size: [f32; 2]) -> egui::Response {
        ui.add_sized(size, egui::Button::new(text)
            .fill(egui::Color32::from_rgb(40, 167, 69)) // Green
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(70, 197, 99))))
    }

    fn danger_button(&self, ui: &mut egui::Ui, text: &str, size: [f32; 2]) -> egui::Response {
        ui.add_sized(size, egui::Button::new(text)
            .fill(egui::Color32::from_rgb(220, 53, 69)) // Red
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(250, 83, 99))))
    }

    // Password strength analysis method
    fn update_password_strength(&mut self) {
        let password = if self.current_screen == Screen::AddEntry {
            &self.add_password
        } else {
            &self.edit_password
        };

        if password.is_empty() {
            self.password_strength.clear();
            self.password_suggestions.clear();
            return;
        }

        // Simple password strength analysis
        let mut score = 0;
        let mut suggestions = Vec::new();

        // Length check
        if password.len() >= 12 {
            score += 25;
        } else if password.len() >= 8 {
            score += 15;
        } else {
            suggestions.push("Use at least 8 characters".to_string());
        }

        // Character variety checks
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_numbers = password.chars().any(|c| c.is_numeric());
        let has_symbols = password.chars().any(|c| !c.is_alphanumeric());

        if has_lowercase { score += 15; } else { suggestions.push("Add lowercase letters".to_string()); }
        if has_uppercase { score += 15; } else { suggestions.push("Add uppercase letters".to_string()); }
        if has_numbers { score += 15; } else { suggestions.push("Add numbers".to_string()); }
        if has_symbols { score += 15; } else { suggestions.push("Add symbols (!@#$%^&*)".to_string()); }

        // Repetition check
        let unique_chars: std::collections::HashSet<char> = password.chars().collect();
        if unique_chars.len() as f32 / password.len() as f32 > 0.7 {
            score += 15;
        } else {
            suggestions.push("Avoid repeating characters".to_string());
        }

        // Set strength description
        self.password_strength = match score {
            0..=30 => format!("Weak ({}%)", score),
            31..=60 => format!("Fair ({}%)", score),
            61..=80 => format!("Good ({}%)", score),
            _ => format!("Strong ({}%)", score),
        };

        self.password_suggestions = suggestions;
    }
}
