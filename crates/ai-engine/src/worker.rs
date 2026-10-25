use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::Arc;
use std::time::Duration;

use parking_lot::RwLock;
use shared_types::{AiConfig, CorrectRequest, InferenceEvent, RunaError, TranslateRequest};
use shared_utils::trim_model_output;
use tokio::sync::broadcast;
use tracing::{info, warn};

use crate::prompts;

#[derive(Debug)]
pub enum Job {
    Correct(CorrectRequest),
    Translate(TranslateRequest),
    Cancel { generation_id: u64 },
}

/// Résultat d'une génération côté backend.
#[derive(Debug)]
pub enum GenerateOutcome {
    Completed(String),
    Cancelled,
}

/// Backend d'inférence — appelé uniquement depuis `llm-worker`.
pub trait InferenceBackend: Send + 'static {
    fn ensure_loaded(&mut self, config: &AiConfig) -> Result<(), RunaError>;
    fn unload(&mut self);
    fn generate(
        &mut self,
        system: &str,
        user: &str,
        temperature: f32,
        max_tokens: u32,
        generation_id: u64,
        active_id: &AtomicU64,
        on_token: &mut dyn FnMut(&str),
    ) -> Result<GenerateOutcome, RunaError>;
}

/// Backend par défaut tant que la feature `llama` est inactive.
#[derive(Debug, Default)]
pub struct UnsupportedBackend;

impl InferenceBackend for UnsupportedBackend {
    fn ensure_loaded(&mut self, _config: &AiConfig) -> Result<(), RunaError> {
        Ok(())
    }

    fn unload(&mut self) {}

    fn generate(
        &mut self,
        _system: &str,
        _user: &str,
        _temperature: f32,
        _max_tokens: u32,
        _generation_id: u64,
        _active_id: &AtomicU64,
        _on_token: &mut dyn FnMut(&str),
    ) -> Result<GenerateOutcome, RunaError> {
        Err(RunaError::unsupported(
            "inférence inactive — relancez avec --features llama (scripts : .\\scripts\\dev.ps1 sans -NoLlama)",
        ))
    }
}

pub struct LlmWorker {
    job_tx: mpsc::Sender<Job>,
    events: broadcast::Sender<InferenceEvent>,
    active_generation: Arc<AtomicU64>,
}

impl LlmWorker {
    pub fn spawn(
        backend: Box<dyn InferenceBackend>,
        config: Arc<RwLock<AiConfig>>,
    ) -> Self {
        let (job_tx, job_rx) = mpsc::channel::<Job>();
        let (events, _) = broadcast::channel(256);
        let active_generation = Arc::new(AtomicU64::new(0));

        let events_thread = events.clone();
        let active = Arc::clone(&active_generation);

        if let Err(err) = std::thread::Builder::new()
            .name("llm-worker".into())
            .spawn(move || worker_loop(job_rx, backend, config, events_thread, active))
        {
            warn!(?err, "échec spawn llm-worker");
        }

        Self {
            job_tx,
            events,
            active_generation,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<InferenceEvent> {
        self.events.subscribe()
    }

    pub fn enqueue(&self, job: Job) -> Result<(), RunaError> {
        match &job {
            Job::Correct(req) => {
                self.active_generation
                    .store(req.generation_id, Ordering::SeqCst);
            }
            Job::Translate(req) => {
                self.active_generation
                    .store(req.generation_id, Ordering::SeqCst);
            }
            Job::Cancel { .. } => {}
        }
        self.job_tx.send(job).map_err(|_| RunaError::Internal {
            message: "llm-worker canal fermé".into(),
        })
    }

    pub fn cancel_current(&self) -> Result<(), RunaError> {
        let id = self.active_generation.swap(0, Ordering::SeqCst);
        self.job_tx
            .send(Job::Cancel { generation_id: id })
            .map_err(|_| RunaError::Internal {
                message: "llm-worker canal fermé".into(),
            })
    }

    #[cfg(test)]
    pub fn active_generation(&self) -> Arc<AtomicU64> {
        Arc::clone(&self.active_generation)
    }
}

fn worker_loop(
    job_rx: mpsc::Receiver<Job>,
    mut backend: Box<dyn InferenceBackend>,
    config: Arc<RwLock<AiConfig>>,
    events: broadcast::Sender<InferenceEvent>,
    active: Arc<AtomicU64>,
) {
    loop {
        let keep_alive = config.read().keep_alive_ms;
        let job = match keep_alive {
            Some(ms) if ms > 0 => match job_rx.recv_timeout(Duration::from_millis(ms)) {
                Ok(job) => job,
                Err(RecvTimeoutError::Timeout) => {
                    info!("keep-alive écoulé — décharge du modèle");
                    backend.unload();
                    match job_rx.recv() {
                        Ok(job) => job,
                        Err(_) => break,
                    }
                }
                Err(RecvTimeoutError::Disconnected) => break,
            },
            _ => match job_rx.recv() {
                Ok(job) => job,
                Err(_) => break,
            },
        };

        match job {
            Job::Cancel { generation_id } => {
                active.store(0, Ordering::SeqCst);
                let _ = events.send(InferenceEvent::Cancelled { generation_id });
            }
            Job::Correct(req) => {
                run_job(
                    &mut backend,
                    &config,
                    &events,
                    &active,
                    req.generation_id,
                    prompts::correction_system(req.tone, req.context),
                    req.text,
                    true,
                );
            }
            Job::Translate(req) => {
                run_job(
                    &mut backend,
                    &config,
                    &events,
                    &active,
                    req.generation_id,
                    prompts::translation_system(req.source, req.target),
                    req.text,
                    false,
                );
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn run_job(
    backend: &mut Box<dyn InferenceBackend>,
    config: &Arc<RwLock<AiConfig>>,
    events: &broadcast::Sender<InferenceEvent>,
    active: &Arc<AtomicU64>,
    generation_id: u64,
    system: String,
    user: String,
    is_correct: bool,
) {
    if active.load(Ordering::SeqCst) != generation_id {
        let _ = events.send(InferenceEvent::Cancelled { generation_id });
        return;
    }

    let _ = events.send(InferenceEvent::Started { generation_id });

    let (temperature, max_tokens, snap) = {
        let cfg = config.read();
        let temperature = if is_correct {
            cfg.temperature_correct
        } else {
            cfg.temperature_translate
        };
        (temperature, cfg.max_tokens, cfg.clone())
    };

    if let Err(err) = backend.ensure_loaded(&snap) {
        warn!(?err, "chargement modèle échoué");
        let _ = events.send(InferenceEvent::Error {
            generation_id,
            error: err,
        });
        return;
    }

    let mut full = String::new();
    let outcome = backend.generate(
        &system,
        &user,
        temperature,
        max_tokens,
        generation_id,
        active,
        &mut |token| {
            full.push_str(token);
            let _ = events.send(InferenceEvent::Token {
                generation_id,
                text: token.to_string(),
            });
        },
    );

    match outcome {
        Ok(GenerateOutcome::Completed(text)) => {
            let trimmed = trim_model_output(&text);
            let _ = events.send(InferenceEvent::Done {
                generation_id,
                full_text: trimmed,
            });
        }
        Ok(GenerateOutcome::Cancelled) => {
            let _ = events.send(InferenceEvent::Cancelled { generation_id });
        }
        Err(err) => {
            warn!(?err, "génération échouée");
            let _ = events.send(InferenceEvent::Error {
                generation_id,
                error: err,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    struct FakeBackend {
        delay_ms: u64,
    }

    impl InferenceBackend for FakeBackend {
        fn ensure_loaded(&mut self, _config: &AiConfig) -> Result<(), RunaError> {
            Ok(())
        }

        fn unload(&mut self) {}

        fn generate(
            &mut self,
            _system: &str,
            _user: &str,
            _temperature: f32,
            _max_tokens: u32,
            generation_id: u64,
            active_id: &AtomicU64,
            on_token: &mut dyn FnMut(&str),
        ) -> Result<GenerateOutcome, RunaError> {
            for i in 0..3 {
                if active_id.load(Ordering::SeqCst) != generation_id {
                    return Ok(GenerateOutcome::Cancelled);
                }
                on_token(&format!("t{i}"));
                thread::sleep(Duration::from_millis(self.delay_ms));
            }
            Ok(GenerateOutcome::Completed("t0t1t2".into()))
        }
    }

    #[test]
    fn cancel_stops_before_third_token() {
        let config = Arc::new(RwLock::new(AiConfig {
            model_path: String::new(),
            n_ctx: 512,
            n_gpu_layers: 0,
            keep_alive_ms: None,
            max_tokens: 32,
            temperature_correct: 0.3,
            temperature_translate: 0.2,
        }));
        let worker = LlmWorker::spawn(
            Box::new(FakeBackend { delay_ms: 40 }),
            Arc::clone(&config),
        );
        let mut rx = worker.subscribe();

        worker
            .enqueue(Job::Correct(CorrectRequest {
                text: "hi".into(),
                tone: shared_types::Tone::Fluid,
                context: shared_types::WritingContext::Note,
                generation_id: 7,
            }))
            .expect("enqueue");

        // Attendre Started + premier token
        let mut saw_token = false;
        for _ in 0..20 {
            match rx.try_recv() {
                Ok(InferenceEvent::Token { .. }) => {
                    saw_token = true;
                    break;
                }
                Ok(_) => {}
                Err(_) => thread::sleep(Duration::from_millis(5)),
            }
        }
        assert!(saw_token, "attendu au moins un token");

        worker.cancel_current().expect("cancel");

        let mut cancelled = false;
        let mut third = false;
        let deadline = std::time::Instant::now() + Duration::from_millis(500);
        while std::time::Instant::now() < deadline {
            match rx.try_recv() {
                Ok(InferenceEvent::Cancelled { generation_id }) => {
                    assert_eq!(generation_id, 7);
                    cancelled = true;
                    break;
                }
                Ok(InferenceEvent::Token { text, .. }) if text == "t2" => {
                    third = true;
                }
                Ok(InferenceEvent::Done { .. }) => {
                    panic!("Done inattendu après cancel");
                }
                _ => thread::sleep(Duration::from_millis(5)),
            }
        }
        assert!(cancelled, "Cancelled attendu");
        assert!(!third, "le 3e token ne doit pas être émis");
    }

    #[test]
    fn cancel_before_started_emits_cancelled() {
        let config = Arc::new(RwLock::new(AiConfig {
            model_path: String::new(),
            n_ctx: 512,
            n_gpu_layers: 0,
            keep_alive_ms: None,
            max_tokens: 32,
            temperature_correct: 0.3,
            temperature_translate: 0.2,
        }));
        // Délai long : si le job démarre quand même, on annule avant le 1er token.
        let worker = LlmWorker::spawn(
            Box::new(FakeBackend { delay_ms: 200 }),
            Arc::clone(&config),
        );
        let mut rx = worker.subscribe();

        worker
            .enqueue(Job::Correct(CorrectRequest {
                text: "hi".into(),
                tone: shared_types::Tone::Fluid,
                context: shared_types::WritingContext::Note,
                generation_id: 42,
            }))
            .expect("enqueue");
        // Invalide active_generation avant que run_job ne passe le check.
        worker.cancel_current().expect("cancel");

        let mut cancelled = false;
        let mut saw_token = false;
        let deadline = std::time::Instant::now() + Duration::from_millis(500);
        while std::time::Instant::now() < deadline {
            match rx.try_recv() {
                Ok(InferenceEvent::Cancelled { generation_id }) => {
                    assert!(generation_id == 42 || generation_id == 0);
                    cancelled = true;
                    break;
                }
                Ok(InferenceEvent::Token { .. }) => {
                    saw_token = true;
                }
                Ok(InferenceEvent::Done { .. }) => {
                    panic!("Done inattendu");
                }
                _ => thread::sleep(Duration::from_millis(5)),
            }
        }
        assert!(cancelled, "Cancelled attendu");
        assert!(!saw_token, "aucun token avant Started/cancel");
    }

    #[test]
    fn unsupported_backend_emits_error_event() {
        let config = Arc::new(RwLock::new(AiConfig {
            model_path: String::new(),
            n_ctx: 512,
            n_gpu_layers: 0,
            keep_alive_ms: None,
            max_tokens: 32,
            temperature_correct: 0.3,
            temperature_translate: 0.2,
        }));
        let worker = LlmWorker::spawn(Box::new(UnsupportedBackend), Arc::clone(&config));
        let mut rx = worker.subscribe();

        worker
            .enqueue(Job::Correct(CorrectRequest {
                text: "hi".into(),
                tone: shared_types::Tone::Fluid,
                context: shared_types::WritingContext::Note,
                generation_id: 9,
            }))
            .expect("enqueue");

        let mut saw_error = false;
        let deadline = std::time::Instant::now() + Duration::from_millis(500);
        while std::time::Instant::now() < deadline {
            match rx.try_recv() {
                Ok(InferenceEvent::Error {
                    generation_id,
                    error,
                }) => {
                    assert_eq!(generation_id, 9);
                    assert!(matches!(error, RunaError::Unsupported { .. }));
                    saw_error = true;
                    break;
                }
                Ok(InferenceEvent::Done { .. }) => panic!("Done inattendu"),
                _ => thread::sleep(Duration::from_millis(5)),
            }
        }
        assert!(saw_error, "Error Unsupported attendu");
    }
}
