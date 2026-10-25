use serde::{Deserialize, Serialize};
use specta::Type;

use crate::RunaError;

/// Progression du téléchargement explicite du modèle GGUF.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModelDownloadEvent {
    Started { total_bytes: Option<u64> },
    Progress {
        downloaded_bytes: u64,
        total_bytes: Option<u64>,
    },
    Done { path: String },
    Error { error: RunaError },
}
