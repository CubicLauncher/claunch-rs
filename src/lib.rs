// Copyright (C) 2025 Santiagolxx, Notstaff and CubicLauncher contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! CLaunch - Universal Minecraft Launcher Library
//!
//! Supports Vanilla, Forge, NeoForge and Fabric with version inheritance system.

pub mod auth;
pub mod launcher;
pub mod models;
pub mod resolvers;
pub mod utils;

pub use auth::{MinecraftUser, AccountType};
pub use launcher::Launcher;
pub use models::{LaunchOptions, Library, VersionInfo};

/// Result type for CLaunch operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for CLaunch
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Main class not found")]
    MainClassNotFound,

    #[error("Classpath is empty")]
    EmptyClasspath,

    #[error("Invalid Java path: {0}")]
    InvalidJavaPath(String),

    #[error("Failed to load version file: {0}")]
    VersionLoadFailed(String),

    #[error("Failed to load base version: {0}")]
    BaseVersionLoadFailed(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}