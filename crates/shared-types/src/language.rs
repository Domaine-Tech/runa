use serde::{Deserialize, Serialize};
use specta::Type;

/// Langues supportées pour la traduction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    Auto,
    Fr,
    En,
    Es,
    De,
}

impl Language {
    pub const fn as_prompt_label(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Fr => "français",
            Self::En => "anglais",
            Self::Es => "espagnol",
            Self::De => "allemand",
        }
    }
}
