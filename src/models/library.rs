// Copyright (C) 2025 Santiagolxx, Notstaff and CubicLauncher contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::utils::json_utils;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// Represents a library with all its information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloads: Option<Downloads>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Downloads {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<Artifact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classifiers: Option<serde_json::Map<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

impl Library {
    /// Check if this library should be included based on rules
    pub fn should_include(&self) -> bool {
        match &self.rules {
            Some(rules) => json_utils::evaluate_rules(rules),
            None => true,
        }
    }

    /// Get artifact path
    pub fn get_artifact_path(&self) -> Option<&str> {
        self.downloads
            .as_ref()
            .and_then(|d| d.artifact.as_ref())
            .map(|a| a.path.as_str())
    }

    /// Resolve library path
    pub fn resolve_path(&self, lib_dir: &Path) -> Option<PathBuf> {
        if let Some(path) = self.get_artifact_path() {
            return Some(lib_dir.join(path));
        }

        // Build path from name
        let parts: Vec<&str> = self.name.split(':').collect();
        if parts.len() >= 3 {
            let group_path = parts[0].replace('.', "/");
            let artifact = parts[1];
            let version = parts[2];
            let classifier = if parts.len() > 3 {
                format!("-{}", parts[3])
            } else {
                String::new()
            };

            return Some(
                lib_dir
                    .join(group_path)
                    .join(artifact)
                    .join(version)
                    .join(format!("{}-{}{}.jar", artifact, version, classifier)),
            );
        }

        None
    }

    /// Get group ID
    pub fn group_id(&self) -> Option<&str> {
        self.name.split(':').next()
    }

    /// Get artifact ID
    pub fn artifact_id(&self) -> Option<&str> {
        self.name.split(':').nth(1)
    }

    /// Get library key for conflict resolution
    pub fn key(&self) -> String {
        match (self.group_id(), self.artifact_id()) {
            (Some(group), Some(artifact)) => format!("{}:{}", group, artifact),
            _ => self.name.clone(),
        }
    }
}
