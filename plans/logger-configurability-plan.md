## Plan: Logger Configurability & Ergonomics

Add `OutputMode` enum, builder API on all config types, and config-level stdout control. Eliminates `#[non_exhaustive]` friction without removing the annotation. All `#[non_exhaustive]` annotations stay — justified for published library forward compatibility.

**Decisions:**

- Builder style: move-based `with_*()` returning `Self` for clean chaining
- `OutputMode::default()` = `Both` for backward compat
- `#[non_exhaustive]` stays on all types — builders solve ergonomics
- No downstream consumer changes — lib-rs workspace only
- No rewrite — architecture is sound, problems are incremental

**Phases: 3**

1. **Phase 1: Add OutputMode Enum + Fix Stdout Control**
   - **Objective:** Add `OutputMode` enum (`Stdout`, `File`, `Both`, `None`) to `LoggerConfig`. Modify `setup_logging()` to conditionally register stdout/file layers based on mode, fixing the bug where stdout always registers when feature is compiled in.
   - **Files/Functions to Modify/Create:**
     - `crates/utils/logger/src/config.rs` — add `OutputMode` enum, add `output_mode` field to `LoggerConfig`
     - `crates/utils/logger/src/lib.rs` — `setup_logging()` checks `output_mode` before registering stdout/file layers
   - **Tests to Write:**
     - `test_output_mode_stdout_only` — only stdout layer active
     - `test_output_mode_file_only` — only file layer active
     - `test_output_mode_both` — both layers
     - `test_output_mode_none` — no output layers (OTel-only mode)
     - `test_output_mode_default_backward_compat` — default matches current behavior
     - `test_stdout_disabled_via_output_mode` — stdout feature enabled but `OutputMode::File` → no stdout layer
   - **Steps:**
     1. Write tests for `OutputMode` variants and stdout control
     2. Run tests (expect fail)
     3. Add `OutputMode` enum with serde support + `#[non_exhaustive]`
     4. Add `output_mode: OutputMode` field to `LoggerConfig` defaulting to `Both`
     5. Modify `setup_logging()` — check `output_mode` before registering stdout/file layers
     6. Run tests (expect pass)

2. **Phase 2: Builder API for Config Types**
   - **Objective:** Add fluent builder methods on `LoggerConfig`, `FormatConfig`, `FileConfig`, `OtelConfig`, `MetricsConfig`, `OtlpMetricsConfig`, `SystemMetricsConfig` so consumers avoid verbose `Default` + mutation pattern.
   - **Files/Functions to Modify/Create:**
     - `crates/utils/logger/src/config.rs` — impl blocks with `with_*()` builder methods
     - `crates/utils/logger/src/metrics/config.rs` — same for metrics configs
   - **Tests to Write:**
     - `test_logger_config_builder` — chain builder methods, verify resulting config
     - `test_format_config_builder` — builder produces expected values
     - `test_file_config_builder` — builder with path, max_size
     - `test_otel_config_builder` — builder with endpoint, protocol, sampler
     - `test_metrics_config_builder` — builder for metrics + system config
   - **Steps:**
     1. Write tests using builder pattern API
     2. Run tests (expect fail)
     3. Add `with_*()` methods on each config struct (move-based, returning Self)
     4. Run tests (expect pass)

3. **Phase 3: Version Bump & Documentation**
   - **Objective:** Bump crate version, update CHANGELOG, update lat.md docs, run `lat check`
   - **Files/Functions to Modify:**
     - `crates/utils/logger/Cargo.toml` — version bump to 0.2.0
     - `crates/utils/logger/CHANGELOG.md` — add entry for new features
     - `lat.md/logger.md` — update Output Control and Configuration sections
   - **Steps:**
     1. Bump version to 0.2.0 (new public API surface)
     2. Update CHANGELOG
     3. Update lat.md docs with OutputMode and builder info
     4. Run `lat check`
