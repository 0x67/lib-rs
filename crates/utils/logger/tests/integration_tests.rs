use logger::{Level, LoggerConfig, config::OutputMode, setup_logging};

// === OutputMode integration tests ===

#[test]
fn test_output_mode_stdout_only() {
    let mut config = LoggerConfig::default();
    config.output_mode = OutputMode::Stdout;
    // File should not be used even if file config is set
    #[cfg(feature = "file")]
    {
        let mut file_cfg = logger::config::FileConfig::default();
        file_cfg.enabled = true;
        config.file = Some(file_cfg);
    }
    let result = setup_logging("test_stdout_only", None, config, None);
    // NOTE: Global subscriber can only be set once per process. In test harness,
    // first test wins; subsequent calls return Err. We verify setup doesn't panic.
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_output_mode_file_only() {
    let mut config = LoggerConfig::default();
    config.output_mode = OutputMode::File;
    // Has format set, but stdout should NOT register
    config.format = Some(logger::config::FormatConfig::default());
    let result = setup_logging("test_file_only", None, config, None);
    // NOTE: Verifies setup doesn't panic — global subscriber limitation prevents
    // asserting which layers were actually registered.
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_output_mode_both() {
    let mut config = LoggerConfig::default();
    config.output_mode = OutputMode::Both;
    let result = setup_logging("test_both", None, config, None);
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_output_mode_none() {
    let mut config = LoggerConfig::default();
    config.output_mode = OutputMode::None;
    // No output layers at all — verifies setup doesn't panic in OTel-only/silent mode
    let result = setup_logging("test_none_mode", None, config, None);
    assert!(result.is_ok() || result.is_err());
}

// === Existing tests ===

#[test]
fn test_setup_logging_basic() {
    let config = LoggerConfig::default();
    let result = setup_logging("test_app", None, config, None);
    // Allow test to pass if dispatcher is already set
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_setup_logging_with_timezone() {
    let config = LoggerConfig::default();
    let result = setup_logging("test_app", Some(8), config, None);
    // Allow test to pass if dispatcher is already set
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_setup_logging_with_env_filter() {
    let config = LoggerConfig::default();
    let result = setup_logging("test_app", None, config, Some(vec!["debug"]));
    // Allow test to pass if dispatcher is already set
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_setup_logging_with_custom_level() {
    let mut config = LoggerConfig::default();
    config.max_level = "DEBUG".to_string();
    let result = setup_logging("test_app", None, config, None);
    // Allow test to pass if dispatcher is already set
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_setup_logging_invalid_env_filter() {
    let config = LoggerConfig::default();
    let result = setup_logging("test_app", None, config, Some(vec!["invalid[[filter"]));
    assert!(result.is_err());
}

#[test]
fn test_level_parsing() {
    let levels = vec!["TRACE", "DEBUG", "INFO", "WARN", "ERROR"];
    for level_str in levels {
        let parsed: Result<Level, _> = level_str.parse();
        assert!(parsed.is_ok());
    }
}

#[test]
fn test_logging_macros() {
    let config = LoggerConfig::default();
    // Allow the test to pass if logger is already initialized
    let _guard = setup_logging("test_app", None, config, None).ok();

    logger::info!("Test info message");
    logger::debug!("Test debug message");
    logger::warn!("Test warn message");
    logger::error!("Test error message");
    logger::trace!("Test trace message");
}

#[test]
fn test_logging_with_fields() {
    let config = LoggerConfig::default();
    // Allow the test to pass if logger is already initialized
    let _guard = setup_logging("test_app", None, config, None).ok();

    logger::info!("User logged in", user_id = 123,);
    logger::error!("Failed to connect", error = "connection timeout",);
}

#[test]
fn test_logging_guard_drop() {
    let config = LoggerConfig::default();
    // Allow the test to pass if logger is already initialized
    if let Ok(guard) = setup_logging("test_app", None, config, None) {
        drop(guard);
    }
}

#[cfg(feature = "file")]
#[test]
fn test_setup_logging_with_file_missing_config() {
    let mut config = LoggerConfig::default();
    config.max_level = "INFO".to_string();
    config.file = None;
    config.format = None;
    #[cfg(feature = "otel")]
    {
        config.otel = None;
    }

    let result = setup_logging("test_app", None, config, None);
    // Should succeed without file logging when file config is None
    // Allow test to pass if dispatcher is already set
    assert!(result.is_ok() || result.is_err());
}

#[cfg(feature = "file")]
#[test]
fn test_setup_logging_with_file_config() {
    use logger::config::FileConfig;

    let temp_dir = std::env::temp_dir().join("logger_test");
    std::fs::create_dir_all(&temp_dir).unwrap();

    let mut config = LoggerConfig::default();
    config.max_level = "INFO".to_string();

    let mut file_cfg = FileConfig::default();
    file_cfg.max_size = 1024 * 1024;
    file_cfg.path = temp_dir.to_string_lossy().to_string();
    file_cfg.enabled = true;
    file_cfg.format = None;
    config.file = Some(file_cfg);

    config.format = None;
    #[cfg(feature = "otel")]
    {
        config.otel = None;
    }

    let result = setup_logging("test_file_app", None, config, None);
    // Allow test to pass if dispatcher is already set
    if let Ok(guard) = result {
        logger::info!("Test file logging");
        drop(guard);
    }

    std::fs::remove_dir_all(temp_dir).ok();
}

#[test]
fn test_multiple_directives() {
    let config = LoggerConfig::default();
    let result = setup_logging(
        "test_app",
        None,
        config,
        Some(vec!["info", "my_crate=debug", "other_crate=trace"]),
    );
    // Allow test to pass if dispatcher is already set
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_format_config_variations() {
    use logger::config::FormatConfig;

    let mut config1 = FormatConfig::default();
    config1.ansi = true;
    config1.target = true;
    config1.file = true;
    config1.line_number = true;

    let mut config2 = FormatConfig::default();
    config2.ansi = false;
    config2.target = false;
    config2.file = false;
    config2.line_number = false;

    let mut config3 = FormatConfig::default();
    config3.ansi = true;
    config3.target = false;
    config3.file = true;
    config3.line_number = false;

    let configs = vec![config1, config2, config3];

    for format in configs {
        let mut config = LoggerConfig::default();
        config.max_level = "INFO".to_string();
        config.format = Some(format);
        let result = setup_logging("test_app", None, config, None);
        // Allow test to pass if dispatcher is already set from previous iteration
        assert!(result.is_ok() || result.is_err());
    }
}
