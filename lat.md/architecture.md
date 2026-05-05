# lib-rs

Shared Rust utility library providing config loading, ID generation, HTTP client, structured logging with OpenTelemetry, and async task management.

## Architecture

Five independent crates under `crates/utils/`, published as separate libraries. Each crate follows library design principles: explicit error types, `#[non_exhaustive]` for forward compatibility, feature-gated optional functionality.

### Dependency Graph

`task-manager` depends on `logger` for structured logging. `config-loader` depends on `http-client` for remote config fetching. Other crates are independent.

### Design Principles

Library-first design — no `anyhow` in public APIs, explicit error types with `#[non_exhaustive]`, serde-based configuration, feature gates for optional heavy dependencies (OpenTelemetry, jemalloc, SIMD).

## Crates

Overview of each utility crate in the workspace.

### config-loader

Async-first configuration loader supporting JSON, YAML, TOML, JSON5, environment variables, and remote HTTP sources via [[crates/utils/http-client/src/builder.rs#HttpClientBuilder]].

### gen-id

Flexible ID generation — UUID v4/v7 (with optional SIMD acceleration), NanoID, and custom UUID with embedded client metadata extraction.

### http-client

Production HTTP client built on reqwest with automatic retries, SSL certificate pinning, compression (brotli/gzip/deflate/zstd), and optional OpenTelemetry distributed tracing.

### logger

Structured tracing-based logging with multi-destination output (stdout, file rotation), OpenTelemetry integration (traces, logs, metrics), system metrics collection, and jemalloc heap introspection. See [[lat.md/logger#Logger]] for details.

### task-manager

Async task orchestration with graceful shutdown via OS signals (SIGINT/SIGTERM), error propagation, cancellation tokens, and optional CPU core affinity pinning.
