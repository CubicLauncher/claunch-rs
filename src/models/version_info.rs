// Copyright (C) 2025 Santiagolxx, Notstaff and CubicLauncher contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::utils::json_utils;
use serde_json::Value;
use std::path::{Path, PathBuf};

/// Encapsulates complete version information with inheritance support
#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub version_data: Value,
    pub base_version_data: Option<Value>,
    pub version_id: String,
    pub base_version_id: Option<String>,
    pub resolved_version_id: String,
    pub shared_dir: PathBuf,
    pub instance_dir: PathBuf,
    pub lib_dir: PathBuf,
    pub assets_dir: PathBuf,
    pub natives_dir: PathBuf,
    pub minimum_jre_version: String,
}

impl VersionInfo {
    /// Create a new VersionInfo from a version JSON file
    pub fn new(
        version_json_path: impl AsRef<Path>,
        shared_dir: impl AsRef<Path>,   // <-- separado
        instance_dir: impl AsRef<Path>, // <-- separado
    ) -> crate::Result<Self> {
        let version_json_path = version_json_path.as_ref();
        let shared_dir = shared_dir.as_ref().to_path_buf();
        let instance_dir = instance_dir.as_ref().to_path_buf();

        let version_data = json_utils::load_json(version_json_path).map_err(|e| {
            crate::Error::VersionLoadFailed(format!(
                "Failed to load version file {}: {}",
                version_json_path.display(),
                e
            ))
        })?;

        let version_id = version_data["id"]
            .as_str()
            .ok_or_else(|| {
                crate::Error::VersionLoadFailed(format!(
                    "Missing version id in {}",
                    version_json_path.display()
                ))
            })?
            .to_string();

        // Cargar versión padre si existe
        let (base_version_data, base_version_id, resolved_version_id) = if let Some(inherits_from) =
            version_data.get("inheritsFrom").and_then(|v| v.as_str())
        {
            let base_json_path = shared_dir
                .join("versions")
                .join(inherits_from)
                .join(format!("{}.json", inherits_from));

            let base_data = json_utils::load_json(&base_json_path).map_err(|e| {
                crate::Error::BaseVersionLoadFailed(format!(
                    "Failed to load base version {}: {}",
                    base_json_path.display(),
                    e
                ))
            })?;

            (
                Some(base_data),
                Some(inherits_from.to_string()),
                inherits_from.to_string(),
            )
        } else {
            (None, None, version_id.clone())
        };

        let minimum_jre_version = Self::resolve_java_version(&version_data, &base_version_data);

        let lib_dir = shared_dir.join("libraries");
        let assets_dir = shared_dir.join("assets");
        let natives_dir = shared_dir.join("natives").join(&resolved_version_id);

        if !lib_dir.exists() {
            log::warn!("Libraries directory does not exist: {}", lib_dir.display());
        }

        Ok(Self {
            version_data,
            base_version_data,
            version_id,
            base_version_id,
            resolved_version_id,
            shared_dir,
            instance_dir,
            lib_dir,
            assets_dir,
            natives_dir,
            minimum_jre_version,
        })
    }

    fn resolve_java_version(version_data: &Value, base_version_data: &Option<Value>) -> String {
        // Try child version first
        if let Some(java_ver) = version_data.get("javaVersion")
            && let Some(major) = java_ver.get("majorVersion").and_then(|v| v.as_u64())
        {
            return major.to_string();
        }

        // Try parent version
        if let Some(base) = base_version_data
            && let Some(java_ver) = base.get("javaVersion")
            && let Some(major) = java_ver.get("majorVersion").and_then(|v| v.as_u64())
        {
            return major.to_string();
        }

        "0".to_string()
    }

    /// Get property with inheritance fallback
    pub fn get_property(&self, key: &str) -> Option<&str> {
        self.version_data
            .get(key)
            .and_then(|v| v.as_str())
            .or_else(|| {
                self.base_version_data
                    .as_ref()
                    .and_then(|base| base.get(key))
                    .and_then(|v| v.as_str())
            })
    }

    pub fn get_client_jar(&self) -> PathBuf {
        self.shared_dir
            .join("versions")
            .join(&self.resolved_version_id)
            .join(format!("{}.jar", self.resolved_version_id))
    }

    pub fn get_version_jar(&self) -> PathBuf {
        self.shared_dir
            .join("versions")
            .join(&self.version_id)
            .join(format!("{}.jar", self.version_id))
    }

    /// Get assets index name
    pub fn get_assets_index_name(&self) -> String {
        self.get_property("assets").unwrap_or("legacy").to_string()
    }

    /// Get assets virtual directory
    pub fn get_assets_virtual_dir(&self) -> PathBuf {
        self.assets_dir
            .join("virtual")
            .join(self.get_assets_index_name())
    }

    /// Check if version has inheritance
    pub fn has_inheritance(&self) -> bool {
        self.base_version_id.is_some()
    }
}