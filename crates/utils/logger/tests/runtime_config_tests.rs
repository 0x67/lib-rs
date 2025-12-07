#![cfg(all(feature = "file", feature = "stdout"))]

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LoggingSetupError {
    #[error("failed to create log directory")]
    CreateDirectory {
        #[source]
        source: std::io::Error,
    },

    #[error("failed to setup logger")]
    SetupError {
        #[source]
        source: logger::SetupLogging,
    },
}

impl From<logger::SetupLogging> for LoggingSetupError {
    fn from(source: logger::SetupLogging) -> Self {
        Self::SetupError { source }
    }
}

/// Setup logging with proper app data directory
///
/// Behavior based on platform and build mode:
/// - Android/iOS: Logging is skipped (handled by conditional compilation)
/// - Debug mode: Logs to stdout with ANSI colors, file paths, and line numbers
/// - Release mode: Logs to file without ANSI colors, file paths, or line numbers
pub fn setup_logging(app_data_dir: PathBuf) -> Result<logger::LoggingGuard, LoggingSetupError> {
    let log_path = app_data_dir.join("logs");

    // Create logs directory if it doesn't exist
    std::fs::create_dir_all(&log_path)
        .map_err(|source| LoggingSetupError::CreateDirectory { source })?;

    // Configure logging based on build mode
    let mut config = logger::LoggerConfig::default();

    if cfg!(debug_assertions) {
        // Debug mode: stdout only with ANSI colors, file paths, and line numbers
        config.max_level = "TRACE".to_string();
        config.file = None; // No file logging in debug mode

        // Enable stdout with formatting
        let mut format = logger::FormatConfig::default();
        format.ansi = true;
        format.target = true;
        format.file = true;
        format.line_number = true;
        config.format = Some(format);
    } else {
        // Release mode: file logging without ANSI, file paths, or line numbers
        config.max_level = "INFO".to_string();
        config.format = None; // No stdout in release mode

        let mut file_config = logger::FileConfig::default();
        file_config.max_size = 100 * 1024 * 1024; // 100MB
        file_config.path = log_path.to_string_lossy().to_string();
        file_config.enabled = true;

        // Override file format to disable ANSI, file paths, and line numbers
        let mut file_format = logger::FormatConfig::default();
        file_format.ansi = false;
        file_format.target = true;
        file_format.file = false;
        file_format.line_number = false;
        file_config.format = Some(file_format);

        config.file = Some(file_config);
    }

    // Initialize logger and return the guard
    let guard = logger::setup_logging("titan-app", None, config, None)?;

    Ok(guard)
}

#[cfg(test)]
mod tests {
    use super::*;
    use logger::{debug, error, info, trace, warn};

    #[test]
    fn test_debug_mode_setup() {
        let temp_dir = std::env::temp_dir().join("logger_test_debug");
        std::fs::create_dir_all(&temp_dir).unwrap();

        // In debug mode, should use stdout only
        let _guard = setup_logging(temp_dir.clone()).expect("Failed to setup logging");

        // Test that logging works
        info!("Test info message in debug mode");
        debug!("Test debug message");
        trace!("Test trace message");

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_release_mode_setup() {
        let temp_dir = std::env::temp_dir().join("logger_test_release");
        std::fs::create_dir_all(&temp_dir).unwrap();

        // Simulate release mode by manually creating release config
        let log_path = temp_dir.join("logs");
        std::fs::create_dir_all(&log_path).unwrap();

        let mut config = logger::LoggerConfig::default();
        config.max_level = "INFO".to_string();
        config.format = None; // No stdout in release mode

        let mut file_config = logger::FileConfig::default();
        file_config.max_size = 100 * 1024 * 1024;
        file_config.path = log_path.to_string_lossy().to_string();
        file_config.enabled = true;

        let mut file_format = logger::FormatConfig::default();
        file_format.ansi = false;
        file_format.target = true;
        file_format.file = false;
        file_format.line_number = false;
        file_config.format = Some(file_format);

        config.file = Some(file_config);

        let _guard = logger::setup_logging("titan-app", None, config, None)
            .expect("Failed to setup logging in release mode");

        // Test that logging works
        info!("Test info message in release mode");
        warn!("Test warning message");
        error!("Test error message");

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_both_stdout_and_file() {
        let temp_dir = std::env::temp_dir().join("logger_test_both");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let log_path = temp_dir.join("logs");
        std::fs::create_dir_all(&log_path).unwrap();

        let mut config = logger::LoggerConfig::default();
        config.max_level = "DEBUG".to_string();

        // Enable stdout
        let mut format = logger::FormatConfig::default();
        format.ansi = true;
        format.target = true;
        format.file = true;
        format.line_number = true;
        config.format = Some(format);

        // Also enable file
        let mut file_config = logger::FileConfig::default();
        file_config.max_size = 10 * 1024 * 1024;
        file_config.path = log_path.to_string_lossy().to_string();
        file_config.enabled = true;
        config.file = Some(file_config);

        let _guard = logger::setup_logging("titan-app", None, config, None)
            .expect("Failed to setup logging with both stdout and file");

        info!("Test message to both stdout and file");
        debug!("Another test message");

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_file_disabled() {
        let temp_dir = std::env::temp_dir().join("logger_test_file_disabled");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let log_path = temp_dir.join("logs");
        std::fs::create_dir_all(&log_path).unwrap();

        let mut config = logger::LoggerConfig::default();
        config.max_level = "INFO".to_string();

        // Enable stdout
        config.format = Some(logger::FormatConfig::default());

        // Create file config but mark it as disabled
        let mut file_config = logger::FileConfig::default();
        file_config.path = log_path.to_string_lossy().to_string();
        file_config.enabled = false; // Disabled!
        config.file = Some(file_config);

        let _guard = logger::setup_logging("titan-app", None, config, None)
            .expect("Failed to setup logging with disabled file");

        info!("This should only go to stdout");

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_no_stdout_no_file() {
        // This tests the edge case where both are disabled
        let temp_dir = std::env::temp_dir().join("logger_test_nothing");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let mut config = logger::LoggerConfig::default();
        config.max_level = "INFO".to_string();
        config.format = None; // No stdout
        config.file = None; // No file

        let _guard = logger::setup_logging("titan-app", None, config, None)
            .expect("Failed to setup logging with nothing enabled");

        // These logs go nowhere, but shouldn't crash
        info!("This goes nowhere");
        warn!("Neither does this");

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
