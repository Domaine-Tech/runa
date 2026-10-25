use std::sync::Arc;

use async_trait::async_trait;
use parking_lot::RwLock;
use shared_types::{AiConfig, CorrectRequest, InferenceEvent, RunaError, TranslateRequest};
use tokio::sync::broadcast;

use crate::worker::{InferenceBackend, Job, LlmWorker, UnsupportedBackend};

#[async_trait]
pub trait LocalAiEngine: Send + Sync {
    async fn correct(&self, req: CorrectRequest) -> Result<(), RunaError>;
    async fn translate(&self, req: TranslateRequest) -> Result<(), RunaError>;
    async fn cancel(&self) -> Result<(), RunaError>;
    fn subscribe(&self) -> broadcast::Receiver<InferenceEvent>;
    /// Inférence en cours (génération active) — utilisé pour ne pas couper le travail (MAJ).
    fn is_busy(&self) -> bool;
}

struct WorkerAiEngine {
    worker: LlmWorker,
}

#[async_trait]
impl LocalAiEngine for WorkerAiEngine {
    async fn correct(&self, req: CorrectRequest) -> Result<(), RunaError> {
        self.worker.enqueue(Job::Correct(req))
    }

    async fn translate(&self, req: TranslateRequest) -> Result<(), RunaError> {
        self.worker.enqueue(Job::Translate(req))
    }

    async fn cancel(&self) -> Result<(), RunaError> {
        self.worker.cancel_current()
    }

    fn subscribe(&self) -> broadcast::Receiver<InferenceEvent> {
        self.worker.subscribe()
    }

    fn is_busy(&self) -> bool {
        self.worker.is_busy()
    }
}

/// Moteur stub (feature `llama` absente).
pub fn create_ai_engine(config: Arc<RwLock<AiConfig>>) -> Arc<dyn LocalAiEngine> {
    let backend: Box<dyn InferenceBackend> = Box::new(UnsupportedBackend);
    Arc::new(WorkerAiEngine {
        worker: LlmWorker::spawn(backend, config),
    })
}

/// Moteur llama.cpp (feature `llama`).
#[cfg(feature = "llama")]
pub fn create_llama_engine(
    config: Arc<RwLock<AiConfig>>,
) -> Result<Arc<dyn LocalAiEngine>, RunaError> {
    let backend = crate::llama::LlamaBackend::try_new()?;
    Ok(Arc::new(WorkerAiEngine {
        worker: LlmWorker::spawn(Box::new(backend), config),
    }))
}
