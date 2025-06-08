use eframe::egui;
use crate::model::{Entry, Vault};
use crate::vault::VaultManager;
use crate::utils::*;
use std::collections::HashMap;
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
    
    // Search and filtering
    search_query: String,
    
    // Messages
    message: String,
    message_type: MessageType,
    
    // Password strength
    password_strength: String,
    password_suggestions: Vec<String>,
}

#[derive(Default, PartialEq)]
#[allow(dead_code)]
enum Screen {
    #[default]
    Welcome,
    Init,
    Login,
    Main,
    AddEntry,
    EditEntry(String),
    Settings,
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
        cc.egui_ctx.set_style(style);        Self {
            vault_file: "vault.dat".to_string(),
            password_length: 16,
            master_password: Zeroizing::new(String::new()),
            init_password: Zeroizing::new(String::new()),
            init_confirm: Zeroizing::new(String::new()),
            login_password: Zeroizing::new(String::new()),
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
        let vault = VaultManager::load(&self.login_password, Some(&self.vault_file))
            .map_err(|e| e.to_string())?;

        *self.master_password = self.login_password.to_string();
        self.vault = Some(vault);
        self.load_entries();
        self.current_screen = Screen::Main;
        *self.login_password = String::new();

        Ok(())
    }

    fn add_entry(&mut self) -> Result<(), String> {
        if let Some(vault) = &mut self.vault {
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

    fn update_password_strength(&mut self) {
        if !self.add_password.is_empty() {
            let (strength, suggestions) = analyze_password_strength(&self.add_password);
            self.password_strength = strength.to_string();
            self.password_suggestions = suggestions;
        } else {
            self.password_strength.clear();
            self.password_suggestions.clear();
        }
    }    // Helper functions for colored buttons
    fn primary_button(&self, ui: &mut egui::Ui, text: &str, size: [f32; 2]) -> egui::Response {
        let button = egui::Button::new(text)
            .fill(egui::Color32::from_rgb(70, 130, 180));
        ui.add_sized(size, button)
    }

    fn success_button(&self, ui: &mut egui::Ui, text: &str, size: [f32; 2]) -> egui::Response {
        let button = egui::Button::new(text)
            .fill(egui::Color32::from_rgb(40, 167, 69));
        ui.add_sized(size, button)
    }

    fn danger_button(&self, ui: &mut egui::Ui, text: &str, size: [f32; 2]) -> egui::Response {
        let button = egui::Button::new(text)
            .fill(egui::Color32::from_rgb(220, 53, 69));
        ui.add_sized(size, button)
    }

    fn secondary_button(&self, ui: &mut egui::Ui, text: &str, size: [f32; 2]) -> egui::Response {
        let button = egui::Button::new(text)
            .fill(egui::Color32::from_rgb(108, 117, 125));
        ui.add_sized(size, button)
    }
}

impl eframe::App for PassmanApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Visuals are set in `PassmanApp::new`
        
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
                    },
                    Screen::Settings => self.show_settings_screen(ui),
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

    fn show_main_screen(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) { // Add ctx as parameter
        // Simple header
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
                                    ctx.output_mut(|o| o.copied_text = entry.password.clone());
                                    self.show_message(format!("Password for \'{}\' copied to clipboard!", id), MessageType::Info);
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
    }

    fn show_edit_entry_screen(&mut self, ui: &mut egui::Ui, id: &str) {
        ui.vertical(|ui| {
            ui.heading(format!("Edit Entry: {}", id));
        });
        ui.separator();
        ui.add_space(PADDING);

        ui.vertical_centered(|ui| {
            ui.add_space(SPACING * 2.0); // Adjusted top spacing
            
            ui.label("Edit functionality coming soon...");
            ui.add_space(SPACING * 2.0);
            if self.secondary_button(ui, "Back", [150.0, BUTTON_HEIGHT]).clicked() { // Adjusted width
                self.current_screen = Screen::Main;
            }
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
            
            if self.secondary_button(ui, "Back", [150.0, BUTTON_HEIGHT]).clicked() { // Adjusted width
                self.current_screen = Screen::Welcome;
            }
        });
    }
}
