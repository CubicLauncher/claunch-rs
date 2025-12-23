// Copyright (C) 2025 Santiagolxx, Notstaff and CubicLauncher contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

/// Launch options for Minecraft
#[derive(Debug, Clone, Default)]
pub struct LaunchOptions {
    pub demo_mode: bool,
    pub quick_play_mode: Option<QuickPlayMode>,
    pub quick_play_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuickPlayMode {
    Singleplayer,
    Multiplayer,
    Realms,
}

impl LaunchOptions {
    /// Create default launch options
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable demo mode
    pub fn with_demo(mut self, demo: bool) -> Self {
        self.demo_mode = demo;
        self
    }

    /// Set Quick Play for singleplayer
    pub fn with_quick_play_singleplayer(mut self, world_name: impl Into<String>) -> Self {
        self.quick_play_mode = Some(QuickPlayMode::Singleplayer);
        self.quick_play_value = Some(world_name.into());
        self
    }

    /// Set Quick Play for multiplayer
    pub fn with_quick_play_multiplayer(mut self, server_address: impl Into<String>) -> Self {
        self.quick_play_mode = Some(QuickPlayMode::Multiplayer);
        self.quick_play_value = Some(server_address.into());
        self
    }

    /// Set Quick Play for realms
    pub fn with_quick_play_realms(mut self, realm_id: impl Into<String>) -> Self {
        self.quick_play_mode = Some(QuickPlayMode::Realms);
        self.quick_play_value = Some(realm_id.into());
        self
    }
}
