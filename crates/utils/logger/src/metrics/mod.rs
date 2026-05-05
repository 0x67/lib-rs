pub mod config;
#[cfg(feature = "jemalloc")]
mod jemalloc;
mod provider;
mod system;

pub use config::*;
pub use provider::{MetricsGuard, SetupMetricsError, SetupMetricsErrorKind, setup_metrics};

pub fn meter(name: &'static str) -> opentelemetry::metrics::Meter {
    opentelemetry::global::meter(name)
}
