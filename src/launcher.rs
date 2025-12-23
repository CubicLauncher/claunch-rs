// Copyright (C) 2025 Santiagolxx, Notstaff and CubicLauncher contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::models::{LaunchOptions, VersionInfo};
use crate::resolvers::{CommandBuilder, DependencyResolver};
use log::info;
use std::collections::HashMap;
use std::path::Path;
use std::process::{Child, Command, Stdio};

/// Universal Minecraft Launcher
/// Supports Vanilla, Forge, NeoForge and Fabric with version inheritance
pub struct Launcher;

impl Launcher {
    /// Simple launch without additional options
    pub fn launch(
        version_json_path: impl AsRef<Path>,
        game_dir: impl AsRef<Path>,
        instance_dir: impl AsRef<Path>,
        username: &str,
        java_path: impl AsRef<Path>,
        min_ram: &str,
        max_ram: &str,
        width: u32,
        height: u32,
        cracked: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Self::launch_with_options(
            version_json_path,
            game_dir,
            instance_dir,
            username,
            java_path,
            min_ram,
            max_ram,
            width,
            height,
            cracked,
            LaunchOptions::default(),
            HashMap::new(),
        )
    }

    /// Launch with custom options
    pub fn launch_with_options(
        version_json_path: impl AsRef<Path>,
        game_dir: impl AsRef<Path>,
        instance_dir: impl AsRef<Path>,
        username: &str,
        java_path: impl AsRef<Path>,
        min_ram: &str,
        max_ram: &str,
        width: u32,
        height: u32,
        cracked: bool,
        options: LaunchOptions,
        custom_env: HashMap<String, String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("=== CubicLauncher CLaunch ===");

        let info = VersionInfo::new(version_json_path, game_dir)?;
        info!("Version: {}", info.version_id);
        info!("Demo mode: {}", options.demo_mode);

        if let (Some(mode), Some(value)) = (&options.quick_play_mode, &options.quick_play_value) {
            info!("Quick Play: {:?} -> {}", mode, value);
        } else {
            info!("Quick Play: disabled");
        }

        if !custom_env.is_empty() {
            info!("Custom environment variables: {:?}", custom_env);
        }

        Self::prepare_directories(&info)?;

        let main_class = info
            .get_property("mainClass")
            .ok_or("Main class not found")?;

        let classpath = Self::build_classpath(&info)?;
        if classpath.is_empty() {
            return Err("Classpath is empty".into());
        }

        let vars = Self::build_variables(&info, username, instance_dir.as_ref());
        let command = Self::build_command(
            &info, vars, options, &java_path, min_ram, max_ram, cracked, &classpath, main_class,
            width, height,
        );

        Self::execute_game(command, &info.game_dir, &java_path, custom_env)?;
        Ok(())
    }

    /// Launch and return the Process for advanced control
    pub fn launch_with_process(
        version_json_path: impl AsRef<Path>,
        game_dir: impl AsRef<Path>,
        instance_dir: impl AsRef<Path>,
        username: &str,
        java_path: impl AsRef<Path>,
        min_ram: &str,
        max_ram: &str,
        width: u32,
        height: u32,
        cracked: bool,
        options: LaunchOptions,
        custom_env: HashMap<String, String>,
    ) -> Result<Child, Box<dyn std::error::Error>> {
        info!("=== CubicLauncher CLaunch ===");

        let info = VersionInfo::new(version_json_path, game_dir)?;
        info!("Version: {}", info.version_id);

        if !custom_env.is_empty() {
            info!("Custom environment variables: {:?}", custom_env);
        }

        Self::prepare_directories(&info)?;

        let main_class = info
            .get_property("mainClass")
            .ok_or("Main class not found")?;

        let classpath = Self::build_classpath(&info)?;
        if classpath.is_empty() {
            return Err("Classpath is empty".into());
        }

        let vars = Self::build_variables(&info, username, instance_dir.as_ref());
        let command = Self::build_command(
            &info, vars, options, &java_path, min_ram, max_ram, cracked, &classpath, main_class,
            width, height,
        );

        Self::start_process(command, &info.game_dir, &java_path, custom_env)
    }

    /// Launch with DPRIME environment variable (for compatibility)
    pub fn launch_with_dprime(
        version_json_path: impl AsRef<Path>,
        game_dir: impl AsRef<Path>,
        instance_dir: impl AsRef<Path>,
        username: &str,
        java_path: impl AsRef<Path>,
        min_ram: &str,
        max_ram: &str,
        width: u32,
        height: u32,
        cracked: bool,
        options: LaunchOptions,
    ) -> Result<Child, Box<dyn std::error::Error>> {
        let mut env = HashMap::new();
        env.insert("DPRIME".to_string(), "1".to_string());

        Self::launch_with_process(
            version_json_path,
            game_dir,
            instance_dir,
            username,
            java_path,
            min_ram,
            max_ram,
            width,
            height,
            cracked,
            options,
            env,
        )
    }

    // ==================== AUXILIARY METHODS ====================

    fn prepare_directories(info: &VersionInfo) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(info.get_assets_virtual_dir())?;
        std::fs::create_dir_all(info.game_dir.join("config"))?;
        Ok(())
    }

    fn build_variables(
        info: &VersionInfo,
        username: &str,
        instance_dir: &Path,
    ) -> HashMap<String, String> {
        let mut vars = HashMap::new();

        // Generate UUID
        let uuid = uuid::Uuid::new_v4().to_string();

        vars.insert("auth_player_name".to_string(), username.to_string());
        vars.insert("version_name".to_string(), info.version_id.clone());
        vars.insert(
            "game_directory".to_string(),
            instance_dir.display().to_string(),
        );
        vars.insert(
            "assets_root".to_string(),
            info.assets_dir.display().to_string(),
        );
        vars.insert(
            "assets_index_name".to_string(),
            info.get_assets_index_name(),
        );
        vars.insert("auth_uuid".to_string(), uuid);
        vars.insert("auth_access_token".to_string(), "0".to_string());
        vars.insert("user_type".to_string(), "mojang".to_string());
        vars.insert("user_properties".to_string(), "{}".to_string());
        vars.insert(
            "version_type".to_string(),
            info.get_property("type").unwrap_or("release").to_string(),
        );

        #[cfg(windows)]
        vars.insert("classpath_separator".to_string(), ";".to_string());
        #[cfg(not(windows))]
        vars.insert("classpath_separator".to_string(), ":".to_string());

        vars.insert(
            "library_directory".to_string(),
            info.lib_dir.display().to_string(),
        );
        vars.insert(
            "natives_directory".to_string(),
            info.natives_dir.display().to_string(),
        );
        vars
    }

    fn build_classpath(info: &VersionInfo) -> Result<String, Box<dyn std::error::Error>> {
        info!("Building classpath...");

        let mut resolver = DependencyResolver::new(info.lib_dir.clone(), info.natives_dir.clone());

        // Process parent first (is_child = false)
        if info.has_inheritance() {
            if let Some(base_data) = &info.base_version_data {
                resolver.process_version(base_data, false);
            }
        }

        // Process child after (is_child = true) - has priority in conflicts
        resolver.process_version(&info.version_data, true);

        let classpath = resolver.build_classpath(info);
        info!(
            "Classpath built with {} libraries",
            resolver.library_count()
        );

        Ok(classpath)
    }

    fn build_command(
        info: &VersionInfo,
        vars: HashMap<String, String>,
        options: LaunchOptions,
        java_path: impl AsRef<Path>,
        min_ram: &str,
        max_ram: &str,
        cracked: bool,
        classpath: &str,
        main_class: &str,
        width: u32,
        height: u32,
    ) -> Vec<String> {
        let mut builder = CommandBuilder::new(info, vars, options);
        builder
            .add_java(java_path)
            .add_jvm_args(min_ram, max_ram, cracked)
            .add_classpath(classpath)
            .add_main_class(main_class)
            .add_game_args(width, height);

        builder.build()
    }

    fn start_process(
        command: Vec<String>,
        game_dir: &Path,
        java_path: impl AsRef<Path>,
        custom_env: HashMap<String, String>,
    ) -> Result<Child, Box<dyn std::error::Error>> {
        info!("\n=== Final Command ===");
        info!("{}", command.join(" "));
        info!("\n=== Starting Game ===");

        let java_home = java_path.as_ref().parent().ok_or("Invalid Java path")?;

        let mut cmd = Command::new(&command[0]);
        cmd.args(&command[1..])
            .current_dir(game_dir)
            .env("JAVA_HOME", java_home);

        for (key, value) in custom_env {
            info!("Setting custom environment variable: {}={}", key, value);
            cmd.env(key, value);
        }

        let child = cmd.spawn()?;
        Ok(child)
    }

    fn execute_game(
        command: Vec<String>,
        game_dir: &Path,
        java_path: impl AsRef<Path>,
        custom_env: HashMap<String, String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("\n=== Final Command ===");
        info!("{}", command.join(" "));
        info!("\n=== Starting Game ===");

        let java_home = java_path.as_ref().parent().ok_or("Invalid Java path")?;

        let mut cmd = Command::new(&command[0]);
        cmd.args(&command[1..])
            .current_dir(game_dir)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .env("JAVA_HOME", java_home);

        for (key, value) in custom_env {
            info!("Setting custom environment variable: {}={}", key, value);
            cmd.env(key, value);
        }

        let mut child = cmd.spawn()?;
        let exit_code = child.wait()?;

        if exit_code.success() {
            info!("Game finished successfully");
        } else {
            log::error!("ERROR: Exit code: {:?}", exit_code.code());
        }

        Ok(())
    }
}
