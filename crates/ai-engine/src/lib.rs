//! Abstraction du moteur IA local et worker de streaming.

pub mod engine;
pub mod prompts;
pub mod worker;

#[cfg(feature = "llama")]
pub mod llama;

pub use engine::{create_ai_engine, LocalAiEngine};
#[cfg(feature = "llama")]
pub use engine::create_llama_engine;
pub use worker::UnsupportedBackend;
