use eframe::egui;
use crate::model::{Entry, Vault};
use crate::vault::VaultManager;
use crate::utils::*;
use std::collections::HashMap;

// UI Constants for consistent sizing
const SMALL_BUTTON_SIZE: [f32; 2] = [32.0, 32.0];
const MEDIUM_BUTTON_SIZE: [f32; 2] = [120.0, 40.0];
const LARGE_BUTTON_SIZE: [f32; 2] = [160.0, 40.0];
const STANDARD_INPUT_WIDTH: f32 = 280.0;
const WIDE_INPUT_WIDTH: f32 = 350.0;
const CARD_WIDTH: f32 = 450.0;
const STANDARD_SPACING: f32 = 16.0;
const LARGE_SPACING: f32 = 32.0;
const SECTION_SPACING: f32 = 24.0;

#[derive(Default)]
pub struct PassmanApp {
    // App state
    current_screen: Screen,
    vault: Option<Vault>,
    vault_file: String,
    master_password: String,
    
    // UI state
    show_password: HashMap<String, bool>,
    entries: Vec<(String, Entry)>,
    
    // Form fields
    init_password: String,
    init_confirm: String,
    login_password: String,
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
enum MessageType {
    #[default]
    None,
    Success,
    Error,
    Info,
}

impl PassmanApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Configure modern egui style
        let mut style = (*cc.egui_ctx.style()).clone();
        
        // Modern dark theme with improved colors
        style.visuals.dark_mode = true;
        style.visuals.override_text_color = Some(egui::Color32::from_rgb(230, 230, 235));
        style.visuals.window_fill = egui::Color32::from_rgb(28, 30, 35);
        style.visuals.panel_fill = egui::Color32::from_rgb(35, 38, 45);
        
        // Modern rounded corners
        style.visuals.window_rounding = egui::Rounding::same(16.0);
        style.visuals.widgets.noninteractive.rounding = egui::Rounding::same(10.0);
        style.visuals.widgets.inactive.rounding = egui::Rounding::same(10.0);
        style.visuals.widgets.hovered.rounding = egui::Rounding::same(10.0);
        style.visuals.widgets.active.rounding = egui::Rounding::same(10.0);
        style.visuals.menu_rounding = egui::Rounding::same(12.0);
        
        // Enhanced button styling with gradients effect
        style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(55, 60, 75);
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(75, 85, 105);
        style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(85, 95, 115);
        
        // Input field styling with better contrast
        style.visuals.extreme_bg_color = egui::Color32::from_rgb(20, 22, 28);
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(45, 50, 60);
        style.visuals.selection.bg_fill = egui::Color32::from_rgb(70, 100, 140);
        
        // Enhanced spacing for better layout
        style.spacing.item_spacing = egui::vec2(16.0, 10.0);
        style.spacing.button_padding = egui::vec2(20.0, 10.0);
        style.spacing.indent = 24.0;
        
        // Typography improvements
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(28.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(16.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(16.0, egui::FontFamily::Proportional),
        );
        
        cc.egui_ctx.set_style(style);

        Self {
            vault_file: "vault.dat".to_string(),
            password_length: 16,
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
    }

    fn init_vault(&mut self) -> Result<(), String> {
        if self.init_password != self.init_confirm {
            return Err("Passwords do not match!".into());
        }

        if self.init_password.len() < 8 {
            return Err("Password must be at least 8 characters long!".into());
        }

        VaultManager::init(&self.init_password, Some(&self.vault_file))
            .map_err(|e| e.to_string())?;

        self.master_password = self.init_password.clone();
        self.vault = Some(Vault::new());
        self.load_entries();
        self.current_screen = Screen::Main;
        self.init_password.clear();
        self.init_confirm.clear();

        Ok(())
    }

    fn login(&mut self) -> Result<(), String> {
        let vault = VaultManager::load(&self.login_password, Some(&self.vault_file))
            .map_err(|e| e.to_string())?;

        self.master_password = self.login_password.clone();
        self.vault = Some(vault);
        self.load_entries();
        self.current_screen = Screen::Main;
        self.login_password.clear();

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
    }

    fn clear_add_form(&mut self) {
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
    }
}

impl eframe::App for PassmanApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Central panel with improved styling
        egui::CentralPanel::default()
            .frame(egui::Frame::none().inner_margin(20.0))
            .show(ctx, |ui| {
                // Show message if any with improved styling
                if !self.message.is_empty() {
                    let (color, bg_color) = match self.message_type {
                        MessageType::Success => (egui::Color32::from_rgb(100, 200, 100), egui::Color32::from_rgb(30, 60, 30)),
                        MessageType::Error => (egui::Color32::from_rgb(220, 100, 100), egui::Color32::from_rgb(60, 30, 30)),
                        MessageType::Info => (egui::Color32::from_rgb(100, 150, 220), egui::Color32::from_rgb(30, 45, 60)),
                        MessageType::None => (egui::Color32::GRAY, egui::Color32::from_rgb(40, 40, 40)),
                    };

                    egui::Frame::none()
                        .fill(bg_color)
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(12.0)
                        .stroke(egui::Stroke::new(1.0, color))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.colored_label(color, &self.message);
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.add_sized([24.0, 24.0], egui::Button::new("✕")).clicked() {
                                        self.clear_message();
                                    }
                                });
                            });
                        });
                    ui.add_space(16.0);
                }

                match self.current_screen {
                    Screen::Welcome => self.show_welcome_screen(ui),
                    Screen::Init => self.show_init_screen(ui),
                    Screen::Login => self.show_login_screen(ui),
                    Screen::Main => self.show_main_screen(ui),
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

// UI Screen implementations
impl PassmanApp {    fn show_welcome_screen(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(LARGE_SPACING * 2.5);
            
            // App title with better styling
            ui.add_space(STANDARD_SPACING);
            ui.heading("🔐 Passman");
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Secure Password Manager")
                .size(18.0)
                .color(egui::Color32::from_rgb(180, 180, 185)));
            ui.add_space(LARGE_SPACING + 8.0);

            // Vault file selection in a card-like container
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(42, 46, 54))
                .rounding(egui::Rounding::same(12.0))
                .inner_margin(SECTION_SPACING)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.set_min_width(CARD_WIDTH);
                        ui.label(egui::RichText::new("Vault Configuration")
                            .size(16.0)
                            .strong());
                        ui.add_space(STANDARD_SPACING);
                        
                        ui.horizontal(|ui| {
                            ui.label("Vault file:");
                            ui.add_space(8.0);
                            ui.add(egui::TextEdit::singleline(&mut self.vault_file)
                                .desired_width(STANDARD_INPUT_WIDTH)
                                .font(egui::TextStyle::Body));
                        });
                    });
                });

            ui.add_space(LARGE_SPACING);

            // Action buttons with improved styling
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - (LARGE_BUTTON_SIZE[0] * 2.0 + STANDARD_SPACING)) / 2.0);
                
                if VaultManager::exists(Some(&self.vault_file)) {
                    if ui.add_sized(LARGE_BUTTON_SIZE, 
                        egui::Button::new(egui::RichText::new("🔓 Open Vault").size(16.0)))
                        .clicked() {
                        self.current_screen = Screen::Login;
                        self.clear_message();
                    }
                } else {
                    if ui.add_sized(LARGE_BUTTON_SIZE, 
                        egui::Button::new(egui::RichText::new("🆕 Create Vault").size(16.0)))
                        .clicked() {
                        self.current_screen = Screen::Init;
                        self.clear_message();
                    }
                }

                ui.add_space(STANDARD_SPACING);
                
                if ui.add_sized(MEDIUM_BUTTON_SIZE, 
                    egui::Button::new(egui::RichText::new("⚙️ Settings").size(16.0)))
                    .clicked() {
                    self.current_screen = Screen::Settings;
                }
            });

            ui.add_space(LARGE_SPACING * 2.0);
        });
    }    fn show_init_screen(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(LARGE_SPACING * 2.5);
            
            ui.heading("Create New Vault");
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Set up a secure master password for your vault")
                .size(16.0)
                .color(egui::Color32::from_rgb(180, 180, 185)));
            ui.add_space(LARGE_SPACING + 8.0);

            // Password form in a card
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(42, 46, 54))
                .rounding(egui::Rounding::same(12.0))
                .inner_margin(SECTION_SPACING)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.set_min_width(CARD_WIDTH);
                        
                        ui.label(egui::RichText::new("Master Password Setup")
                            .size(16.0)
                            .strong());
                        ui.add_space(STANDARD_SPACING);

                        ui.horizontal(|ui| {
                            ui.label("Master Password:");
                            ui.add_space(8.0);
                            ui.add(egui::TextEdit::singleline(&mut self.init_password)
                                .password(true)
                                .desired_width(STANDARD_INPUT_WIDTH));
                        });
                        ui.add_space(STANDARD_SPACING);

                        ui.horizontal(|ui| {
                            ui.label("Confirm Password:");
                            ui.add_space(8.0);
                            ui.add(egui::TextEdit::singleline(&mut self.init_confirm)
                                .password(true)
                                .desired_width(STANDARD_INPUT_WIDTH));
                        });
                        
                        ui.add_space(STANDARD_SPACING);
                        ui.label(egui::RichText::new("💡 Use a strong password with at least 8 characters")
                            .size(14.0)
                            .color(egui::Color32::from_rgb(160, 160, 165)));
                    });
                });

            ui.add_space(LARGE_SPACING);

            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - (LARGE_BUTTON_SIZE[0] + MEDIUM_BUTTON_SIZE[0] + STANDARD_SPACING)) / 2.0);
                
                if ui.add_sized(LARGE_BUTTON_SIZE, 
                    egui::Button::new(egui::RichText::new("✅ Create Vault").size(16.0)))
                    .clicked() {
                    match self.init_vault() {
                        Ok(()) => {
                            self.show_message("Vault created successfully!".to_string(), MessageType::Success);
                        }
                        Err(e) => {
                            self.show_message(e, MessageType::Error);
                        }
                    }
                }

                ui.add_space(STANDARD_SPACING);
                
                if ui.add_sized(MEDIUM_BUTTON_SIZE, 
                    egui::Button::new(egui::RichText::new("❌ Cancel").size(16.0)))
                    .clicked() {
                    self.current_screen = Screen::Welcome;
                    self.init_password.clear();
                    self.init_confirm.clear();
                    self.clear_message();
                }
            });
        });
    }    fn show_login_screen(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(LARGE_SPACING * 2.5);
            
            ui.heading("Open Vault");
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Enter your master password to unlock your vault")
                .size(16.0)
                .color(egui::Color32::from_rgb(180, 180, 185)));
            ui.add_space(LARGE_SPACING + 8.0);

            // Login form in a card
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(42, 46, 54))
                .rounding(egui::Rounding::same(12.0))
                .inner_margin(SECTION_SPACING)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.set_min_width(CARD_WIDTH);
                        
                        ui.label(egui::RichText::new("🔒 Vault Authentication")
                            .size(16.0)
                            .strong());
                        ui.add_space(STANDARD_SPACING);

                        ui.horizontal(|ui| {
                            ui.label("Master Password:");
                            ui.add_space(8.0);
                            ui.add(egui::TextEdit::singleline(&mut self.login_password)
                                .password(true)
                                .desired_width(STANDARD_INPUT_WIDTH));
                        });
                        
                        ui.add_space(STANDARD_SPACING);
                        ui.label(egui::RichText::new(format!("Vault: {}", self.vault_file))
                            .size(14.0)
                            .color(egui::Color32::from_rgb(160, 160, 165)));
                    });
                });            ui.add_space(LARGE_SPACING);

            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - (MEDIUM_BUTTON_SIZE[0] * 2.0 + STANDARD_SPACING)) / 2.0);
                
                if ui.add_sized(MEDIUM_BUTTON_SIZE, 
                    egui::Button::new(egui::RichText::new("🔓 Open").size(16.0)))
                    .clicked() {
                    match self.login() {
                        Ok(()) => {
                            self.show_message("Vault opened successfully!".to_string(), MessageType::Success);
                        }
                        Err(e) => {
                            self.show_message(e, MessageType::Error);
                        }
                    }
                }

                ui.add_space(STANDARD_SPACING);
                
                if ui.add_sized(MEDIUM_BUTTON_SIZE, 
                    egui::Button::new(egui::RichText::new("❌ Cancel").size(16.0)))
                    .clicked() {
                    self.current_screen = Screen::Welcome;
                    self.login_password.clear();
                    self.clear_message();
                }
            });
        });
    }    fn show_main_screen(&mut self, ui: &mut egui::Ui) {
        // Modern top bar with better styling
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(42, 46, 54))
            .inner_margin(STANDARD_SPACING)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("🔐 Password Vault");
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(format!("({} entries)", self.entries.len()))
                        .size(14.0)
                        .color(egui::Color32::from_rgb(160, 160, 165)));
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add_sized(MEDIUM_BUTTON_SIZE, 
                            egui::Button::new(egui::RichText::new("🔒 Lock").size(14.0)))
                            .clicked() {
                            self.current_screen = Screen::Welcome;
                            self.vault = None;
                            self.master_password.clear();
                            self.clear_message();
                        }
                        
                        ui.add_space(8.0);
                        
                        if ui.add_sized(SMALL_BUTTON_SIZE, 
                            egui::Button::new("⚙️"))
                            .clicked() {
                            self.current_screen = Screen::Settings;
                        }
                        
                        ui.add_space(8.0);
                        
                        if ui.add_sized(LARGE_BUTTON_SIZE, 
                            egui::Button::new(egui::RichText::new("➕ Add Entry").size(14.0)))
                            .clicked() {
                            self.current_screen = Screen::AddEntry;
                            self.clear_add_form();
                        }
                    });
                });
            });

        ui.add_space(STANDARD_SPACING);

        // Search bar with modern styling
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(42, 46, 54))
            .rounding(egui::Rounding::same(10.0))
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("🔍");
                    ui.add(egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text("Search entries...")
                        .desired_width(ui.available_width() - 30.0));
                });
            });

        ui.add_space(STANDARD_SPACING);

        // Entries list with improved cards
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let filtered_entries: Vec<(String, Entry)> = self.filter_entries()
                    .into_iter()
                    .map(|(id, entry)| (id.clone(), entry.clone()))
                    .collect();
                
                if filtered_entries.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(LARGE_SPACING * 2.5);
                        ui.label(egui::RichText::new("No entries found")
                            .size(18.0)
                            .color(egui::Color32::from_rgb(140, 140, 145)));
                        ui.add_space(8.0);
                        if !self.search_query.is_empty() {
                            ui.label(egui::RichText::new("Try adjusting your search")
                                .size(14.0)
                                .color(egui::Color32::from_rgb(120, 120, 125)));
                        } else {
                            ui.label(egui::RichText::new("Click 'Add Entry' to get started")
                                .size(14.0)
                                .color(egui::Color32::from_rgb(120, 120, 125)));
                        }
                    });
                } else {
                    for (i, (id, entry)) in filtered_entries.iter().enumerate() {
                        // Modern entry card
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(42, 46, 54))
                            .rounding(egui::Rounding::same(10.0))
                            .inner_margin(STANDARD_SPACING)
                            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(55, 60, 70)))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    // Entry info section
                                    ui.vertical(|ui| {
                                        ui.label(egui::RichText::new(id)
                                            .size(16.0)
                                            .strong()
                                            .color(egui::Color32::from_rgb(220, 220, 225)));
                                        
                                        ui.add_space(4.0);
                                        
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new("👤")
                                                .size(14.0));
                                            ui.label(egui::RichText::new(&entry.username)
                                                .size(14.0)
                                                .color(egui::Color32::from_rgb(180, 180, 185)));
                                        });
                                        
                                        ui.add_space(4.0);
                                        
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new("🔑")
                                                .size(14.0));
                                            
                                            if *self.show_password.get(id).unwrap_or(&false) {
                                                ui.label(egui::RichText::new(&entry.password)
                                                    .size(14.0)
                                                    .family(egui::FontFamily::Monospace)
                                                    .color(egui::Color32::from_rgb(180, 180, 185)));
                                            } else {
                                                ui.label(egui::RichText::new("••••••••••••")
                                                    .size(14.0)
                                                    .color(egui::Color32::from_rgb(120, 120, 125)));
                                            }
                                        });
                                        
                                        if let Some(note) = &entry.note {
                                            if !note.is_empty() {
                                                ui.add_space(4.0);
                                                ui.horizontal(|ui| {
                                                    ui.label(egui::RichText::new("📝")
                                                        .size(14.0));
                                                    ui.label(egui::RichText::new(note)
                                                        .size(13.0)
                                                        .color(egui::Color32::from_rgb(160, 160, 165)));
                                                });
                                            }
                                        }
                                    });
                                    
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                                        // Action buttons
                                        ui.vertical(|ui| {
                                            if ui.add_sized(SMALL_BUTTON_SIZE, 
                                                egui::Button::new("🗑️")
                                                    .fill(egui::Color32::from_rgb(120, 60, 60)))
                                                .on_hover_text("Delete entry")
                                                .clicked() {
                                                match self.remove_entry(id) {
                                                    Ok(()) => {
                                                        self.show_message(format!("Entry '{}' deleted", id), MessageType::Success);
                                                    }
                                                    Err(e) => {
                                                        self.show_message(e, MessageType::Error);
                                                    }
                                                }
                                            }
                                            
                                            ui.add_space(4.0);
                                            
                                            if ui.add_sized(SMALL_BUTTON_SIZE, 
                                                egui::Button::new("📋"))                                                .on_hover_text("Copy password")
                                                .clicked() {
                                                match copy_to_clipboard(&entry.password) {
                                                    Ok(()) => {
                                                        self.show_message("Password copied to clipboard!".to_string(), MessageType::Success);
                                                    }
                                                    Err(e) => {
                                                        self.show_message(format!("Failed to copy: {}", e), MessageType::Error);
                                                    }
                                                }
                                            }
                                            
                                            ui.add_space(4.0);
                                            
                                            let eye_icon = if *self.show_password.get(id).unwrap_or(&false) { "🙈" } else { "👁️" };
                                            if ui.add_sized(SMALL_BUTTON_SIZE, 
                                                egui::Button::new(eye_icon))
                                                .on_hover_text("Toggle password visibility")
                                                .clicked() {
                                                let current = *self.show_password.get(id).unwrap_or(&false);
                                                self.show_password.insert(id.clone(), !current);
                                            }
                                        });
                                    });
                                });
                            });
                        
                        if i < filtered_entries.len() - 1 {
                            ui.add_space(STANDARD_SPACING - 4.0);
                        }
                    }
                    ui.add_space(STANDARD_SPACING);
                }
            });
    }    fn show_add_entry_screen(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(LARGE_SPACING + 8.0);
            
            ui.heading("Add New Entry");
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Create a new password entry")
                .size(16.0)
                .color(egui::Color32::from_rgb(180, 180, 185)));
            ui.add_space(LARGE_SPACING + 8.0);

            // Form in a card
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(42, 46, 54))
                .rounding(egui::Rounding::same(12.0))
                .inner_margin(SECTION_SPACING)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.set_min_width(CARD_WIDTH + 50.0);
                        
                        ui.label(egui::RichText::new("Entry Details")
                            .size(16.0)
                            .strong());
                        ui.add_space(STANDARD_SPACING);

                        ui.horizontal(|ui| {
                            ui.label("Entry ID:");
                            ui.add_space(8.0);
                            ui.add(egui::TextEdit::singleline(&mut self.add_id)
                                .desired_width(WIDE_INPUT_WIDTH)
                                .hint_text("e.g., gmail, work, etc."));
                        });
                        ui.add_space(STANDARD_SPACING);

                        ui.horizontal(|ui| {
                            ui.label("Username:");
                            ui.add_space(8.0);
                            ui.add(egui::TextEdit::singleline(&mut self.add_username)
                                .desired_width(WIDE_INPUT_WIDTH)
                                .hint_text("Username or email"));
                        });
                        ui.add_space(STANDARD_SPACING);

                        ui.checkbox(&mut self.generate_password, "Generate secure password");
                        ui.add_space(8.0);

                        if self.generate_password {
                            ui.horizontal(|ui| {
                                ui.label("Password length:");
                                ui.add_space(8.0);
                                ui.add(egui::Slider::new(&mut self.password_length, 8..=64)
                                    .text("characters"));
                            });
                        } else {
                            ui.horizontal(|ui| {
                                ui.label("Password:");
                                ui.add_space(8.0);
                                if ui.add(egui::TextEdit::singleline(&mut self.add_password)
                                    .desired_width(WIDE_INPUT_WIDTH)
                                    .password(true))
                                    .changed() {
                                    self.update_password_strength();
                                }
                            });

                            if !self.password_strength.is_empty() {
                                ui.add_space(8.0);
                                ui.label(egui::RichText::new(format!("Strength: {}", self.password_strength))
                                    .size(14.0)
                                    .color(egui::Color32::from_rgb(160, 160, 165)));
                                
                                if !self.password_suggestions.is_empty() {
                                    ui.add_space(4.0);
                                    ui.label(egui::RichText::new("Suggestions:")
                                        .size(14.0)
                                        .color(egui::Color32::from_rgb(160, 160, 165)));
                                    for suggestion in &self.password_suggestions {
                                        ui.label(egui::RichText::new(format!("• {}", suggestion))
                                            .size(13.0)
                                            .color(egui::Color32::from_rgb(140, 140, 145)));
                                    }
                                }
                            }
                        }
                        
                        ui.add_space(STANDARD_SPACING);

                        ui.horizontal(|ui| {
                            ui.label("Note (optional):");
                            ui.add_space(8.0);
                            ui.add(egui::TextEdit::multiline(&mut self.add_note)
                                .desired_width(WIDE_INPUT_WIDTH)
                                .desired_rows(3)
                                .hint_text("Additional notes about this entry"));
                        });
                    });
                });

            ui.add_space(LARGE_SPACING);

            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - (LARGE_BUTTON_SIZE[0] + MEDIUM_BUTTON_SIZE[0] + STANDARD_SPACING)) / 2.0);
                
                if ui.add_sized(LARGE_BUTTON_SIZE, 
                    egui::Button::new(egui::RichText::new("✅ Add Entry").size(16.0)))
                    .clicked() {
                    match self.add_entry() {
                        Ok(()) => {
                            self.show_message("Entry added successfully!".to_string(), MessageType::Success);
                        }
                        Err(e) => {
                            self.show_message(e, MessageType::Error);
                        }
                    }
                }

                ui.add_space(STANDARD_SPACING);
                
                if ui.add_sized(MEDIUM_BUTTON_SIZE, 
                    egui::Button::new(egui::RichText::new("❌ Cancel").size(16.0)))
                    .clicked() {
                    self.current_screen = Screen::Main;
                    self.clear_add_form();
                }
            });
        });
    }    fn show_edit_entry_screen(&mut self, ui: &mut egui::Ui, id: &str) {
        ui.vertical_centered(|ui| {
            ui.add_space(LARGE_SPACING * 2.5);
            
            ui.heading(format!("Edit Entry: {}", id));
            ui.add_space(STANDARD_SPACING);

            ui.label(egui::RichText::new("Edit functionality coming soon...")
                .size(16.0)
                .color(egui::Color32::from_rgb(180, 180, 185)));
            ui.add_space(LARGE_SPACING);

            if ui.add_sized(MEDIUM_BUTTON_SIZE, 
                egui::Button::new(egui::RichText::new("❌ Back").size(16.0)))
                .clicked() {
                self.current_screen = Screen::Main;
            }
        });
    }

    fn show_settings_screen(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(LARGE_SPACING * 2.5);
            
            ui.heading("Settings");
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Configure your password manager")
                .size(16.0)
                .color(egui::Color32::from_rgb(180, 180, 185)));
            ui.add_space(LARGE_SPACING + 8.0);

            // Settings form in a card
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(42, 46, 54))
                .rounding(egui::Rounding::same(12.0))
                .inner_margin(SECTION_SPACING)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.set_min_width(CARD_WIDTH + 50.0);
                        
                        ui.label(egui::RichText::new("Vault Configuration")
                            .size(16.0)
                            .strong());
                        ui.add_space(STANDARD_SPACING);

                        ui.horizontal(|ui| {
                            ui.label("Default vault file:");
                            ui.add_space(8.0);
                            ui.add(egui::TextEdit::singleline(&mut self.vault_file)
                                .desired_width(WIDE_INPUT_WIDTH));
                        });

                        ui.add_space(STANDARD_SPACING);

                        ui.label(egui::RichText::new("Available vault files:")
                            .size(14.0)
                            .strong());
                        ui.add_space(8.0);
                        
                        if let Ok(vaults) = VaultManager::list_vaults() {
                            for vault in vaults {
                                ui.horizontal(|ui| {
                                    ui.label(&vault);
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.add_sized(MEDIUM_BUTTON_SIZE, egui::Button::new("Select")).clicked() {
                                            self.vault_file = vault;
                                        }
                                    });
                                });
                            }
                        }
                    });
                });

            ui.add_space(LARGE_SPACING);

            if ui.add_sized(MEDIUM_BUTTON_SIZE, 
                egui::Button::new(egui::RichText::new("❌ Back").size(16.0)))
                .clicked() {
                self.current_screen = Screen::Welcome;
            }
        });
    }
}
