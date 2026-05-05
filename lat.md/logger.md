# Logger

Structured logging library built on `tracing` + `tracing-subscriber` with multi-destination output, OpenTelemetry integration, and system metrics collection.

## Configuration

All config structs use `#[non_exhaustive]` and `serde::Deserialize` for forward-compatible deserialization. Fluent move-based `with_*()` builder methods available on all config types for ergonomic construction.

### LoggerConfig

Top-level config for log level, output mode, file output, OTel export, and stdout formatting.

Fields: `max_level`, `output_mode: OutputMode`, `file: Option<FileConfig>`, `otel: Option<OtelConfig>`, `format: Option<FormatConfig>`.

### OutputMode

Enum controlling which output layers `setup_logging` registers. Variants: `Stdout`, `File`, `Both` (default), `None`. Defined in [[crates/utils/logger/src/config.rs#OutputMode]]. Helper methods `enables_stdout()` and `enables_file()` provide predicate checks.

### FormatConfig

Controls log output formatting — ANSI colors, target module display, source file/line numbers, and span event tracing. Used by both stdout and file layers independently.

### FileConfig

File-based log output with size-based rotation. Fields: `enabled`, `path`, `max_size` (bytes), optional per-file `format` override.

### OtelConfig

OpenTelemetry exporter config for traces, logs, and metrics. Supports gRPC and HTTP protocols, configurable sampling strategies, batch export tuning, custom headers and resource attributes.

### SamplerConfig

Trace sampling strategy enum — `AlwaysOn`, `AlwaysOff`, `ParentBased(Box<SamplerConfig>)`, `TraceIdRatioBased(f64)`.

## Setup

Core initialization via [[crates/utils/logger/src/lib.rs#setup_logging]].

### setup_logging

Builds layered `tracing` subscriber with env filter, optional OTel/file/stdout layers. Returns `LoggingGuard` holding provider handles.

Layers registered conditionally based on feature gates (`stdout`, `file`, `otel`) and config values.

### Output Control

Controlled by feature gates, `OutputMode`, and config fields.

Stdout layer activates when `stdout` feature enabled AND `output_mode.enables_stdout()` AND `format` is `Some(...)`. File layer activates when `file` feature enabled AND `output_mode.enables_file()` AND `FileConfig.enabled` is true. `OutputMode::None` disables both stdout and file (OTel-only or silent mode).

## Metrics

OpenTelemetry metrics subsystem behind `metrics` feature gate.

### MetricsConfig

Two sub-configs: `OtlpMetricsConfig` (OTLP exporter: endpoint, protocol, interval, timeout, headers, attributes) and `SystemMetricsConfig` (process/system metric collection interval).

### setup_metrics

Creates OTLP metric exporter, builds `SdkMeterProvider`, sets global provider, optionally spawns async system metrics collection task. Returns `MetricsGuard`.

### System Metrics

Collects process CPU utilization, memory (physical/virtual), disk I/O, network I/O at configurable intervals. Behind `jemalloc` feature: also collects heap stats (allocated, resident, mapped, retained, metadata).

## Error Types

All error types use `#[non_exhaustive]` with thiserror derives. Structured as paired `Error` + `ErrorKind` enums for pattern matching while preserving forward compatibility.

### Error Catalog

Paired Error + ErrorKind types for each failure domain.

- `SetupLogging` / `SetupLoggingKind` — logging initialization failures
- `FileAppenderError` / `FileAppenderErrorKind` — file creation/permission issues
- `OtelExporterError` / `OtelExporterErrorKind` — OTLP connection/build failures
- `SysInfoError` / `SysInfoErrorKind` — system info collection failures

## Known Limitations

Current design constraints that downstream consumers should be aware of.

### Non-exhaustive Config Structs

All config structs marked `#[non_exhaustive]` for SemVer safety, but this adds friction for consumers constructing configs programmatically. Serde already handles unknown fields gracefully, so config structs may not need this annotation.
