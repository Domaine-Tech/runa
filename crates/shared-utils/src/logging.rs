use shared_types::RunaError;
use tracing_subscriber::{fmt, EnvFilter};

/// Initialise `tracing` une seule fois (idempotent via try_init).
pub fn init_tracing(default_level: &str) -> Result<(), RunaError> {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));

    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .try_init()
        .map_err(|e| RunaError::Internal {
            message: format!("tracing init: {e}"),
        })
}
