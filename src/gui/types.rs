//! GUI Types Module
//!
//! Shared types and enums used throughout the GUI.

#![allow(dead_code)]

use std::time::Instant;

// UI Constants
pub const BUTTON_HEIGHT: f32 = 36.0;
pub const INPUT_WIDTH: f32 = 300.0;
pub const SPACING: f32 = 10.0;
pub const PADDING: f32 = 20.0;
pub const MIN_WINDOW_WIDTH: f32 = 500.0;
pub const MIN_WINDOW_HEIGHT: f32 = 400.0;

/// Get responsive input width based on available space
pub fn responsive_input_width(available_width: f32) -> f32 {
    let base = INPUT_WIDTH;
    if available_width < 600.0 {
        (available_width * 0.7).max(200.0).min(base)
    } else if available_width > 1000.0 {
        base * 1.3
    } else {
        base
    }
}

/// Get responsive button size
#[allow(dead_code)]
pub fn responsive_button_size(available_width: f32) -> [f32; 2] {
    if available_width < 600.0 {
        [120.0, BUTTON_HEIGHT]
    } else {
        [150.0, BUTTON_HEIGHT]
    }
}

/// Application screens
#[derive(Default, PartialEq, Clone)]
pub enum Screen {
    #[default]
    Welcome,
    Init,
    Login,
    Main,
    AddEntry,
    EditEntry(String),
    Settings,
    HealthDashboard,
    ImportExport,
}

/// Message types for UI feedback
#[derive(Default, PartialEq, Clone, Copy)]
pub enum MessageType {
    #[default]
    None,
    Success,
    Error,
    Info,
}

/// Toast notification types
#[derive(Clone, Copy, PartialEq)]
pub enum ToastType {
    Success,
    Error,
    Info,
    Warning,
}

/// Toast notification with auto-dismiss
#[derive(Clone)]
pub struct Toast {
    pub message: String,
    pub toast_type: ToastType,
    pub created_at: Instant,
    pub duration_secs: f32,
}

impl Toast {
    pub fn new(message: impl Into<String>, toast_type: ToastType) -> Self {
        Self {
            message: message.into(),
            toast_type,
            created_at: Instant::now(),
            duration_secs: 3.0,
        }
    }

    #[allow(dead_code)]
    pub fn with_duration(mut self, secs: f32) -> Self {
        self.duration_secs = secs;
        self
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed().as_secs_f32() >= self.duration_secs
    }

    pub fn progress(&self) -> f32 {
        1.0 - (self.created_at.elapsed().as_secs_f32() / self.duration_secs).min(1.0)
    }
}

/// Application theme
#[derive(Default, PartialEq, Clone, Copy)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

impl Theme {
    pub fn name(&self) -> &'static str {
        match self {
            Theme::Dark => "Dark",
            Theme::Light => "Light",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        }
    }
}

/// Export file formats
#[derive(Default, PartialEq, Clone, Copy)]
pub enum ExportFormat {
    #[default]
    Json,
    Csv,
}

/// Import file formats
#[derive(Default, PartialEq, Clone, Copy)]
pub enum ImportFormat {
    #[default]
    Json,
    Csv,
    Chrome,
}
