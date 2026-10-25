use async_trait::async_trait;
use shared_types::{RunaError, ScreenPoint, WindowToken};

/// Port de capture / restauration de la fenêtre active.
#[async_trait]
pub trait WindowFocus: Send + Sync {
    async fn capture_foreground(&self) -> Result<WindowToken, RunaError>;
    async fn restore(&self, token: &WindowToken) -> Result<(), RunaError>;
    /// Position écran du caret / sélection si accessible ; `None` sinon.
    async fn capture_caret(&self) -> Result<Option<ScreenPoint>, RunaError>;
}
