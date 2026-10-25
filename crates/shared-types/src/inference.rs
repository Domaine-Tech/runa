use serde::{Deserialize, Serialize};
use specta::Type;

use crate::RunaError;

/// Événements de streaming du moteur IA vers l'UI.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InferenceEvent {
    Started { generation_id: u64 },
    Token { generation_id: u64, text: String },
    Done { generation_id: u64, full_text: String },
    Cancelled { generation_id: u64 },
    Error { generation_id: u64, error: RunaError },
}
