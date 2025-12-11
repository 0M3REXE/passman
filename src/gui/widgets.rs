//! Widgets Module
//!
//! Reusable UI widgets and button helpers.

#![allow(dead_code)]

use eframe::egui;
use std::collections::HashMap;

/// Button style helpers for consistent UI
pub struct ButtonWidgets;

impl ButtonWidgets {
    /// Primary action button (steel blue)
    pub fn primary(ui: &mut egui::Ui, text: &str, size: [f32; 2]) -> egui::Response {
        ui.add_sized(size, egui::Button::new(text)
            .fill(egui::Color32::from_rgb(70, 130, 180))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 150, 200))))
    }

    /// Secondary action button (gray)
    pub fn secondary(ui: &mut egui::Ui, text: &str, size: [f32; 2]) -> egui::Response {
        ui.add_sized(size, egui::Button::new(text)
            .fill(egui::Color32::from_rgb(108, 117, 125))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(138, 147, 155))))
    }

    /// Success/confirm button (green)
    pub fn success(ui: &mut egui::Ui, text: &str, size: [f32; 2]) -> egui::Response {
        ui.add_sized(size, egui::Button::new(text)
            .fill(egui::Color32::from_rgb(40, 167, 69))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(70, 197, 99))))
    }

    /// Danger/delete button (red)
    pub fn danger(ui: &mut egui::Ui, text: &str, size: [f32; 2]) -> egui::Response {
        ui.add_sized(size, egui::Button::new(text)
            .fill(egui::Color32::from_rgb(220, 53, 69))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(250, 83, 99))))
    }
}

/// Show inline error for a form field
pub fn show_field_error(ui: &mut egui::Ui, form_errors: &HashMap<String, String>, field: &str) {
    if let Some(error) = form_errors.get(field) {
        ui.colored_label(egui::Color32::from_rgb(220, 53, 69), format!("⚠ {}", error));
    }
}

/// Visual password strength indicator with progress bar and color
pub fn show_password_strength_indicator(ui: &mut egui::Ui, password: &str) {
    if password.is_empty() {
        return;
    }
    
    // Calculate strength score (0-100)
    let mut score = 0;
    let mut suggestions = Vec::new();
    
    // Length scoring
    if password.len() >= 16 {
        score += 30;
    } else if password.len() >= 12 {
        score += 25;
    } else if password.len() >= 8 {
        score += 15;
    } else {
        suggestions.push("Use at least 8 characters");
    }
    
    // Character variety
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_numbers = password.chars().any(|c| c.is_numeric());
    let has_symbols = password.chars().any(|c| !c.is_alphanumeric());
    
    if has_lowercase { score += 15; } else { suggestions.push("Add lowercase letters"); }
    if has_uppercase { score += 15; } else { suggestions.push("Add uppercase letters"); }
    if has_numbers { score += 15; } else { suggestions.push("Add numbers"); }
    if has_symbols { score += 15; } else { suggestions.push("Add symbols (!@#$%^&*)"); }
    
    // Uniqueness bonus
    let unique_chars: std::collections::HashSet<char> = password.chars().collect();
    if unique_chars.len() as f32 / password.len() as f32 > 0.7 {
        score += 10;
    }
    
    let score = score.min(100);
    
    // Determine color and label based on score
    let (color, label) = match score {
        0..=25 => (egui::Color32::from_rgb(220, 53, 69), "Very Weak"),
        26..=50 => (egui::Color32::from_rgb(255, 140, 0), "Weak"),
        51..=70 => (egui::Color32::from_rgb(255, 193, 7), "Fair"),
        71..=85 => (egui::Color32::from_rgb(40, 167, 69), "Good"),
        _ => (egui::Color32::from_rgb(0, 200, 83), "Strong"),
    };
    
    // Draw the strength indicator
    ui.horizontal(|ui| {
        ui.label("Strength:");
        
        // Progress bar
        let bar_width = 150.0;
        let bar_height = 8.0;
        let (rect, _response) = ui.allocate_exact_size(
            egui::vec2(bar_width, bar_height),
            egui::Sense::hover()
        );
        
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            
            // Background
            painter.rect_filled(
                rect,
                egui::Rounding::same(4.0),
                egui::Color32::from_rgb(60, 60, 60)
            );
            
            // Filled portion
            let filled_width = rect.width() * (score as f32 / 100.0);
            let filled_rect = egui::Rect::from_min_size(
                rect.min,
                egui::vec2(filled_width, bar_height)
            );
            painter.rect_filled(
                filled_rect,
                egui::Rounding::same(4.0),
                color
            );
        }
        
        ui.add_space(8.0);
        ui.colored_label(color, format!("{} ({}%)", label, score));
    });
    
    // Show suggestions in a collapsible section
    if !suggestions.is_empty() {
        ui.collapsing("💡 Suggestions", |ui| {
            for suggestion in suggestions {
                ui.horizontal(|ui| {
                    ui.label("•");
                    ui.label(suggestion);
                });
            }
        });
    }
}

/// Analyze password and return strength description and suggestions
pub fn analyze_password_strength(password: &str) -> (String, Vec<String>) {
    if password.is_empty() {
        return (String::new(), Vec::new());
    }

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
    let strength = match score {
        0..=30 => format!("Weak ({}%)", score),
        31..=60 => format!("Fair ({}%)", score),
        61..=80 => format!("Good ({}%)", score),
        _ => format!("Strong ({}%)", score),
    };

    (strength, suggestions)
}
