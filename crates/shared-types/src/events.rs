use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{ScreenPoint, WindowToken};

/// Événement émis lorsque la séquence Ctrl+C+C est détectée.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TriggerEvent {
    pub clipboard_text: String,
    pub origin: WindowToken,
    pub sequence_ms: u64,
    /// Position caret / sélection (écran) si disponible — pour placer la popup.
    pub caret: Option<ScreenPoint>,
}

/// Noms d'événements IPC (source unique pour le front généré).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum IpcEvent {
    TriggerFired,
    Inference,
    ModelDownload,
}
