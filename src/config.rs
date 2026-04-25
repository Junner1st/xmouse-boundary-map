use std::fs;
use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::geometry::{EdgeMapping, MappingMode, Side};

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_poll_interval_ms")]
    pub poll_interval_ms: u64,
    #[serde(default = "default_warp_cooldown_ms")]
    pub warp_cooldown_ms: u64,
    #[serde(default = "default_ignore_drag")]
    pub ignore_drag: bool,
    #[serde(default)]
    pub edge: Vec<EdgeConfig>,
}

#[derive(Debug, Deserialize)]
pub struct EdgeConfig {
    pub from: String,
    pub to: String,
    pub side: Side,
    #[serde(default)]
    pub map: MappingMode,
}

impl AppConfig {
    pub fn load_optional(path: Option<&Path>) -> Result<Self> {
        match path {
            Some(path) => {
                let raw = fs::read_to_string(path)
                    .with_context(|| format!("read config {}", path.display()))?;
                toml::from_str(&raw).with_context(|| format!("parse config {}", path.display()))
            }
            None => Ok(Self::default()),
        }
    }

    pub fn edge_mappings(&self) -> Result<Vec<EdgeMapping>> {
        self.edge
            .iter()
            .map(|edge| {
                Ok(EdgeMapping {
                    from: edge.from.clone(),
                    to: edge.to.clone(),
                    side: edge.side,
                    mode: edge.map,
                })
            })
            .collect()
    }

    pub fn warp_cooldown(&self) -> Duration {
        Duration::from_millis(self.warp_cooldown_ms)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            poll_interval_ms: default_poll_interval_ms(),
            warp_cooldown_ms: default_warp_cooldown_ms(),
            ignore_drag: default_ignore_drag(),
            edge: Vec::new(),
        }
    }
}

fn default_poll_interval_ms() -> u64 {
    8
}

fn default_warp_cooldown_ms() -> u64 {
    50
}

fn default_ignore_drag() -> bool {
    true
}
