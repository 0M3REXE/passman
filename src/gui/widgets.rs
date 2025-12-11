//! Widgets Module
//!
//! Reusable UI widgets and button helpers.

#![allow(dead_code)]

use eframe::egui;
use std::collections::HashMap;

// ============================================================================
// BUTTON WIDGETS
// ============================================================================

/// Button style helpers for consistent UI
pub struct ButtonWidgets;

impl ButtonWidgets {
    /// Primary action button (steel blue) with rounded style
    pub fn primary(ui: &mut egui::Ui, text: &str, size: [f32; 2]) -> egui::Response {
        let button = egui::Button::new(
            egui::RichText::new(text).color(egui::Color32::WHITE)
        )
        .fill(egui::Color32::from_rgb(59, 130, 246))
        .stroke(egui::Stroke::NONE)
        .rounding(egui::Rounding::same(6.0));
        ui.add_sized(size, button)
    }

    /// Secondary action button (subtle gray)
    pub fn secondary(ui: &mut egui::Ui, text: &str, size: [f32; 2]) -> egui::Response {
        let button = egui::Button::new(text)
            .fill(egui::Color32::from_rgb(75, 85, 99))
            .stroke(egui::Stroke::NONE)
            .rounding(egui::Rounding::same(6.0));
        ui.add_sized(size, button)
    }

    /// Success/confirm button (green)
    pub fn success(ui: &mut egui::Ui, text: &str, size: [f32; 2]) -> egui::Response {
        let button = egui::Button::new(
            egui::RichText::new(text).color(egui::Color32::WHITE)
        )
        .fill(egui::Color32::from_rgb(34, 197, 94))
        .stroke(egui::Stroke::NONE)
        .rounding(egui::Rounding::same(6.0));
        ui.add_sized(size, button)
    }

    /// Danger/delete button (red)
    pub fn danger(ui: &mut egui::Ui, text: &str, size: [f32; 2]) -> egui::Response {
        let button = egui::Button::new(
            egui::RichText::new(text).color(egui::Color32::WHITE)
        )
        .fill(egui::Color32::from_rgb(239, 68, 68))
        .stroke(egui::Stroke::NONE)
        .rounding(egui::Rounding::same(6.0));
        ui.add_sized(size, button)
    }
    
    /// Icon button (compact, for toolbars)
    pub fn icon(ui: &mut egui::Ui, icon: &str, size: f32, tooltip: &str) -> egui::Response {
        let button = egui::Button::new(icon)
            .fill(egui::Color32::TRANSPARENT)
            .stroke(egui::Stroke::NONE)
            .rounding(egui::Rounding::same(4.0));
        ui.add_sized([size, size], button).on_hover_text(tooltip)
    }
    
    /// Outlined button variant
    pub fn outlined(ui: &mut egui::Ui, text: &str, size: [f32; 2], color: egui::Color32) -> egui::Response {
        let button = egui::Button::new(
            egui::RichText::new(text).color(color)
        )
        .fill(egui::Color32::TRANSPARENT)
        .stroke(egui::Stroke::new(1.5, color))
        .rounding(egui::Rounding::same(6.0));
        ui.add_sized(size, button)
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

// ============================================================================
// CARD HELPERS
// ============================================================================

/// Calculate password strength score (0-100)
pub fn calculate_password_score(password: &str) -> u32 {
    if password.is_empty() {
        return 0;
    }
    
    let mut score = 0u32;
    
    // Length scoring
    if password.len() >= 16 {
        score += 30;
    } else if password.len() >= 12 {
        score += 25;
    } else if password.len() >= 8 {
        score += 15;
    }
    
    // Character variety
    if password.chars().any(|c| c.is_lowercase()) { score += 15; }
    if password.chars().any(|c| c.is_uppercase()) { score += 15; }
    if password.chars().any(|c| c.is_numeric()) { score += 15; }
    if password.chars().any(|c| !c.is_alphanumeric()) { score += 15; }
    
    // Uniqueness bonus
    let unique_chars: std::collections::HashSet<char> = password.chars().collect();
    if unique_chars.len() as f32 / password.len() as f32 > 0.7 {
        score += 10;
    }
    
    score.min(100)
}

/// Get strength color based on score
pub fn strength_color(score: u32) -> egui::Color32 {
    match score {
        0..=25 => egui::Color32::from_rgb(239, 68, 68),    // Red
        26..=50 => egui::Color32::from_rgb(251, 146, 60),  // Orange
        51..=70 => egui::Color32::from_rgb(250, 204, 21),  // Yellow
        71..=85 => egui::Color32::from_rgb(34, 197, 94),   // Green
        _ => egui::Color32::from_rgb(16, 185, 129),        // Emerald
    }
}

/// Paint a strength indicator bar (small dots or line)
pub fn paint_strength_dots(ui: &mut egui::Ui, score: u32) {
    let num_dots = 4;
    let filled = match score {
        0..=25 => 1,
        26..=50 => 2,
        51..=75 => 3,
        _ => 4,
    };
    let color = strength_color(score);
    let inactive_color = egui::Color32::from_rgb(60, 60, 65);
    
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 3.0;
        for i in 0..num_dots {
            let dot_color = if i < filled { color } else { inactive_color };
            let (rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
            ui.painter().circle_filled(rect.center(), 3.5, dot_color);
        }
    });
}

/// Section header with optional action
pub fn section_header(ui: &mut egui::Ui, title: &str) {
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(title).size(13.0).color(egui::Color32::from_rgb(156, 163, 175)));
    });
    ui.add_space(2.0);
}

/// Styled search bar
pub fn styled_search_bar(
    ui: &mut egui::Ui, 
    search_query: &mut String, 
    width: f32,
    bg_color: egui::Color32,
    border_color: egui::Color32,
) -> egui::Response {
    egui::Frame::none()
        .fill(bg_color)
        .rounding(egui::Rounding::same(8.0))
        .stroke(egui::Stroke::new(1.0, border_color))
        .inner_margin(egui::Margin::symmetric(12.0, 8.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🔍").size(14.0).color(egui::Color32::from_rgb(156, 163, 175)));
                ui.add_space(4.0);
                ui.add(
                    egui::TextEdit::singleline(search_query)
                        .hint_text("Search entries...")
                        .frame(false)
                        .desired_width(width - 50.0)
                )
            }).inner
        }).inner
}

/// Empty state widget with icon and message
pub fn empty_state(ui: &mut egui::Ui, icon: &str, title: &str, subtitle: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(60.0);
        ui.label(egui::RichText::new(icon).size(48.0));
        ui.add_space(16.0);
        ui.label(egui::RichText::new(title).size(18.0).strong());
        ui.add_space(8.0);
        ui.label(egui::RichText::new(subtitle).size(14.0).color(egui::Color32::from_rgb(156, 163, 175)));
    });
}
