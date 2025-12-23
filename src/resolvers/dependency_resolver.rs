// Copyright (C) 2025 Santiagolxx, Notstaff and CubicLauncher contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::models::{Library, VersionInfo};
use log::{debug, info, warn};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct DependencyResolver {
    added_paths: Vec<String>,
    library_keys: HashMap<String, String>,
    lib_dir: PathBuf,
}

impl DependencyResolver {
    pub fn new(lib_dir: PathBuf, _natives_dir: PathBuf) -> Self {
        Self {
            added_paths: Vec::new(),
            library_keys: HashMap::new(),
            lib_dir,
        }
    }

    pub fn process_version(&mut self, version_data: &Value, is_child: bool) {
        let Some(libs_array) = version_data.get("libraries").and_then(|v| v.as_array()) else {
            return;
        };

        for lib_value in libs_array {
            if let Ok(lib) = serde_json::from_value::<Library>(lib_value.clone()) {
                if !lib.should_include() {
                    continue;
                }

                self.add_artifact(&lib, is_child);
            }
        }
    }

    fn add_artifact(&mut self, lib: &Library, is_child: bool) {
        if let Some(path) = lib.get_artifact_path() {
            if self.is_native_jar(path) {
                debug!("Skipping native JAR from classpath: {}", path);
                return;
            }

            let full_path = self.lib_dir.join(path);
            self.add_path(&full_path, &lib.name, &lib.key(), is_child);
        } else if let Some(path) = lib.resolve_path(&self.lib_dir) {
            let path_str = path.to_string_lossy();
            if self.is_native_jar(&path_str) {
                debug!("Skipping native JAR from classpath: {}", path_str);
                return;
            }
            self.add_path(&path, &lib.name, &lib.key(), is_child);
        }
    }

    /// Detect if a JAR is a native library
    fn is_native_jar(&self, path: &str) -> bool {
        let lower = path.to_lowercase();
        lower.contains("-natives-")
            || lower.contains("/natives/")
            || lower.ends_with("-natives.jar")
            || (lower.contains("lwjgl")
                && (lower.contains("-linux")
                    || lower.contains("-windows")
                    || lower.contains("-macos")
                    || lower.contains("-freebsd")))
    }

    fn add_path(&mut self, path: &Path, description: &str, library_key: &str, is_child: bool) {
        if !path.exists() {
            warn!("Library not found: {} -> {}", description, path.display());
            return;
        }

        let path_str = path.to_string_lossy().to_string();

        if let Some(existing_path) = self.library_keys.get(library_key) {
            if is_child {
                info!(
                    "Library conflict resolved - Child priority: {}",
                    library_key
                );
                info!("  Replacing: {}", existing_path);
                info!("  With: {}", path_str);

                // Remover el path anterior y agregar el nuevo
                self.added_paths.retain(|p| p != existing_path);
                self.added_paths.push(path_str.clone());
                self.library_keys.insert(library_key.to_string(), path_str);
            }
        } else {
            self.library_keys
                .insert(library_key.to_string(), path_str.clone());
            // Solo agregar si no existe ya (para mantener unicidad)
            if !self.added_paths.contains(&path_str) {
                self.added_paths.push(path_str.clone());
                debug!("Added library: {} -> {}", description, path_str);
            }
        }
    }

    /// Build classpath string
    pub fn build_classpath(&self, info: &VersionInfo) -> String {
        let mut classpath: Vec<String> = self.added_paths.clone();
        self.add_version_jars(&mut classpath, info);

        #[cfg(windows)]
        let separator = ";";
        #[cfg(not(windows))]
        let separator = ":";

        classpath.join(separator)
    }

    fn add_version_jars(&self, classpath: &mut Vec<String>, info: &VersionInfo) {
        let loader_type = self.detect_loader(&info.version_id);
        let client_jar = info.get_client_jar();
        let version_jar = info.get_version_jar();

        info!("Loader: {}", loader_type);

        match loader_type {
            "forge" => {
                self.add_if_exists(classpath, &client_jar);
                self.add_if_exists(classpath, &version_jar);
                if let Some(forge_jar) = self.find_forge_universal_jar(info) {
                    self.add_if_exists(classpath, &forge_jar);
                }
            }
            "neoforge" => {
                self.add_if_exists(classpath, &version_jar);
            }
            _ => {
                self.add_if_exists(classpath, &client_jar);
                if client_jar != version_jar {
                    self.add_if_exists(classpath, &version_jar);
                }
            }
        }
    }

    fn add_if_exists(&self, classpath: &mut Vec<String>, jar: &Path) {
        if jar.exists() {
            let jar_path = jar.to_string_lossy().to_string();
            if !classpath.contains(&jar_path) {
                classpath.push(jar_path.clone());
                debug!("Added JAR to classpath: {}", jar_path);
            }
        }
    }

    fn find_forge_universal_jar(&self, info: &VersionInfo) -> Option<PathBuf> {
        self.search_forge_in_libraries(&info.version_data)
            .or_else(|| {
                info.base_version_data
                    .as_ref()
                    .and_then(|base| self.search_forge_in_libraries(base))
            })
    }

    fn search_forge_in_libraries(&self, version_data: &Value) -> Option<PathBuf> {
        let libs = version_data.get("libraries")?.as_array()?;

        for lib_value in libs {
            if let Ok(lib) = serde_json::from_value::<Library>(lib_value.clone())
                && (lib.name.contains("net.minecraftforge:forge:")
                    || lib.name.contains("net.minecraftforge:minecraftforge:"))
                {
                    return self.build_forge_path(&lib.name);
                }
        }
        None
    }

    fn build_forge_path(&self, name: &str) -> Option<PathBuf> {
        let parts: Vec<&str> = name.split(':').collect();
        if parts.len() < 3 {
            return None;
        }

        let group_path = parts[0].replace('.', "/");
        let artifact = parts[1];
        let version = parts[2];

        let universal = self
            .lib_dir
            .join(&group_path)
            .join(artifact)
            .join(version)
            .join(format!("{}-{}-universal.jar", artifact, version));

        if universal.exists() {
            return Some(universal);
        }

        Some(
            self.lib_dir
                .join(group_path)
                .join(artifact)
                .join(version)
                .join(format!("{}-{}.jar", artifact, version)),
        )
    }

    fn detect_loader(&self, version_id: &str) -> &str {
        let lower = version_id.to_lowercase();
        if lower.contains("neoforge") {
            "neoforge"
        } else if lower.contains("forge") {
            "forge"
        } else if lower.contains("fabric") {
            "fabric"
        } else {
            "vanilla"
        }
    }

    pub fn library_count(&self) -> usize {
        self.added_paths.len()
    }
}
