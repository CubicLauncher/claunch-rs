// Copyright (C) 2025 Santiagolxx, Notstaff and CubicLauncher contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

use serde_json::Value;
use std::fs;
use std::path::Path;

/// Load JSON from file
pub fn load_json(file_path: impl AsRef<Path>) -> Result<Value, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let value: Value = serde_json::from_str(&content)?;
    Ok(value)
}

/// Get arguments from version JSON
pub fn get_args_from_version<'a>(
    version_data: &'a Value,
    arg_type: &'a str,
) -> Option<&'a Vec<Value>> {
    version_data
        .get("arguments")
        .and_then(|args| args.get(arg_type))
        .and_then(|v| v.as_array())
}

/// Evaluate OS and feature rules
pub fn evaluate_rules(rules: &[Value]) -> bool {
    let mut allow = false;
    let current_os = std::env::consts::OS;
    let current_arch = std::env::consts::ARCH;

    for rule in rules {
        let action = rule
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("allow");

        if let Some(os) = rule.get("os") {
            let name = os.get("name").and_then(|v| v.as_str());
            let arch = os.get("arch").and_then(|v| v.as_str());

            let os_match = name.is_none_or(|n| match_os(n, current_os));
            let arch_match = arch.is_none_or(|a| match_arch(a, current_arch));

            if os_match && arch_match {
                allow = action == "allow";
            }
        } else {
            allow = action == "allow";
        }
    }

    allow
}

fn match_os(name: &str, current_os: &str) -> bool {
    match name {
        "windows" => current_os == "windows",
        "linux" => current_os == "linux",
        "osx" => current_os == "macos",
        _ => false,
    }
}

fn match_arch(arch: &str, current_arch: &str) -> bool {
    match arch {
        "x86" => current_arch == "x86" || current_arch == "i686",
        "x64" => current_arch == "x86_64" || current_arch == "aarch64",
        _ => true,
    }
}
