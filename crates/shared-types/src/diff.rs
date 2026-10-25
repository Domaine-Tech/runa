use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum DiffKind {
    Equal,
    Insert,
    Delete,
}

/// Segment de diff mot-à-mot pour l'affichage UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct DiffSpan {
    pub kind: DiffKind,
    pub text: String,
}
