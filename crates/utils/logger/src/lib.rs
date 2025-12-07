pub mod config;
#[cfg(feature = "file")]
pub mod file;
#[cfg(feature = "otel")]
pub mod otel;
pub mod tracing_unwrap;
pub mod util;

#[cfg(feature = "file")]
use crate::file::setup_file_appender;
#[cfg(feature = "otel")]
use crate::otel::setup_otel;
pub use crate::util::{utc_offset_hms, utc_offset_hours};
pub use config::*;
pub use time::UtcOffset;
use time::{format_description::BorrowedFormatItem, macros::format_description};
pub use tracing::{
    Level, debug, debug_span, error, error_span, info, info_span, instrument, span, trace,
    trace_span, warn, warn_span,
};

#[cfg(feature = "otel")]
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{EnvFilter, Registry, fmt::time::OffsetTime, layer::SubscriberExt};

#[cfg(feature = "sysinfo")]
pub mod sysinfo;

/// Error that occurs when setting up logging
#[derive(Debug, thiserror::Error)]
#[error("failed to setup logging")]
#[non_exhaustive]
pub struct SetupLogging {
    #[source]
    pub kind: SetupLoggingKind,
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SetupLoggingKind {
    #[error("invalid env filter directive '{directive}'")]
    #[non_exhaustive]
    InvalidEnvFilter {
        directive: String,
        #[source]
        source: tracing_subscriber::filter::ParseError,
    },

    #[error("missing {config_type} configuration")]
    #[non_exhaustive]
    MissingConfig { config_type: &'static str },

    #[error("failed to set global subscriber")]
    #[non_exhaustive]
    SetGlobalSubscriber {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[cfg(feature = "file")]
    #[error("file appender error")]
    #[non_exhaustive]
    FileAppender {
        #[source]
        source: FileAppenderError,
    },

    #[cfg(feature = "otel")]
    #[error("otel exporter error")]
    #[non_exhaustive]
    OtelExporter {
        #[source]
        source: OtelExporterError,
    },
}

impl SetupLogging {
    pub fn new(kind: SetupLoggingKind) -> Self {
        Self { kind }
    }

    pub fn invalid_env_filter(
        directive: impl Into<String>,
        source: tracing_subscriber::filter::ParseError,
    ) -> Self {
        Self::new(SetupLoggingKind::InvalidEnvFilter {
            directive: directive.into(),
            source,
        })
    }

    pub fn missing_config(config_type: &'static str) -> Self {
        Self::new(SetupLoggingKind::MissingConfig { config_type })
    }
}

#[cfg(feature = "file")]
/// Error that occurs when setting up file appender
#[derive(Debug, thiserror::Error)]
#[error("failed to setup file appender")]
#[non_exhaustive]
pub struct FileAppenderError {
    #[source]
    pub kind: FileAppenderErrorKind,
}

#[cfg(feature = "file")]
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum FileAppenderErrorKind {
    #[error("failed to create directory '{}'" , path.display())]
    #[non_exhaustive]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("path '{}' is not a directory", path.display())]
    #[non_exhaustive]
    NotDirectory { path: PathBuf },

    #[error("no write permission for directory '{}'", path.display())]
    #[non_exhaustive]
    NoWritePermission { path: PathBuf },

    #[error("failed to open log file '{}'", path.display())]
    #[non_exhaustive]
    OpenLogFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[cfg(feature = "file")]
impl FileAppenderError {
    pub fn new(kind: FileAppenderErrorKind) -> Self {
        Self { kind }
    }
}

#[cfg(feature = "otel")]
/// Error that occurs when setting up OpenTelemetry exporter
#[derive(Debug, thiserror::Error)]
#[error("failed to setup otel exporter")]
#[non_exhaustive]
pub struct OtelExporterError {
    #[source]
    pub kind: OtelExporterErrorKind,
}

#[cfg(feature = "otel")]
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum OtelExporterErrorKind {
    #[error("failed to build span exporter")]
    #[non_exhaustive]
    BuildSpanExporter {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("failed to build log exporter")]
    #[non_exhaustive]
    BuildLogExporter {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("failed to build metric exporter")]
    #[non_exhaustive]
    BuildMetricExporter {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

#[cfg(feature = "otel")]
impl OtelExporterError {
    pub fn new(kind: OtelExporterErrorKind) -> Self {
        Self { kind }
    }
}

#[cfg(feature = "file")]
use std::path::PathBuf;

#[cfg(feature = "sysinfo")]
/// Error that occurs when collecting system information
#[derive(Debug, thiserror::Error)]
#[error("failed to collect system information")]
#[non_exhaustive]
pub struct SysInfoError {
    #[source]
    pub kind: SysInfoErrorKind,
}

#[cfg(feature = "sysinfo")]
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SysInfoErrorKind {
    #[error("failed to get process id")]
    #[non_exhaustive]
    GetPid {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("process {pid} not found")]
    #[non_exhaustive]
    ProcessNotFound { pid: u32 },
}

#[cfg(feature = "sysinfo")]
impl SysInfoError {
    pub fn new(kind: SysInfoErrorKind) -> Self {
        Self { kind }
    }

    pub fn get_pid(source: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        Self::new(SysInfoErrorKind::GetPid {
            source: source.into(),
        })
    }

    pub fn process_not_found(pid: u32) -> Self {
        Self::new(SysInfoErrorKind::ProcessNotFound { pid })
    }
}

pub struct LoggingGuard {
    #[cfg(feature = "file")]
    /// Need to keep the guard alive to keep the file appender open
    pub file_guard: tracing_appender::non_blocking::WorkerGuard,
    #[cfg(feature = "otel")]
    /// Keep tracer provider alive for proper shutdown
    pub tracer_provider: Option<opentelemetry_sdk::trace::SdkTracerProvider>,
    #[cfg(feature = "otel")]
    /// Keep logger provider alive for proper shutdown
    pub logger_provider: Option<opentelemetry_sdk::logs::SdkLoggerProvider>,
    #[cfg(feature = "otel")]
    /// Keep meter provider alive for proper shutdown
    pub meter_provider: Option<opentelemetry_sdk::metrics::SdkMeterProvider>,
    #[cfg(feature = "stdout")]
    /// Keep stdout guard alive to ensure all logs are flushed
    pub stdout_guard: tracing_appender::non_blocking::WorkerGuard,
    /// Dummy field to ensure struct is never empty when all features are disabled
    #[cfg(not(any(feature = "file", feature = "stdout", feature = "otel")))]
    _dummy: (),
}

impl Drop for LoggingGuard {
    /// Shutdown all logging providers gracefully
    fn drop(&mut self) {
        #[cfg(feature = "otel")]
        if let Some(ref tracer) = self.tracer_provider
            && let Err(e) = tracer.force_flush()
        {
            eprintln!("Failed to force flush tracer provider: {}", e);
        }

        #[cfg(feature = "otel")]
        if let Some(ref logger) = self.logger_provider
            && let Err(e) = logger.force_flush()
        {
            eprintln!("Failed to force flush logger provider: {}", e);
        }

        #[cfg(feature = "otel")]
        if let Some(ref meter) = self.meter_provider
            && let Err(e) = meter.force_flush()
        {
            eprintln!("Failed to force flush meter provider: {}", e);
        }
        #[cfg(feature = "otel")]
        if let Some(ref tracer) = self.tracer_provider
            && let Err(e) = tracer.shutdown()
        {
            eprintln!("Failed to shutdown tracer provider: {}", e);
        }

        #[cfg(feature = "otel")]
        if let Some(ref logger) = self.logger_provider
            && let Err(e) = logger.shutdown()
        {
            eprintln!("Failed to shutdown logger provider: {}", e);
        }

        #[cfg(feature = "otel")]
        if let Some(ref meter) = self.meter_provider
            && let Err(e) = meter.shutdown()
        {
            eprintln!("Failed to shutdown meter provider: {}", e);
        }
    }
}

pub fn setup_logging(
    app_name: impl Into<String>,
    timezone_offset: Option<i8>,
    logger_config: LoggerConfig,
    env_filter_override: Option<Vec<&str>>,
) -> Result<LoggingGuard, SetupLogging> {
    #[cfg_attr(not(any(feature = "file", feature = "otel")), allow(unused_variables))]
    let app_name: String = app_name.into();
    let fmt: &[BorrowedFormatItem<'_>] = if cfg!(debug_assertions) {
        format_description!("[hour]:[minute]:[second].[subsecond digits:3]")
    } else {
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]")
    };

    let timezone = match timezone_offset {
        Some(offset) => utc_offset_hours(offset),
        None => UtcOffset::UTC,
    };
    let timer = OffsetTime::new(timezone, fmt);

    let max_level = logger_config
        .max_level
        .parse::<Level>()
        .unwrap_or(Level::INFO);

    let mut env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    if let Some(directives) = env_filter_override {
        for dir in directives {
            let directive = dir
                .parse()
                .map_err(|e| SetupLogging::invalid_env_filter(dir, e))?;
            env_filter = env_filter.add_directive(directive);
        }
    }

    let level_filter = tracing_subscriber::filter::LevelFilter::from_level(max_level);

    let registry = Registry::default();

    #[cfg(feature = "otel")]
    let (registry, tracer_provider, logger_provider, meter_provider) = {
        if let Some(otel_config) = logger_config.otel.as_ref() {
            let (otel_layer, tracer, logger, meter) =
                setup_otel(app_name.clone(), otel_config.clone())?;
            let bridge =
                opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(&logger);
            (
                registry.with(Some(otel_layer)).with(Some(bridge)),
                Some(tracer),
                Some(logger),
                Some(meter),
            )
        } else {
            // When otel feature is enabled but no config provided, use empty layers
            (registry.with(None).with(None), None, None, None)
        }
    };

    #[cfg(not(feature = "otel"))]
    let (_tracer_provider, _logger_provider, _meter_provider): (
        Option<()>,
        Option<()>,
        Option<()>,
    ) = (None, None, None);

    let registry = registry.with(env_filter).with(level_filter);

    #[cfg(feature = "file")]
    let (registry, file_guard) = {
        let file_layer = logger_config
            .file
            .as_ref()
            .filter(|fc| fc.enabled)
            .map(|file_config| {
                let (non_blocking, guard) =
                    setup_file_appender(app_name.clone(), file_config.clone())?;
                let file_format = file_config
                    .format
                    .as_ref()
                    .or(logger_config.format.as_ref());
                let layer = tracing_subscriber::fmt::Layer::default()
                    .with_writer(non_blocking)
                    .with_timer(timer.clone())
                    .with_ansi(file_format.map(|f| f.ansi).unwrap_or(false))
                    .with_target(file_format.map(|f| f.target).unwrap_or(true))
                    .with_file(file_format.map(|f| f.file).unwrap_or(true))
                    .with_line_number(file_format.map(|f| f.line_number).unwrap_or(true));
                Ok::<_, SetupLogging>((layer, guard))
            })
            .transpose()?;

        let (layer, guard) = file_layer.unzip();
        let guard = guard.unwrap_or_else(|| {
            let (_, g) = tracing_appender::non_blocking(std::io::sink());
            g
        });
        (registry.with(layer), guard)
    };

    #[cfg(not(feature = "file"))]
    let registry = registry;

    #[cfg(feature = "stdout")]
    let (registry, stdout_guard) = {
        let stdout_layer = logger_config.format.as_ref().map(|stdout_format| {
            let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stdout());
            let layer = tracing_subscriber::fmt::Layer::default()
                .with_writer(non_blocking)
                .with_timer(timer)
                .with_ansi(stdout_format.ansi)
                .with_target(stdout_format.target)
                .with_file(stdout_format.file)
                .with_line_number(stdout_format.line_number);
            (layer, guard)
        });

        let (layer, guard) = stdout_layer.unzip();
        let guard = guard.unwrap_or_else(|| {
            let (_, g) = tracing_appender::non_blocking(std::io::sink());
            g
        });
        (registry.with(layer), guard)
    };

    #[cfg(not(feature = "stdout"))]
    let registry = registry;

    if tracing::dispatcher::has_been_set() {
        warn!("Global trace dispatcher already set, skipping re-init");
    } else {
        tracing::subscriber::set_global_default(registry).map_err(|e| {
            SetupLogging::new(SetupLoggingKind::SetGlobalSubscriber {
                source: Box::new(e),
            })
        })?;
    }

    Ok(LoggingGuard {
        #[cfg(feature = "file")]
        file_guard,
        #[cfg(feature = "otel")]
        tracer_provider,
        #[cfg(feature = "otel")]
        logger_provider,
        #[cfg(feature = "otel")]
        meter_provider,
        #[cfg(feature = "stdout")]
        stdout_guard,
        #[cfg(not(any(feature = "file", feature = "stdout", feature = "otel")))]
        _dummy: (),
    })
}
