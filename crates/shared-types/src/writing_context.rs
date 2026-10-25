use serde::{Deserialize, Serialize};
use specta::Type;

/// Contexte de rédaction pour la reformulation (axe orthogonal au ton).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "snake_case")]
pub enum WritingContext {
    Mail,
    Chat,
    #[default]
    Note,
}

impl WritingContext {
    pub const fn as_prompt_label(self) -> &'static str {
        match self {
            Self::Mail => "Mail",
            Self::Chat => "Chat",
            Self::Note => "Note",
        }
    }
}
