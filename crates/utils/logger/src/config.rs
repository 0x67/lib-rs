use serde::{Deserialize, Serialize};
#[cfg(feature = "otel")]
use std::time::Duration;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct LoggerConfig {
    pub max_level: String,
    #[cfg(feature = "file")]
    pub file: Option<FileConfig>,
    #[cfg(feature = "otel")]
    pub otel: Option<OtelConfig>,
    pub format: Option<FormatConfig>,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            max_level: "INFO".to_string(),
            #[cfg(feature = "file")]
            file: None,
            #[cfg(feature = "otel")]
            otel: None,
            format: Some(FormatConfig::default()),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct FormatConfig {
    /// Enable ANSI color codes in output
    pub ansi: bool,
    /// Include target (module path) in log output
    pub target: bool,
    /// Include file name in log output
    pub file: bool,
    /// Include line number in log output
    pub line_number: bool,
    /// Include span events in log output (enter/exit events)
    #[serde(default = "default_true")]
    pub with_span_events: bool,
}

fn default_true() -> bool {
    true
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            ansi: true,
            target: true,
            file: true,
            line_number: true,
            with_span_events: true,
        }
    }
}

#[cfg(feature = "file")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct FileConfig {
    /// Maximum size in bytes before rotation
    pub max_size: u64,
    /// Directory path for log files
    pub path: String,
    /// Enable file logging
    pub enabled: bool,
    /// Format configuration for file output (overrides global format if set)
    pub format: Option<FormatConfig>,
}

#[cfg(feature = "file")]
impl Default for FileConfig {
    fn default() -> Self {
        Self {
            max_size: 100 * 1024 * 1024, // 100MB
            path: "./logs".to_string(),
            enabled: false,
            format: None,
        }
    }
}

#[cfg(feature = "otel")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct OtelConfig {
    /// OpenTelemetry collector endpoint
    pub endpoint: String,
    /// Enable OpenTelemetry exporter
    pub enabled: bool,
    /// Trace sampler configuration
    pub sampler: Option<SamplerConfig>,
    /// Export timeout in seconds
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
    /// Maximum queue size for batching
    #[serde(default = "default_max_queue_size")]
    pub max_queue_size: usize,
    /// Delay between export attempts in milliseconds
    #[serde(default = "default_scheduled_delay_ms")]
    pub scheduled_delay_ms: u64,
    /// Maximum export batch size
    #[serde(default = "default_max_export_batch_size")]
    pub max_export_batch_size: usize,
    /// Maximum events per span
    #[serde(default = "default_max_events_per_span")]
    pub max_events_per_span: u32,
    /// Maximum attributes per span
    #[serde(default = "default_max_attributes_per_span")]
    pub max_attributes_per_span: u32,
}

#[cfg(feature = "otel")]
fn default_timeout_secs() -> u64 {
    3
}

#[cfg(feature = "otel")]
fn default_max_queue_size() -> usize {
    65536
}

#[cfg(feature = "otel")]
fn default_scheduled_delay_ms() -> u64 {
    200
}

#[cfg(feature = "otel")]
fn default_max_export_batch_size() -> usize {
    512
}

#[cfg(feature = "otel")]
fn default_max_events_per_span() -> u32 {
    64
}

#[cfg(feature = "otel")]
fn default_max_attributes_per_span() -> u32 {
    16
}

#[cfg(feature = "otel")]
impl Default for OtelConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:4317".to_string(),
            enabled: false,
            sampler: Some(SamplerConfig::default()),
            timeout_secs: default_timeout_secs(),
            max_queue_size: default_max_queue_size(),
            scheduled_delay_ms: default_scheduled_delay_ms(),
            max_export_batch_size: default_max_export_batch_size(),
            max_events_per_span: default_max_events_per_span(),
            max_attributes_per_span: default_max_attributes_per_span(),
        }
    }
}

#[cfg(feature = "otel")]
impl OtelConfig {
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs)
    }

    pub fn scheduled_delay(&self) -> Duration {
        Duration::from_millis(self.scheduled_delay_ms)
    }
}

#[cfg(feature = "otel")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum SamplerConfig {
    /// Sample all traces
    AlwaysOn,
    /// Sample no traces
    AlwaysOff,
    /// Sample traces based on parent span decision
    ParentBased {
        #[serde(default)]
        root: Box<SamplerConfig>,
    },
    /// Sample a fraction of traces (0.0 to 1.0)
    TraceIdRatioBased { ratio: f64 },
}

#[cfg(feature = "otel")]
impl Default for SamplerConfig {
    fn default() -> Self {
        Self::TraceIdRatioBased { ratio: 1.0 }
    }
}

#[cfg(feature = "otel")]
impl SamplerConfig {
    pub fn to_sampler(&self) -> opentelemetry_sdk::trace::Sampler {
        use opentelemetry_sdk::trace::Sampler;

        match self {
            Self::AlwaysOn => Sampler::AlwaysOn,
            Self::AlwaysOff => Sampler::AlwaysOff,
            Self::ParentBased { root } => Sampler::ParentBased(Box::new(root.to_sampler())),
            Self::TraceIdRatioBased { ratio } => Sampler::TraceIdRatioBased(*ratio),
        }
    }
}
