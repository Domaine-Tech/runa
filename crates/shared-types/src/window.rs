use serde::{Deserialize, Serialize};
use specta::Type;

/// Jeton opaque de la fenêtre active capturée avant l'ouverture de la popup.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "platform", rename_all = "snake_case")]
pub enum WindowToken {
    Windows { hwnd: u64 },
    Macos { pid: u32 },
    Linux { pid: u32 },
    /// Placeholder lorsque la capture n'est pas encore implémentée.
    Unknown,
}
