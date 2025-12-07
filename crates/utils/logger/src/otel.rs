use crate::{
    OpenTelemetryLayer, OtelConfig, OtelExporterError, OtelExporterErrorKind, SetupLogging,
    SetupLoggingKind,
};
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::{
    Resource,
    trace::{RandomIdGenerator, Sampler},
};
pub use time::UtcOffset;
use tracing_subscriber::Registry;

pub fn setup_otel(
    app_name: String,
    otel_config: OtelConfig,
) -> Result<
    (
        OpenTelemetryLayer<Registry, opentelemetry_sdk::trace::Tracer>,
        opentelemetry_sdk::trace::SdkTracerProvider,
        opentelemetry_sdk::logs::SdkLoggerProvider,
        opentelemetry_sdk::metrics::SdkMeterProvider,
    ),
    SetupLogging,
> {
    let otel_endpoint = otel_config.endpoint.clone();
    let timeout = otel_config.timeout();
    let max_queue_size = otel_config.max_queue_size;
    let scheduled_delay = otel_config.scheduled_delay();
    let max_export_batch_size = otel_config.max_export_batch_size;
    let max_events_per_span = otel_config.max_events_per_span;
    let max_attributes_per_span = otel_config.max_attributes_per_span;
    let sampler = otel_config
        .sampler
        .as_ref()
        .map(|s| s.to_sampler())
        .unwrap_or(Sampler::AlwaysOn);

    // Setup trace exporter for spans with timeout
    let trace_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&otel_endpoint)
        .with_protocol(Protocol::Grpc)
        .with_timeout(timeout)
        .build()
        .map_err(|e| {
            SetupLogging::new(SetupLoggingKind::OtelExporter {
                source: OtelExporterError::new(OtelExporterErrorKind::BuildSpanExporter {
                    source: Box::new(e),
                }),
            })
        })?;

    // Create resource with service name
    let resource = Resource::builder()
        .with_service_name(app_name.clone())
        .build();

    // Configure batch span processor with configurable settings
    let batch_config = opentelemetry_sdk::trace::BatchConfigBuilder::default()
        .with_max_queue_size(max_queue_size)
        .with_scheduled_delay(scheduled_delay)
        .with_max_export_batch_size(max_export_batch_size)
        .build();

    let batch_processor =
        opentelemetry_sdk::trace::BatchSpanProcessor::new(trace_exporter, batch_config);

    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_span_processor(batch_processor)
        .with_sampler(sampler)
        .with_id_generator(RandomIdGenerator::default())
        .with_max_events_per_span(max_events_per_span)
        .with_max_attributes_per_span(max_attributes_per_span)
        .with_resource(resource.clone())
        .build();

    let tracer: opentelemetry_sdk::trace::Tracer = tracer_provider.tracer(app_name.clone());
    opentelemetry::global::set_tracer_provider(tracer_provider.clone());

    // Setup log exporter with timeout
    let log_exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .with_endpoint(&otel_endpoint)
        .with_protocol(Protocol::Grpc)
        .with_timeout(timeout)
        .build()
        .map_err(|e| {
            SetupLogging::new(SetupLoggingKind::OtelExporter {
                source: OtelExporterError::new(OtelExporterErrorKind::BuildLogExporter {
                    source: Box::new(e),
                }),
            })
        })?;

    // Configure batch log processor
    let log_batch_config = opentelemetry_sdk::logs::BatchConfigBuilder::default()
        .with_max_queue_size(max_queue_size)
        .with_scheduled_delay(scheduled_delay)
        .with_max_export_batch_size(max_export_batch_size)
        .build();

    let log_batch_processor = opentelemetry_sdk::logs::BatchLogProcessor::builder(log_exporter)
        .with_batch_config(log_batch_config)
        .build();

    let logger_provider = opentelemetry_sdk::logs::SdkLoggerProvider::builder()
        .with_log_processor(log_batch_processor)
        .with_resource(resource.clone())
        .build();

    // Setup metrics exporter with timeout
    let metric_exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(&otel_endpoint)
        .with_protocol(Protocol::Grpc)
        .with_timeout(timeout)
        .build()
        .map_err(|e| {
            SetupLogging::new(SetupLoggingKind::OtelExporter {
                source: OtelExporterError::new(OtelExporterErrorKind::BuildMetricExporter {
                    source: Box::new(e),
                }),
            })
        })?;

    // Configure periodic streams for metrics
    let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_reader(
            opentelemetry_sdk::metrics::PeriodicReader::builder(metric_exporter)
                .with_interval(std::time::Duration::from_secs(60)) // Export every 60 seconds
                .build(),
        )
        .build();

    opentelemetry::global::set_meter_provider(meter_provider.clone());

    // Create the OpenTelemetry layer for automatic span capturing
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    Ok((
        telemetry_layer,
        tracer_provider,
        logger_provider,
        meter_provider,
    ))
}
