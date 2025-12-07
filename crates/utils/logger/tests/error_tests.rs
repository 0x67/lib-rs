use logger::{SetupLogging, SetupLoggingKind};

#[test]
fn test_setup_logging_error_display() {
    let error = SetupLogging::invalid_env_filter(
        "invalid[[filter",
        "invalid[[filter"
            .parse::<tracing_subscriber::filter::Directive>()
            .unwrap_err(),
    );

    let display = format!("{}", error);
    assert!(display.contains("failed to setup logging"));

    let debug = format!("{:?}", error);
    assert!(debug.contains("SetupLogging"));
}

#[test]
fn test_setup_logging_error_source() {
    let error = SetupLogging::invalid_env_filter(
        "invalid[[filter",
        "invalid[[filter"
            .parse::<tracing_subscriber::filter::Directive>()
            .unwrap_err(),
    );

    use std::error::Error;
    assert!(error.source().is_some());
}

#[test]
fn test_invalid_env_filter_error() {
    let error = SetupLogging::invalid_env_filter(
        "invalid[bracket",
        "invalid[bracket"
            .parse::<tracing_subscriber::filter::Directive>()
            .unwrap_err(),
    );

    match error.kind {
        SetupLoggingKind::InvalidEnvFilter { directive, .. } => {
            assert_eq!(directive, "invalid[bracket");
        }
        _ => panic!("Expected InvalidEnvFilter error kind"),
    }
}

#[test]
fn test_missing_config_error() {
    let error = SetupLogging::missing_config("otel");

    match error.kind {
        SetupLoggingKind::MissingConfig { config_type, .. } => {
            assert_eq!(config_type, "otel");
        }
        _ => panic!("Expected MissingConfig error kind"),
    }

    let display = format!("{}", error.kind);
    assert!(display.contains("missing otel configuration"));
}

#[cfg(feature = "sysinfo")]
#[test]
fn test_sysinfo_error_display() {
    use logger::SysInfoError;

    let error = SysInfoError::get_pid("test error");
    let display = format!("{}", error);
    assert!(display.contains("failed to collect system information"));

    let kind_display = format!("{}", error.kind);
    assert!(kind_display.contains("failed to get process id"));
}

#[cfg(feature = "sysinfo")]
#[test]
fn test_sysinfo_error_kinds() {
    use logger::{SysInfoError, SysInfoErrorKind};

    let get_pid_err = SysInfoError::get_pid("pid error");
    match get_pid_err.kind {
        SysInfoErrorKind::GetPid { .. } => {}
        _ => panic!("Expected GetPid error kind"),
    }

    let not_found_err = SysInfoError::process_not_found(12345);
    match not_found_err.kind {
        SysInfoErrorKind::ProcessNotFound { pid, .. } => {
            assert_eq!(pid, 12345);
        }
        _ => panic!("Expected ProcessNotFound error kind"),
    }

    let display = format!("{}", not_found_err.kind);
    assert!(display.contains("process 12345 not found"));
}

#[test]
fn test_error_chain() {
    use std::error::Error;

    let parse_err = "invalid[[filter"
        .parse::<tracing_subscriber::filter::Directive>()
        .unwrap_err();
    let error = SetupLogging::invalid_env_filter("invalid[[filter", parse_err);

    let mut source = error.source();
    let mut depth = 0;
    while let Some(err) = source {
        depth += 1;
        source = err.source();
        if depth > 10 {
            break;
        }
    }

    assert!(depth > 0, "Error should have a source chain");
}

#[test]
fn test_setup_logging_error_non_exhaustive() {
    let error = SetupLogging::missing_config("test");

    // This should compile, demonstrating that the struct is accessible
    let _kind = &error.kind;
}
