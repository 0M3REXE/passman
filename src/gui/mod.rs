//! GUI Module
//!
//! Graphical user interface for the Passman password manager.
//! 
//! # Module Structure
//! 
//! - `types` - Shared types and enums (Screen, Theme, Toast, etc.)
//! - `theme` - Theme handling and visual styling
//! - `widgets` - Reusable UI widgets (buttons, password strength)
//! - `toasts` - Toast notification system
//! - `overlays` - Modal dialogs, loading overlay, onboarding
//! - `app` - Main PassmanApp struct and state management
//! - `screens` - Individual screen implementations
//!   - `welcome` - Welcome, Init, Login screens
//!   - `main` - Main vault screen
//!   - `entry` - Add/Edit entry screens
//!   - `settings` - Settings screen
//!   - `health` - Health dashboard
//!   - `import_export` - Import/Export screen

pub mod types;
pub mod theme;
pub mod widgets;
pub mod toasts;
pub mod overlays;
pub mod app;
pub mod screens;

// Re-export main types for convenience
pub use app::PassmanApp;
