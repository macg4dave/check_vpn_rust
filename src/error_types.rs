use std::process::ExitCode;
use thiserror::Error;
use anyhow::Error as AnyhowError;
use crate::config::validation::ValidationErrors;

/// Application-level error kinds. These are used at boundaries so the binary
/// can map errors to meaningful exit codes and messages.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("configuration validation error: {0}")]
    ConfigError(#[from] ValidationErrors),
    #[error("configuration load error: {0}")]
    ConfigLoadError(#[source] AnyhowError),
    #[error("runtime error: {0}")]
    RuntimeError(#[source] AnyhowError),
    #[error("action failed: {0}")]
    ActionFailed(#[source] AnyhowError),
    #[error("argument error: {0}")]
    ArgsError(#[source] AnyhowError),
    #[error("other error: {0}")]
    Other(#[source] AnyhowError),
}

/// High level exit reasons which are mapped to specific numeric `ExitCode`s.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitReason {
    Success,
    ConfigError,
    RuntimeError,
    ActionFailed,
    ArgsError,
    Other,
}

impl From<&AppError> for ExitReason {
    fn from(err: &AppError) -> Self {
        match err {
            AppError::ConfigError(_) | AppError::ConfigLoadError(_) => ExitReason::ConfigError,
            AppError::RuntimeError(_) => ExitReason::RuntimeError,
            AppError::ActionFailed(_) => ExitReason::ActionFailed,
            AppError::ArgsError(_) => ExitReason::ArgsError,
            AppError::Other(_) => ExitReason::Other,
        }
    }
}

/// Map `ExitReason` into a platform `ExitCode`. We try to keep parity with
/// the existing `config` constants where appropriate (e.g., invalid config = 2)
pub fn exit_code_for_reason(r: ExitReason) -> ExitCode {
    match r {
        ExitReason::Success => ExitCode::SUCCESS,
        ExitReason::ConfigError => ExitCode::from(crate::config::EXIT_INVALID_CONFIG as u8),
        ExitReason::RuntimeError => ExitCode::from(1),
        ExitReason::ActionFailed => ExitCode::from(3),
        ExitReason::ArgsError => ExitCode::from(4),
        ExitReason::Other => ExitCode::from(1),
    }
}

/// Map an `AppError` directly into an `ExitCode`.
pub fn exit_code_for_error(e: &AppError) -> ExitCode {
    exit_code_for_reason(ExitReason::from(e))
}
