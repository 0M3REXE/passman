//! GUI Types Module
//!
//! Shared types and enums used throughout the GUI.

use std::time::Instant;

// UI Constants
pub const BUTTON_HEIGHT: f32 = 36.0;
pub const SPACING: f32 = 10.0;
pub const PADDING: f32 = 20.0;

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

/// Toast notification types
#[derive(Clone, Copy, PartialEq)]
pub enum ToastType {
    Success,
    Error,
    Info,
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
