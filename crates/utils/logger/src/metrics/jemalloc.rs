use opentelemetry::metrics::Meter;

/// Collect jemalloc heap statistics and record them as OTel gauge values.
///
/// Must call `tikv_jemalloc_ctl::epoch::advance()` before reading stats
/// to ensure fresh values.
pub(crate) fn record_jemalloc_metrics(meter: &Meter) -> JemallocInstruments {
    let allocated = meter
        .u64_gauge("process.runtime.jemalloc.memory.allocated")
        .with_description("Total bytes allocated by the application via jemalloc")
        .with_unit("By")
        .build();

    let resident = meter
        .u64_gauge("process.runtime.jemalloc.memory.resident")
        .with_description("Total bytes in physically resident pages mapped by jemalloc")
        .with_unit("By")
        .build();

    let mapped = meter
        .u64_gauge("process.runtime.jemalloc.memory.mapped")
        .with_description("Total bytes in active extents mapped by jemalloc")
        .with_unit("By")
        .build();

    let retained = meter
        .u64_gauge("process.runtime.jemalloc.memory.retained")
        .with_description(
            "Total bytes in virtual memory mappings retained by jemalloc (not returned to OS)",
        )
        .with_unit("By")
        .build();

    let metadata = meter
        .u64_gauge("process.runtime.jemalloc.memory.metadata")
        .with_description("Total bytes used by jemalloc for internal bookkeeping")
        .with_unit("By")
        .build();

    JemallocInstruments {
        allocated,
        resident,
        mapped,
        retained,
        metadata,
    }
}

pub(crate) struct JemallocInstruments {
    allocated: opentelemetry::metrics::Gauge<u64>,
    resident: opentelemetry::metrics::Gauge<u64>,
    mapped: opentelemetry::metrics::Gauge<u64>,
    retained: opentelemetry::metrics::Gauge<u64>,
    metadata: opentelemetry::metrics::Gauge<u64>,
}

impl JemallocInstruments {
    /// Advance the jemalloc epoch and record current stats.
    pub(crate) fn collect(&self) {
        // Advance epoch to get a consistent snapshot of stats.
        if tikv_jemalloc_ctl::epoch::advance().is_err() {
            return;
        }

        if let Ok(v) = tikv_jemalloc_ctl::stats::allocated::read() {
            self.allocated.record(v as u64, &[]);
        }
        if let Ok(v) = tikv_jemalloc_ctl::stats::resident::read() {
            self.resident.record(v as u64, &[]);
        }
        if let Ok(v) = tikv_jemalloc_ctl::stats::mapped::read() {
            self.mapped.record(v as u64, &[]);
        }
        if let Ok(v) = tikv_jemalloc_ctl::stats::retained::read() {
            self.retained.record(v as u64, &[]);
        }
        if let Ok(v) = tikv_jemalloc_ctl::stats::metadata::read() {
            self.metadata.record(v as u64, &[]);
        }
    }
}
