## Plan: Metrics Collector via Logger Crate

Add a `metrics` feature to the upstream `logger` crate that initializes an OpenTelemetry MeterProvider with Prometheus + OTLP exporters. Logger collects generic system metrics (CPU, memory, network, heap) in a background task. Downstream crates define crate-specific metrics using the OTel `Meter` API and publish through the logger's exporter pipeline.

```
┌─────────────────────────────────────────────────────┐
│  logger crate (feature = "metrics")                 │
│                                                     │
│  setup_metrics(name, MetricsConfig)                 │
│    → MeterProvider (Prometheus + OTLP exporters)    │
│    → Background system metrics task (sysinfo/procfs)│
│    → Global meter via opentelemetry::global::meter()│
│    → MetricsGuard (RAII shutdown)                   │
└──────────────────────┬──────────────────────────────┘
                       │ OTel global meter API
        ┌──────────────┼──────────────────────┐
        ▼              ▼                      ▼
   itch-v1-client  sink-worker   market-data-processor
   (own counters)  (own gauges)  (own histograms)
        │              │                      │
        └──────────────┴──────────────────────┘
                       │
              All metrics flow through
              logger's MeterProvider exporters
                       │
              ┌────────┴────────┐
              ▼                 ▼
        Prometheus          OTLP Collector
        /metrics HTTP       (Grafana/Datadog)
```

---

### Phase 1: Logger crate changes (upstream lib-rs repo)

**Step 1.1** — Add `metrics` feature + dependencies to `logger/Cargo.toml`

- New features: `metrics` (gates OTel metrics SDK, sysinfo, prometheus, tokio), `metrics-procfs` (Linux-only, gates `procfs` for `/proc/self` stats)
- Reuses `opentelemetry 0.31` already in the dependency tree via the `otel` feature

**Step 1.2** — `MetricsConfig` struct (new `logger/src/metrics/config.rs`)

- `PrometheusConfig { enabled, host, port }` — scrape endpoint
- `OtlpConfig { enabled, endpoint, interval_secs }` — push exporter
- `SystemMetricsConfig { enabled, interval_secs }` — background collection interval

**Step 1.3** — MeterProvider initialization (`logger/src/metrics/provider.rs`)

- `pub fn setup_metrics(service_name: &str, config: MetricsConfig) -> Result<MetricsGuard>`
- Builds `SdkMeterProvider` with Prometheus reader + OTLP periodic exporter (both optional)
- Registers as global via `opentelemetry::global::set_meter_provider()`
- Spawns Prometheus HTTP server on `host:port` serving `/metrics`
- Spawns system metrics background task
- Returns `MetricsGuard` — drop shuts down provider + cancels tasks

**Step 1.4** — Generic system metrics collector (`logger/src/metrics/system.rs`)

- Background tokio task, collects every N seconds via `sysinfo`:
  - `process.cpu.utilization` (gauge), `process.memory.usage` (gauge), `process.memory.virtual` (gauge)
  - `system.network.io.transmit` / `.receive` (counters, bytes)
- Linux-only (`metrics-procfs`): `process.memory.rss`, `process.open_file_descriptors`, `process.cpu.context_switches`

**Step 1.5** — Public API (`logger/src/metrics/mod.rs`)

- Exports: `setup_metrics()`, `MetricsConfig`, `MetricsGuard`
- Re-exports `opentelemetry::global::meter` for convenience — downstream crates call `logger::metrics::meter("crate_name")` to create instruments

**Step 1.6** — Tag and release

- Develop on branch `feat/metrics`; workspace uses branch ref during iteration; pin to tag `logger-v0.2.0` when stable

---

### Phase 2: Workspace integration (this repo)

**Step 2.1** — Update root Cargo.toml logger dependency to branch/tag with metrics support

**Step 2.2** — Add `features = ["metrics"]` to each target crate's Cargo.toml:

- market-data-processor/Cargo.toml
- itch-v1-client/Cargo.toml
- itch-v2-client/Cargo.toml
- sink-worker/Cargo.toml
- scheduler-service/Cargo.toml

**Step 2.3** — Add `[metrics]` section to each service's config TOML (Prometheus port, OTLP endpoint, system collection interval)

**Step 2.4** — Initialize metrics in each `main.rs` alongside existing logging:

```rust
let _logging_guard = setup_logging("service", None, config, env_filter)?;
let _metrics_guard = logger::metrics::setup_metrics("service", cfg.metrics)?;
let meter = logger::metrics::meter("service");
```

---

### Phase 3: Crate-specific metrics

**market-data-processor:**

- `mdp.messages.processed` (counter, `{message_type}`), `mdp.messages.processing_latency_ms` (histogram)
- `mdp.channel.depth.realtime` / `.delayed` (gauges), `mdp.orderbooks.active` (gauge), `mdp.orders.active` (gauge)
- `mdp.ranking.compute_duration_ms` (histogram), `mdp.decoder.errors` (counter)

**itch-v1-client:**

- `itch_v1.messages.ingested` (counter, `{message_type}`), `itch_v1.connection.status` (gauge 0/1)
- `itch_v1.connection.reconnects` (counter), `itch_v1.sequence.current` (gauge), `itch_v1.sequence.gaps` (counter)
- `itch_v1.sink.events` (counter, `{sink_type}`), `itch_v1.sink.flush_duration_ms` (histogram)

**itch-v2-client:**

- Same as v1 but with `{channel: itch|mdf}` attribute on all instruments
- `itch_v2.channels.lag_delta` (gauge) — drift between ITCH and MDF sequences

**sink-worker:**

- `sink_worker.events.processed` (counter, `{channel}`), `sink_worker.events.duplicates_skipped` (counter)
- `sink_worker.sink.write_duration_ms` (histogram, `{sink_type}`), `sink_worker.sink.batch_size` (histogram)
- `sink_worker.dedup_set.size` (gauge)

**scheduler-service:**

- `scheduler.job.executions` (counter, `{status: success|failure|skipped}`)
- `scheduler.job.duration_secs` (histogram), `scheduler.job.phase_duration_secs` (histogram, `{phase}`)
- `scheduler.csv.rows_processed` (counter), `scheduler.db.rows_inserted` (counter, `{table}`)

---

### Verification

1. `cargo build --features metrics` passes for all 5 target crates; `cargo build` without feature still compiles
2. Start any service → `curl http://localhost:9090/metrics` → confirm both `process.*` system metrics and crate-specific metrics appear in Prometheus text format
3. `MetricsGuard` drop cleanly shuts down HTTP server + background tasks (no leaked threads)
4. Feature is fully additive — no code changes needed in services that don't opt in

### Decisions

- **Placement**: Logger crate owns infra (MeterProvider, exporters, system metrics); app crates own domain metrics
- **Export path**: All metrics (system + domain) flow through logger's shared MeterProvider
- **Global meter access**: Standard OTel `opentelemetry::global::meter()` — no custom global state
- **System metrics**: `sysinfo` (cross-platform) + `procfs` (Linux-only behind `metrics-procfs`)
- **Excluded from this plan**: Heap/jemalloc metrics (requires per-binary allocator change — follow-up), alerting rules, Grafana dashboards

### Further Considerations

1. **Prometheus port conflicts** — Each service needs a unique port. Suggest convention: MDP=9090, v1=9091, v2=9092, sink=9093, scheduler=9094. Config-driven.
2. **Metric cardinality** — `{message_type}` attributes on ITCH messages could be high-cardinality (~30 types). Acceptable for this scale; monitor if scrape size grows.
3. **jemalloc heap metrics** — For accurate `process.runtime.jemalloc.*` stats, each binary needs `tikv-jemallocator` as global allocator. Plan as a follow-up.
