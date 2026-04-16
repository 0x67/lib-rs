use serde::{Deserialize, Serialize};

use crate::config::{ProtocolConfig, default_protocol, default_timeout_secs};

fn default_interval_secs() -> u64 {
    60
}

fn default_system_interval_secs() -> u64 {
    15
}

fn default_endpoint() -> String {
    "http://localhost:4317".to_string()
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MetricsConfig {
    pub otlp: OtlpMetricsConfig,
    pub system: SystemMetricsConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct OtlpMetricsConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_endpoint")]
    pub endpoint: String,
    pub metrics_path: Option<String>,
    #[serde(default = "default_protocol")]
    pub protocol: ProtocolConfig,
    #[serde(default = "default_interval_secs")]
    pub interval_secs: u64,
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
    pub headers: Option<std::collections::HashMap<String, String>>,
    pub attributes: Option<std::collections::HashMap<String, String>>,
}

impl Default for OtlpMetricsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoint: default_endpoint(),
            metrics_path: None,
            protocol: ProtocolConfig::Grpc,
            interval_secs: default_interval_secs(),
            timeout_secs: default_timeout_secs(),
            headers: None,
            attributes: None,
        }
    }
}

impl OtlpMetricsConfig {
    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.timeout_secs)
    }

    pub fn interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.interval_secs)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct SystemMetricsConfig {
    #[serde(default = "default_system_enabled")]
    pub enabled: bool,
    #[serde(default = "default_system_interval_secs")]
    pub interval_secs: u64,
}

fn default_system_enabled() -> bool {
    true
}

impl Default for SystemMetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: default_system_interval_secs(),
        }
    }
}

impl SystemMetricsConfig {
    pub fn interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.interval_secs)
    }
}
