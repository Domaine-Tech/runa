use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{Language, Tone, WindowToken, WritingContext};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CorrectRequest {
    pub text: String,
    pub tone: Tone,
    pub context: WritingContext,
    /// Identifiant monotone assigné par le host pour l'annulation.
    pub generation_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TranslateRequest {
    pub text: String,
    pub source: Language,
    pub target: Language,
    pub generation_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ApplyReplacementRequest {
    pub text: String,
    pub origin: WindowToken,
}
