//! Adapter macOS (stubs — implémentation native dans une passe suivante).

use async_trait::async_trait;
use shared_types::{RunaError, ScreenPoint, WindowToken};
use tokio::sync::mpsc;

use crate::clipboard::{ClipboardImpulse, ClipboardWatcher};
use crate::focus::WindowFocus;
use crate::injector::TextInjector;

#[derive(Debug, Default)]
pub struct MacosClipboard;

#[async_trait]
impl ClipboardWatcher for MacosClipboard {
    async fn start(&self, _tx: mpsc::Sender<ClipboardImpulse>) -> Result<(), RunaError> {
        Err(RunaError::unsupported(
            "macOS NSPasteboard changeCount polling non implémenté",
        ))
    }

    async fn stop(&self) -> Result<(), RunaError> {
        Ok(())
    }

    async fn read_text(&self) -> Result<String, RunaError> {
        Err(RunaError::unsupported("macOS clipboard read non implémenté"))
    }

    async fn write_text(&self, _text: &str) -> Result<(), RunaError> {
        Err(RunaError::unsupported("macOS clipboard write non implémenté"))
    }
}

#[derive(Debug, Default)]
pub struct MacosFocus;

#[async_trait]
impl WindowFocus for MacosFocus {
    async fn capture_foreground(&self) -> Result<WindowToken, RunaError> {
        Err(RunaError::unsupported("macOS AX focus capture non implémenté"))
    }

    async fn restore(&self, _token: &WindowToken) -> Result<(), RunaError> {
        Err(RunaError::unsupported("macOS focus restore non implémenté"))
    }

    async fn capture_caret(&self) -> Result<Option<ScreenPoint>, RunaError> {
        Ok(None)
    }
}

#[derive(Debug, Default)]
pub struct MacosInjector;

#[async_trait]
impl TextInjector for MacosInjector {
    async fn paste_clipboard(&self) -> Result<(), RunaError> {
        Err(RunaError::unsupported("CGEvent Cmd+V non implémenté"))
    }
}
