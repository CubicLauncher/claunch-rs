// Copyright (C) 2025 Santiagolxx, Notstaff and CubicLauncher contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::models::{LaunchOptions, QuickPlayMode, VersionInfo};
use crate::utils::json_utils;
use log::info;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::Path;

const DEMO_ARGS: &[&str] = &["--demo"];
const QUICKPLAY_ARGS: &[&str] = &[
    "--quickPlaySingleplayer",
    "--quickPlayMultiplayer",
    "--quickPlayRealms",
    "--quickPlayPath",
];

/// Command builder for Minecraft launch
pub struct CommandBuilder<'a> {
    command: Vec<String>,
    info: &'a VersionInfo,
    vars: HashMap<String, String>,
    options: LaunchOptions,
}

impl<'a> CommandBuilder<'a> {
    pub fn new(
        info: &'a VersionInfo,
        vars: HashMap<String, String>,
        options: LaunchOptions,
    ) -> Self {
        Self {
            command: Vec::new(),
            info,
            vars,
            options,
        }
    }

    pub fn add_java(&mut self, java_path: impl AsRef<Path>) -> &mut Self {
        let java_bin = self.get_java_bin(java_path);
        self.command.push(java_bin);
        self
    }

    pub fn add_jvm_args(&mut self, min_ram: &str, max_ram: &str, cracked: bool) -> &mut Self {
        self.command.push(format!(
            "-Djava.library.path={}",
            self.info.natives_dir.display()
        ));
        self.command
            .push("-Dminecraft.launcher.brand=CubicLauncher".to_string());
        self.command
            .push("-Dminecraft.launcher.version=1.0".to_string());

        if cracked {
            info!("Offline mode enabled");
            self.command.push("-Dminecraft.api.env=custom".to_string());
            self.command
                .push("-Dminecraft.api.auth.host=https://invalid.invalid".to_string());
            self.command
                .push("-Dminecraft.api.account.host=https://invalid.invalid".to_string());
            self.command
                .push("-Dminecraft.api.session.host=https://invalid.invalid".to_string());
            self.command
                .push("-Dminecraft.api.services.host=https://invalid.invalid".to_string());
        }

        self.command.push(format!("-Xms{}", min_ram));
        self.command.push(format!("-Xmx{}", max_ram));

        self.process_jvm_arguments();
        self
    }

    pub fn add_classpath(&mut self, classpath: &str) -> &mut Self {
        self.command.push("-cp".to_string());
        self.command.push(classpath.to_string());
        self
    }

    pub fn add_main_class(&mut self, main_class: &str) -> &mut Self {
        self.command.push(main_class.to_string());
        self
    }

    pub fn add_game_args(&mut self, width: u32, height: u32) -> &mut Self {
        self.process_game_arguments();
        self.add_default_game_args(width, height);
        self.add_optional_args();
        self.cleanup_unresolved_vars();
        self
    }

    fn process_jvm_arguments(&mut self) {
        if let Some(child_args) = json_utils::get_args_from_version(&self.info.version_data, "jvm")
        {
            self.process_jvm_array(child_args);
        }

        if self.info.has_inheritance()
            && let Some(base_data) = &self.info.base_version_data
                && let Some(parent_args) = json_utils::get_args_from_version(base_data, "jvm") {
                    self.process_jvm_array(parent_args);
                }
    }

    fn process_jvm_array(&mut self, args: &[Value]) {
        for element in args {
            if let Some(arg_str) = element.as_str() {
                self.add_jvm_arg(arg_str);
            } else if let Some(arg_obj) = element.as_object() {
                self.process_conditional_arg(arg_obj);
            }
        }
    }

    fn add_jvm_arg(&mut self, arg: &str) {
        if arg == "-cp" || arg == "-classpath" || arg.contains("${classpath}") {
            return;
        }

        let replaced = self.replace_vars(arg);

        if replaced.starts_with("--") || replaced.starts_with("-D") || replaced.starts_with("-X") {
            self.command.push(replaced);
        } else if !self.command.contains(&replaced) {
            self.command.push(replaced);
        }
    }

    fn process_conditional_arg(&mut self, arg_obj: &serde_json::Map<String, Value>) {
        if let Some(rules) = arg_obj.get("rules").and_then(|v| v.as_array())
            && !json_utils::evaluate_rules(rules) {
                return;
            }

        if let Some(value) = arg_obj.get("value") {
            if let Some(val_str) = value.as_str() {
                self.add_jvm_arg(val_str);
            } else if let Some(values) = value.as_array() {
                self.process_value_array(values);
            }
        }
    }

    fn process_value_array(&mut self, values: &[Value]) {
        if values.is_empty() {
            return;
        }

        if let Some(first) = values[0].as_str() {
            if first.starts_with("--") && values.len() > 2 {
                let flag = self.replace_vars(first);
                for val in &values[1..] {
                    if let Some(val_str) = val.as_str() {
                        let replaced = self.replace_vars(val_str);
                        if !self.has_flag_value(&flag, &replaced) {
                            self.command.push(flag.clone());
                            self.command.push(replaced);
                        }
                    }
                }
            } else {
                let mut to_add = Vec::new();
                let mut skip = false;

                for val in values {
                    if let Some(arg) = val.as_str() {
                        if arg.contains("${classpath}") || arg == "-cp" {
                            skip = true;
                            break;
                        }
                        to_add.push(self.replace_vars(arg));
                    }
                }

                if !skip && !to_add.is_empty() && !self.command.contains(&to_add[0]) {
                    self.command.extend(to_add);
                }
            }
        }
    }

    fn has_flag_value(&self, flag: &str, value: &str) -> bool {
        for i in 0..self.command.len().saturating_sub(1) {
            if self.command[i] == flag && self.command[i + 1] == value {
                return true;
            }
        }
        false
    }

    fn process_game_arguments(&mut self) {
        let child_args = json_utils::get_args_from_version(&self.info.version_data, "game");

        if let Some(args) = child_args {
            self.add_game_args_array(args);
        } else if self.info.has_inheritance() {
            if let Some(base_data) = &self.info.base_version_data
                && let Some(parent_args) = json_utils::get_args_from_version(base_data, "game") {
                    self.add_game_args_array(parent_args);
                }
        } else {
            self.add_legacy_args();
        }
    }

    fn add_game_args_array(&mut self, args: &[Value]) {
        let demo_set: HashSet<&str> = DEMO_ARGS.iter().copied().collect();
        let quickplay_set: HashSet<&str> = QUICKPLAY_ARGS.iter().copied().collect();

        let mut i = 0;
        while i < args.len() {
            let element = &args[i];

            if let Some(arg) = element.as_str() {
                if demo_set.contains(arg) && !self.options.demo_mode {
                    i += 1;
                    continue;
                }

                if quickplay_set.contains(arg) && self.options.quick_play_mode.is_none() {
                    if i + 1 < args.len() && args[i + 1].is_string() {
                        i += 2;
                    } else {
                        i += 1;
                    }
                    continue;
                }

                self.command.push(self.replace_vars(arg));
            } else if let Some(arg_obj) = element.as_object() {
                if let Some(rules) = arg_obj.get("rules").and_then(|v| v.as_array())
                    && !self.evaluate_rules_with_options(rules) {
                        i += 1;
                        continue;
                    }

                if let Some(value) = arg_obj.get("value") {
                    self.add_game_value(value);
                }
            }

            i += 1;
        }
    }

    fn evaluate_rules_with_options(&self, rules: &[Value]) -> bool {
        let mut allow = false;

        for rule in rules {
            if let Some(rule_obj) = rule.as_object() {
                let action = rule_obj
                    .get("action")
                    .and_then(|v| v.as_str())
                    .unwrap_or("allow");

                if let Some(features) = rule_obj.get("features").and_then(|v| v.as_object()) {
                    let mut feature_match = true;

                    if let Some(demo) = features.get("is_demo_user").and_then(|v| v.as_bool()) {
                        feature_match = demo == self.options.demo_mode;
                    }
                    if features.contains_key("has_custom_resolution") {
                        feature_match = true;
                    }
                    if features.contains_key("is_quick_play_singleplayer") {
                        feature_match = matches!(
                            self.options.quick_play_mode,
                            Some(QuickPlayMode::Singleplayer)
                        );
                    }
                    if features.contains_key("is_quick_play_multiplayer") {
                        feature_match = matches!(
                            self.options.quick_play_mode,
                            Some(QuickPlayMode::Multiplayer)
                        );
                    }
                    if features.contains_key("is_quick_play_realms") {
                        feature_match =
                            matches!(self.options.quick_play_mode, Some(QuickPlayMode::Realms));
                    }
                    if features.contains_key("has_quick_plays_support") {
                        feature_match = self.options.quick_play_mode.is_some();
                    }

                    if feature_match {
                        allow = action == "allow";
                    }
                }
            }
        }

        allow
    }

    fn add_game_value(&mut self, value: &Value) {
        if let Some(val_str) = value.as_str() {
            if self.should_filter_arg(val_str) {
                self.command.push(self.replace_vars(val_str));
            }
        } else if let Some(values) = value.as_array() {
            for val in values {
                if let Some(arg) = val.as_str()
                    && self.should_filter_arg(arg) {
                        self.command.push(self.replace_vars(arg));
                    }
            }
        }
    }

    fn should_filter_arg(&self, arg: &str) -> bool {
        if DEMO_ARGS.contains(&arg) && !self.options.demo_mode {
            return false;
        }
        if QUICKPLAY_ARGS.contains(&arg) && self.options.quick_play_mode.is_none() {
            return false;
        }
        true
    }

    fn add_legacy_args(&mut self) {
        let legacy_args = self.info.get_property("minecraftArguments").or_else(|| {
            self.info
                .base_version_data
                .as_ref()
                .and_then(|base| base.get("minecraftArguments"))
                .and_then(|v| v.as_str())
        });

        if let Some(args) = legacy_args {
            for arg in args.split_whitespace() {
                self.command.push(self.replace_vars(arg));
            }
        }
    }

    fn add_default_game_args(&mut self, width: u32, height: u32) {
        let defaults = vec![
            ("--width", width.to_string()),
            ("--height", height.to_string()),
            ("--assetIndex", self.info.get_assets_index_name()),
            ("--assetsDir", self.info.assets_dir.display().to_string()),
            (
                "--username",
                self.vars
                    .get("auth_player_name")
                    .unwrap_or(&String::new())
                    .clone(),
            ),
            (
                "--uuid",
                self.vars.get("auth_uuid").unwrap_or(&String::new()).clone(),
            ),
            (
                "--accessToken",
                self.vars
                    .get("auth_access_token")
                    .unwrap_or(&String::new())
                    .clone(),
            ),
            (
                "--version",
                if self.info.has_inheritance() {
                    self.info.base_version_id.as_ref().unwrap().clone()
                } else {
                    self.info.version_id.clone()
                },
            ),
            (
                "--gameDir",
                self.vars
                    .get("game_directory")
                    .unwrap_or(&String::new())
                    .clone(),
            ),
        ];

        for (key, value) in defaults {
            if !self.command.contains(&key.to_string()) && !value.is_empty() {
                self.command.push(key.to_string());
                self.command.push(value);
            }
        }
    }

    fn add_optional_args(&mut self) {
        if self.options.demo_mode && !self.command.contains(&"--demo".to_string()) {
            self.command.push("--demo".to_string());
        }

        if let (Some(mode), Some(value)) = (
            &self.options.quick_play_mode,
            &self.options.quick_play_value,
        ) {
            let quick_play_arg = match mode {
                QuickPlayMode::Singleplayer => "--quickPlaySingleplayer",
                QuickPlayMode::Multiplayer => "--quickPlayMultiplayer",
                QuickPlayMode::Realms => "--quickPlayRealms",
            };

            if !self.command.contains(&quick_play_arg.to_string()) {
                self.command.push(quick_play_arg.to_string());
                self.command.push(value.clone());
            }
        }
    }

    fn cleanup_unresolved_vars(&mut self) {
        let mut to_remove = Vec::new();

        for (i, arg) in self.command.iter().enumerate() {
            if arg.contains("${") {
                to_remove.push(i);
                if i > 0
                    && self.command[i - 1].starts_with("--")
                    && !self.command[i - 1].contains("${")
                {
                    to_remove.push(i - 1);
                }
            }
        }

        to_remove.sort_unstable();
        to_remove.reverse();
        to_remove.dedup();

        for idx in to_remove {
            if idx < self.command.len() {
                info!("Removing unresolved arg: {}", self.command[idx]);
                self.command.remove(idx);
            }
        }
    }

    fn replace_vars(&self, s: &str) -> String {
        let mut result = s.to_string();

        for (key, value) in &self.vars {
            result = result.replace(&format!("${{{}}}", key), value);
        }

        #[cfg(windows)]
        let separator = ";";
        #[cfg(not(windows))]
        let separator = ":";

        result
            .replace("${launcher_name}", "CubicLauncher")
            .replace("${launcher_version}", "1.0")
            .replace("${classpath_separator}", separator)
    }

    fn get_java_bin(&self, java_path: impl AsRef<Path>) -> String {
        let path = java_path.as_ref();
        if !path.exists() || !path.is_file() {
            panic!("Invalid Java path: {}", path.display());
        }
        path.to_string_lossy().to_string()
    }

    pub fn build(self) -> Vec<String> {
        self.command
    }
}
