use super::config::MetricsConfig;
use crate::{OtelExporterError, otlp_helpers::build_metric_exporter};
use opentelemetry_sdk::{Resource, metrics::SdkMeterProvider};
use tokio::task::JoinHandle;

#[derive(Debug, thiserror::Error)]
#[error("failed to setup metrics")]
#[non_exhaustive]
pub struct SetupMetricsError {
    #[source]
    pub kind: SetupMetricsErrorKind,
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SetupMetricsErrorKind {
    #[error("failed to build metric exporter")]
    #[non_exhaustive]
    BuildMetricExporter {
        #[source]
        source: OtelExporterError,
    },
}

impl SetupMetricsError {
    pub fn new(kind: SetupMetricsErrorKind) -> Self {
        Self { kind }
    }
}

pub struct MetricsGuard {
    meter_provider: SdkMeterProvider,
    system_handle: Option<JoinHandle<()>>,
}

impl Drop for MetricsGuard {
    fn drop(&mut self) {
        if let Some(handle) = self.system_handle.take() {
            handle.abort();
        }

        if let Err(e) = self.meter_provider.force_flush() {
            eprintln!("Failed to force flush meter provider: {e}");
        }

        if let Err(e) = self.meter_provider.shutdown() {
            eprintln!("Failed to shutdown meter provider: {e}");
        }
    }
}

pub fn setup_metrics(
    service_name: &'static str,
    config: MetricsConfig,
) -> Result<MetricsGuard, SetupMetricsError> {
    let otlp = &config.otlp;

    let metric_exporter = build_metric_exporter(
        &otlp.endpoint,
        &otlp.protocol,
        otlp.timeout(),
        otlp.headers.as_ref(),
        otlp.metrics_path.as_deref(),
    )
    .map_err(|e| {
        SetupMetricsError::new(SetupMetricsErrorKind::BuildMetricExporter { source: e })
    })?;

    let mut resource_builder = Resource::builder().with_service_name(service_name);

    if let Some(attributes) = &otlp.attributes {
        for (key, value) in attributes {
            resource_builder = resource_builder
                .with_attribute(opentelemetry::KeyValue::new(key.clone(), value.clone()));
        }
    }

    let resource = resource_builder.build();

    let meter_provider = SdkMeterProvider::builder()
        .with_reader(
            opentelemetry_sdk::metrics::PeriodicReader::builder(metric_exporter)
                .with_interval(otlp.interval())
                .build(),
        )
        .with_resource(resource)
        .build();

    opentelemetry::global::set_meter_provider(meter_provider.clone());

    let system_handle = if config.system.enabled {
        let meter = opentelemetry::global::meter(service_name);
        Some(super::system::spawn_system_metrics(meter, config.system))
    } else {
        None
    };

    Ok(MetricsGuard {
        meter_provider,
        system_handle,
    })
}
