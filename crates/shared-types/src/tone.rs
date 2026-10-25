use serde::{Deserialize, Serialize};
use specta::Type;

/// Tons de reformulation pour l'onglet Correction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "snake_case")]
pub enum Tone {
    Formal,
    Concise,
    #[default]
    Fluid,
}

impl Tone {
    pub const fn as_prompt_label(self) -> &'static str {
        match self {
            Self::Formal => "Formel",
            Self::Concise => "Concis",
            Self::Fluid => "Fluide",
        }
    }
}
