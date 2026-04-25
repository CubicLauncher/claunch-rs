// Copyright (C) 2025 Santiagolxx, Notstaff and CubicLauncher contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod microsoft;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftUser {
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub user_type: AccountType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountType {
    Cracked,
    Microsoft,
}

impl MinecraftUser {
    /// Create a new cracked (offline) user
    pub fn cracked(username: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            uuid: uuid::Uuid::new_v4().to_string(),
            access_token: "0".to_string(),
            refresh_token: None,
            user_type: AccountType::Cracked,
        }
    }

    /// Create a new premium user (from Microsoft auth)
    pub fn premium(username: String, uuid: String, access_token: String, refresh_token: Option<String>) -> Self {
        Self {
            username,
            uuid,
            access_token,
            refresh_token,
            user_type: AccountType::Microsoft,
        }
    }
}