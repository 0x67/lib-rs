use logger::config::*;

// === OutputMode tests ===

#[test]
fn test_output_mode_default() {
    assert_eq!(OutputMode::default(), OutputMode::Both);
}

#[test]
fn test_output_mode_serde() {
    let variants = vec![
        (OutputMode::Stdout, "\"stdout\""),
        (OutputMode::File, "\"file\""),
        (OutputMode::Both, "\"both\""),
        (OutputMode::None, "\"none\""),
    ];

    for (variant, expected_json) in variants {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(json, expected_json, "serialize {variant:?}");
        let deserialized: OutputMode = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, variant, "roundtrip {variant:?}");
    }
}

#[test]
fn test_output_mode_backward_compat() {
    // Config JSON without output_mode field should deserialize with default (Both)
    let json = r#"{"max_level":"INFO"}"#;
    let config: LoggerConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.output_mode, OutputMode::Both);
}

#[test]
fn test_output_mode_enables_stdout() {
    assert!(OutputMode::Stdout.enables_stdout());
    assert!(OutputMode::Both.enables_stdout());
    assert!(!OutputMode::File.enables_stdout());
    assert!(!OutputMode::None.enables_stdout());
}

#[test]
fn test_output_mode_enables_file() {
    assert!(OutputMode::File.enables_file());
    assert!(OutputMode::Both.enables_file());
    assert!(!OutputMode::Stdout.enables_file());
    assert!(!OutputMode::None.enables_file());
}

// === Existing tests ===

#[test]
fn test_logger_config_default() {
    let config = LoggerConfig::default();
    assert_eq!(config.max_level, "INFO");
    assert!(config.format.is_some());

    let format = config.format.unwrap();
    assert!(format.ansi);
    assert!(format.target);
    assert!(format.file);
    assert!(format.line_number);
}

#[test]
fn test_format_config_default() {
    let format = FormatConfig::default();
    assert!(format.ansi);
    assert!(format.target);
    assert!(format.file);
    assert!(format.line_number);
}

#[test]
fn test_format_config_custom() {
    let mut format = FormatConfig::default();
    format.ansi = false;
    format.target = false;
    format.file = true;
    format.line_number = false;

    assert!(!format.ansi);
    assert!(!format.target);
    assert!(format.file);
    assert!(!format.line_number);
}

#[cfg(feature = "file")]
#[test]
fn test_file_config_default() {
    let config = FileConfig::default();
    assert_eq!(config.max_size, 100 * 1024 * 1024);
    assert_eq!(config.path, "./logs");
    assert!(!config.enabled);
    assert!(config.format.is_none());
}

#[cfg(feature = "file")]
#[test]
fn test_file_config_custom() {
    let mut config = FileConfig::default();
    config.max_size = 50 * 1024 * 1024;
    config.path = "/var/log/myapp".to_string();
    config.enabled = true;

    let mut format = FormatConfig::default();
    format.ansi = false;
    format.target = true;
    format.file = false;
    format.line_number = true;
    config.format = Some(format);

    assert_eq!(config.max_size, 50 * 1024 * 1024);
    assert_eq!(config.path, "/var/log/myapp");
    assert!(config.enabled);
    assert!(config.format.is_some());
}

#[cfg(feature = "otel")]
#[test]
fn test_otel_config_default() {
    let config = OtelConfig::default();
    assert_eq!(config.endpoint, "http://localhost:4317");
    assert!(!config.enabled);
    assert_eq!(config.timeout_secs, 3);
    assert_eq!(config.max_queue_size, 65536);
    assert_eq!(config.scheduled_delay_ms, 200);
    assert_eq!(config.max_export_batch_size, 512);
    assert_eq!(config.max_events_per_span, 64);
    assert_eq!(config.max_attributes_per_span, 16);
    assert!(config.sampler.is_some());
}

#[cfg(feature = "otel")]
#[test]
fn test_otel_config_timeout() {
    let config = OtelConfig::default();
    assert_eq!(config.timeout(), std::time::Duration::from_secs(3));

    let mut custom = OtelConfig::default();
    custom.timeout_secs = 10;
    assert_eq!(custom.timeout(), std::time::Duration::from_secs(10));
}

#[cfg(feature = "otel")]
#[test]
fn test_otel_config_scheduled_delay() {
    let config = OtelConfig::default();
    assert_eq!(
        config.scheduled_delay(),
        std::time::Duration::from_millis(200)
    );

    let mut custom = OtelConfig::default();
    custom.scheduled_delay_ms = 500;
    assert_eq!(
        custom.scheduled_delay(),
        std::time::Duration::from_millis(500)
    );
}

#[cfg(feature = "otel")]
#[test]
fn test_sampler_config_default() {
    let sampler = SamplerConfig::default();
    match sampler {
        SamplerConfig::TraceIdRatioBased { ratio } => {
            assert_eq!(ratio, 1.0);
        }
        _ => panic!("Expected TraceIdRatioBased sampler"),
    }
}

#[cfg(feature = "otel")]
#[test]
fn test_sampler_config_to_sampler() {
    use opentelemetry_sdk::trace::Sampler;

    let always_on = SamplerConfig::AlwaysOn;
    matches!(always_on.to_sampler(), Sampler::AlwaysOn);

    let always_off = SamplerConfig::AlwaysOff;
    matches!(always_off.to_sampler(), Sampler::AlwaysOff);

    let ratio = SamplerConfig::TraceIdRatioBased { ratio: 0.5 };
    matches!(ratio.to_sampler(), Sampler::TraceIdRatioBased(_));
}

#[cfg(feature = "otel")]
#[test]
fn test_sampler_config_parent_based() {
    let parent_based = SamplerConfig::ParentBased {
        root: Box::new(SamplerConfig::AlwaysOn),
    };

    match parent_based {
        SamplerConfig::ParentBased { root } => {
            matches!(*root, SamplerConfig::AlwaysOn);
        }
        _ => panic!("Expected ParentBased sampler"),
    }
}

#[test]
fn test_logger_config_serde() {
    let mut config = LoggerConfig::default();
    config.max_level = "DEBUG".to_string();

    #[cfg(feature = "file")]
    {
        let mut file_cfg = FileConfig::default();
        file_cfg.max_size = 1024;
        file_cfg.path = "/tmp/logs".to_string();
        file_cfg.enabled = true;
        file_cfg.format = None;
        config.file = Some(file_cfg);
    }

    let mut format = FormatConfig::default();
    format.ansi = false;
    format.target = true;
    format.file = false;
    format.line_number = true;
    config.format = Some(format);

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: LoggerConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.max_level, "DEBUG");
    assert!(deserialized.format.is_some());

    #[cfg(feature = "file")]
    {
        assert!(deserialized.file.is_some());
        let file_cfg = deserialized.file.unwrap();
        assert_eq!(file_cfg.max_size, 1024);
        assert_eq!(file_cfg.path, "/tmp/logs");
        assert!(file_cfg.enabled);
    }
}

// === Builder API tests ===

#[test]
fn test_logger_config_builder() {
    let config = LoggerConfig::default()
        .with_max_level("DEBUG")
        .with_output_mode(OutputMode::File);
    assert_eq!(config.max_level, "DEBUG");
    assert_eq!(config.output_mode, OutputMode::File);
}

#[test]
fn test_logger_config_builder_with_format() {
    let config = LoggerConfig::default().with_format(FormatConfig::default().with_ansi(false));
    let fmt = config.format.unwrap();
    assert!(!fmt.ansi);
}

#[cfg(feature = "file")]
#[test]
fn test_logger_config_builder_with_file() {
    let config = LoggerConfig::default().with_file(FileConfig::default().with_enabled(true));
    assert!(config.file.unwrap().enabled);
}

#[cfg(feature = "otel")]
#[test]
fn test_logger_config_builder_with_otel() {
    let config = LoggerConfig::default().with_otel(OtelConfig::default().with_enabled(true));
    assert!(config.otel.unwrap().enabled);
}

#[test]
fn test_format_config_builder() {
    let config = FormatConfig::default()
        .with_ansi(false)
        .with_target(false)
        .with_line_number(false);
    assert!(!config.ansi);
    assert!(!config.target);
    assert!(!config.line_number);
}

#[test]
fn test_format_config_builder_all_fields() {
    let config = FormatConfig::default()
        .with_ansi(false)
        .with_target(false)
        .with_file(false)
        .with_line_number(false)
        .with_span_events(false);
    assert!(!config.ansi);
    assert!(!config.target);
    assert!(!config.file);
    assert!(!config.line_number);
    assert!(!config.with_span_events);
}

#[cfg(feature = "file")]
#[test]
fn test_file_config_builder() {
    let config = FileConfig::default()
        .with_path("/var/log/app")
        .with_max_size(50 * 1024 * 1024)
        .with_enabled(true);
    assert_eq!(config.path, "/var/log/app");
    assert_eq!(config.max_size, 50 * 1024 * 1024);
    assert!(config.enabled);
}

#[cfg(feature = "file")]
#[test]
fn test_file_config_builder_with_format() {
    let config = FileConfig::default().with_format(FormatConfig::default().with_ansi(false));
    let fmt = config.format.unwrap();
    assert!(!fmt.ansi);
}

#[cfg(feature = "otel")]
#[test]
fn test_otel_config_builder() {
    let config = OtelConfig::default()
        .with_endpoint("http://otel:4317")
        .with_protocol(ProtocolConfig::Http)
        .with_enabled(true)
        .with_timeout_secs(5);
    assert_eq!(config.endpoint, "http://otel:4317");
    assert!(matches!(config.protocol, ProtocolConfig::Http));
    assert!(config.enabled);
    assert_eq!(config.timeout_secs, 5);
}

#[cfg(feature = "otel")]
#[test]
fn test_otel_config_builder_with_sampler() {
    let config = OtelConfig::default().with_sampler(SamplerConfig::AlwaysOn);
    match config.sampler.unwrap() {
        SamplerConfig::AlwaysOn => {}
        other => panic!("Expected AlwaysOn, got {other:?}"),
    }
}

#[cfg(feature = "metrics")]
#[test]
fn test_metrics_config_builder() {
    use logger::metrics::*;

    let config = MetricsConfig::default()
        .with_otlp(
            OtlpMetricsConfig::default()
                .with_enabled(true)
                .with_endpoint("http://otel:4317")
                .with_interval_secs(30),
        )
        .with_system(SystemMetricsConfig::default().with_enabled(false));
    assert!(config.otlp.enabled);
    assert_eq!(config.otlp.endpoint, "http://otel:4317");
    assert_eq!(config.otlp.interval_secs, 30);
    assert!(!config.system.enabled);
}

#[cfg(feature = "metrics")]
#[test]
fn test_otlp_metrics_config_builder() {
    use logger::metrics::*;

    let config = OtlpMetricsConfig::default()
        .with_enabled(true)
        .with_endpoint("http://collector:4318")
        .with_protocol(ProtocolConfig::Http)
        .with_interval_secs(45)
        .with_timeout_secs(10);
    assert!(config.enabled);
    assert_eq!(config.endpoint, "http://collector:4318");
    assert_eq!(config.interval_secs, 45);
    assert_eq!(config.timeout_secs, 10);
}

#[cfg(feature = "metrics")]
#[test]
fn test_system_metrics_config_builder() {
    use logger::metrics::*;

    let config = SystemMetricsConfig::default()
        .with_enabled(false)
        .with_interval_secs(30);
    assert!(!config.enabled);
    assert_eq!(config.interval_secs, 30);
}
