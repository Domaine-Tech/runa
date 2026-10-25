//! Utilitaires purs et partagés — aucune logique métier OS/UI ici.

pub mod config;
pub mod hash;
pub mod language_detect;
pub mod logging;
pub mod model;
pub mod sequence;
pub mod text_diff;
pub mod text_trim;
pub mod time;

pub use config::{config_path, load_config, save_config};
pub use hash::{blake3_file, blake3_hex};
pub use language_detect::{detect_language, resolve_translate_target};
pub use logging::init_tracing;
pub use model::{
    default_model_path, models_dir, DEFAULT_MODEL_BLAKE3, DEFAULT_MODEL_FILENAME, DEFAULT_MODEL_URL,
};
pub use sequence::{SequenceDetector, SequenceOutcome};
pub use text_diff::word_diff;
pub use text_trim::trim_model_output;
pub use time::{elapsed_ms, should_emit_impulse, InstantMillis};
