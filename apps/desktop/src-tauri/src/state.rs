use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use ai_engine::LocalAiEngine;
use parking_lot::RwLock as SyncRwLock;
use platform_engine::{PlatformStack, TriggerEngine};
use shared_types::{AiConfig, AppConfig};
use tokio::sync::RwLock;

use crate::model_download::ModelDownloadHub;

pub struct AppState {
    pub config: RwLock<AppConfig>,
    /// Miroir sync de `config.ai` pour le llm-worker.
    pub ai_config: Arc<SyncRwLock<AiConfig>>,
    pub stack: Arc<PlatformStack>,
    pub trigger: Arc<dyn TriggerEngine>,
    pub ai: Arc<dyn LocalAiEngine>,
    pub model_download: ModelDownloadHub,
    pub generation: AtomicU64,
    pub last_origin: RwLock<shared_types::WindowToken>,
    /// Ignore les `Focused(false)` jusqu'à cet instant (ms epoch) après un show.
    ignore_blur_until_ms: AtomicU64,
}

impl AppState {
    pub fn new(
        config: AppConfig,
        ai_config: Arc<SyncRwLock<AiConfig>>,
        stack: Arc<PlatformStack>,
        trigger: Arc<dyn TriggerEngine>,
        ai: Arc<dyn LocalAiEngine>,
        model_download: ModelDownloadHub,
    ) -> Self {
        Self {
            config: RwLock::new(config),
            ai_config,
            stack,
            trigger,
            ai,
            model_download,
            generation: AtomicU64::new(1),
            last_origin: RwLock::new(shared_types::WindowToken::Unknown),
            ignore_blur_until_ms: AtomicU64::new(0),
        }
    }

    pub fn next_generation(&self) -> u64 {
        self.generation.fetch_add(1, Ordering::SeqCst)
    }

    pub fn mark_popup_shown(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        self.ignore_blur_until_ms
            .store(now.saturating_add(250), Ordering::SeqCst);
    }

    pub fn should_hide_on_blur(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        now >= self.ignore_blur_until_ms.load(Ordering::SeqCst)
    }

    /// Persiste `ai` depuis le miroir sync vers le TOML.
    pub async fn persist_ai_from_mirror(&self) -> Result<(), shared_types::RunaError> {
        let ai = self.ai_config.read().clone();
        let mut config = self.config.write().await;
        config.ai = ai;
        shared_utils::save_config(&config)
    }
}
