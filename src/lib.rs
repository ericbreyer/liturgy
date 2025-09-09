//! # Liturgy
//!
//! A Rust library for generating liturgical calendars for both the 1962 Roman Calendar (Extraordinary Form)
//! and the Ordinary Form (Post-Vatican II) calendars.
//!
//! ## Features
//!
//! - Load liturgical calendars from TOML configuration files
//! - Support for both 1962 and Ordinary Form calendars
//! - Extensible calendar system with feast merging
//! - Comprehensive date rule system (fixed dates, Easter-based, etc.)
//! - Advanced feast ranking and conflict resolution
//! - CSV export functionality and utilities
//! - Web API for REST-based calendar access
//!
//! ## Modules
//!
//! - [`calender`] - Core liturgical calendar functionality
//! - [`csv_utils`] - CSV data processing and analysis utilities
//! - [`web`] - REST API backend server
//!
//! ## Quick Start
//!
//! ```rust
//! use liturgy::GenericCalendarHandle;
//!
//! // Load a calendar
//! let calendar = GenericCalendarHandle::load_from_file("calendar_data/of.toml")?;
//!
//! // Generate a year calendar
//! let year_2025 = calendar.create_year_calendar(2025);
//!
//! // Export to CSV
//! year_2025.export_csv("calendar_2025.csv")?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Web API
//!
//! ```rust
//! use liturgy::web::{run_web_app, WebConfig};
//!
//! // Set up API server
//! let config = WebConfig::default();
//! // Server provides REST endpoints at /api/*
//! // run_web_app(config).await?;
//!
//! // API endpoints available:
//! // GET /api/calendars - List calendars
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod calender;
pub mod csv_utils;
mod date_calc;
pub mod web;

// Re-export main public API types
pub use calender::{LiturgicalUnit, GenericCalendarHandle, YearCalendarHandle};

// Re-export convenience modules
pub use csv_utils::CsvUtils;

