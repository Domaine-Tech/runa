//! Source unique de vérité pour les types Runa (domaine, DTO, IPC, erreurs).

pub mod config;
pub mod diff;
pub mod error;
pub mod events;
pub mod geometry;
pub mod inference;
pub mod language;
pub mod model_download;
pub mod requests;
pub mod tone;
pub mod window;
pub mod writing_context;

pub use config::{
    AccentPreset, AiConfig, AppConfig, ThemeMode, TriggerConfig, UiConfig, UiDensity,
};
pub use diff::{DiffKind, DiffSpan};
pub use error::RunaError;
pub use events::{IpcEvent, TriggerEvent};
pub use geometry::ScreenPoint;
pub use inference::InferenceEvent;
pub use language::Language;
pub use model_download::ModelDownloadEvent;
pub use requests::{ApplyReplacementRequest, CorrectRequest, TranslateRequest};
pub use tone::Tone;
pub use window::WindowToken;
pub use writing_context::WritingContext;
