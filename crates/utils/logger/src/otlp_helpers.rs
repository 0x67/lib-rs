use crate::{OtelExporterError, OtelExporterErrorKind, config::ProtocolConfig};
use opentelemetry_otlp::{Protocol, WithExportConfig, WithHttpConfig, WithTonicConfig};
use std::convert::TryFrom;

pub(crate) fn build_metric_exporter(
    endpoint: &str,
    protocol: &ProtocolConfig,
    timeout: std::time::Duration,
    headers: Option<&std::collections::HashMap<String, String>>,
    metrics_path: Option<&str>,
) -> Result<opentelemetry_otlp::MetricExporter, OtelExporterError> {
    match protocol {
        ProtocolConfig::Grpc => {
            let mut builder = opentelemetry_otlp::MetricExporter::builder()
                .with_tonic()
                .with_endpoint(endpoint)
                .with_protocol(Protocol::Grpc)
                .with_timeout(timeout);

            if let Some(headers) = headers {
                let mut metadata = tonic::metadata::MetadataMap::new();
                for (key, value) in headers {
                    if let (Ok(key), Ok(value)) = (
                        tonic::metadata::MetadataKey::from_bytes(key.as_bytes()),
                        tonic::metadata::MetadataValue::try_from(value.as_str()),
                    ) {
                        metadata.insert(key, value);
                    }
                }
                builder = builder.with_metadata(metadata);
            }

            builder.build()
        }
        ProtocolConfig::Http => {
            let endpoint = if let Some(path) = metrics_path {
                format!("{}{}", endpoint, path)
            } else {
                endpoint.to_owned()
            };
            let mut builder = opentelemetry_otlp::MetricExporter::builder()
                .with_http()
                .with_endpoint(&endpoint)
                .with_protocol(Protocol::HttpBinary)
                .with_timeout(timeout);

            if let Some(headers) = headers {
                let mut header_map = std::collections::HashMap::new();
                for (key, value) in headers {
                    header_map.insert(key.clone(), value.clone());
                }
                builder = builder.with_headers(header_map);
            }

            builder.build()
        }
    }
    .map_err(|e| {
        OtelExporterError::new(OtelExporterErrorKind::BuildMetricExporter {
            source: Box::new(e),
        })
    })
}
