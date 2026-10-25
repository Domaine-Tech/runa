use std::sync::{Arc, Mutex};
use std::time::Instant;

use async_trait::async_trait;
use shared_types::{AppConfig, RunaError, TriggerEvent};
use shared_utils::{InstantMillis, SequenceDetector, SequenceOutcome};
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, info, warn};

use crate::clipboard::ClipboardImpulse;
use crate::PlatformStack;

/// Port d'orchestration Ctrl+C+C.
#[async_trait]
pub trait TriggerEngine: Send + Sync {
    async fn start(&self) -> Result<(), RunaError>;
    async fn stop(&self) -> Result<(), RunaError>;
    fn subscribe(&self) -> broadcast::Receiver<TriggerEvent>;
}

pub fn create_trigger_engine(
    stack: Arc<PlatformStack>,
    config: AppConfig,
) -> Arc<dyn TriggerEngine> {
    Arc::new(DefaultTriggerEngine::new(stack, config))
}

struct DefaultTriggerEngine {
    stack: Arc<PlatformStack>,
    config: AppConfig,
    events: broadcast::Sender<TriggerEvent>,
    stop_tx: Mutex<Option<mpsc::Sender<()>>>,
}

impl DefaultTriggerEngine {
    fn new(stack: Arc<PlatformStack>, config: AppConfig) -> Self {
        let (events, _) = broadcast::channel(16);
        Self {
            stack,
            config,
            events,
            stop_tx: Mutex::new(None),
        }
    }
}

#[async_trait]
impl TriggerEngine for DefaultTriggerEngine {
    async fn start(&self) -> Result<(), RunaError> {
        let (impulse_tx, mut impulse_rx) = mpsc::channel::<ClipboardImpulse>(64);
        let (stop_tx, mut stop_rx) = mpsc::channel::<()>(1);

        {
            let mut guard = self.stop_tx.lock().map_err(|_| RunaError::Internal {
                message: "trigger stop mutex empoisonné".into(),
            })?;
            *guard = Some(stop_tx);
        }

        self.stack.clipboard.start(impulse_tx).await?;

        let stack = Arc::clone(&self.stack);
        let events = self.events.clone();
        let window_ms = self.config.trigger.sequence_window_ms;
        let required_hits = self.config.trigger.required_hits;

        tokio::spawn(async move {
            let mut detector = SequenceDetector::new(window_ms, required_hits);
            loop {
                tokio::select! {
                    _ = stop_rx.recv() => {
                        debug!("trigger engine arrêté");
                        break;
                    }
                    impulse = impulse_rx.recv() => {
                        let Some(impulse) = impulse else { break };
                        let fired_sequence_ms = if impulse.immediate {
                            Some(0)
                        } else {
                            match detector.record(InstantMillis::now()) {
                                SequenceOutcome::Armed { hits } => {
                                    tracing::info!(hits, "impulsion Ctrl+C armée");
                                    None
                                }
                                SequenceOutcome::Fired { sequence_ms } => Some(sequence_ms),
                            }
                        };
                        let Some(sequence_ms) = fired_sequence_ms else {
                            continue;
                        };
                        let fired_at = Instant::now();
                        // Focus avant show() — le foreground est encore l'app source.
                        let origin = match stack.focus.capture_foreground().await {
                            Ok(token) => token,
                            Err(err) => {
                                warn!(?err, "capture focus échouée");
                                shared_types::WindowToken::Unknown
                            }
                        };
                        let capture_ms = fired_at.elapsed().as_millis() as u64;
                        // Preview immédiat (sans caret) : le bridge peut show() sans attendre caret/clipboard.
                        let preview = TriggerEvent {
                            clipboard_text: String::new(),
                            origin: origin.clone(),
                            sequence_ms,
                            caret: None,
                        };
                        match events.send(preview) {
                            Ok(n) => info!(
                                sequence_ms,
                                capture_ms,
                                receivers = n,
                                "TriggerFired preview"
                            ),
                            Err(_) => warn!(
                                sequence_ms,
                                capture_ms,
                                "TriggerFired preview sans subscriber"
                            ),
                        }
                        // Caret hors chemin show — le foreground peut encore être l'app source.
                        let caret = match stack.focus.capture_caret().await {
                            Ok(point) => point,
                            Err(err) => {
                                warn!(?err, "capture caret échouée");
                                None
                            }
                        };
                        // Sleep initial + retries hors chemin show (writer OS Empty+Set).
                        let clipboard_text =
                            match read_clipboard_with_retry(stack.clipboard.as_ref()).await {
                                Ok(text) => text,
                                Err(err) => {
                                    warn!(?err, "lecture presse-papiers échouée");
                                    String::new()
                                }
                            };
                        let clipboard_ready_ms = fired_at.elapsed().as_millis() as u64;
                        let text_len = clipboard_text.len();
                        let hydrated = TriggerEvent {
                            clipboard_text,
                            origin,
                            sequence_ms,
                            caret,
                        };
                        match events.send(hydrated) {
                            Ok(n) => info!(
                                sequence_ms,
                                capture_ms,
                                clipboard_ready_ms,
                                text_len,
                                receivers = n,
                                "TriggerFired hydrated"
                            ),
                            Err(_) => warn!(
                                sequence_ms,
                                clipboard_ready_ms,
                                text_len,
                                "TriggerFired hydrated sans subscriber"
                            ),
                        }
                    }
                }
            }
            let _ = stack.clipboard.stop().await;
        });

        Ok(())
    }

    async fn stop(&self) -> Result<(), RunaError> {
        let tx = {
            let mut guard = self.stop_tx.lock().map_err(|_| RunaError::Internal {
                message: "trigger stop mutex empoisonné".into(),
            })?;
            guard.take()
        };
        if let Some(tx) = tx {
            let _ = tx.send(()).await;
        }
        self.stack.clipboard.stop().await
    }

    fn subscribe(&self) -> broadcast::Receiver<TriggerEvent> {
        self.events.subscribe()
    }
}

async fn read_clipboard_with_retry(
    clipboard: &dyn crate::ClipboardWatcher,
) -> Result<String, RunaError> {
    const ATTEMPTS: u32 = 6;
    // Premier délai hors chemin show : laisse le writer OS fermer le clipboard.
    const INITIAL_DELAY_MS: u64 = 40;
    const RETRY_DELAY_MS: u64 = 25;
    let mut last_err: Option<RunaError> = None;
    for attempt in 0..ATTEMPTS {
        if attempt == 0 {
            tokio::time::sleep(std::time::Duration::from_millis(INITIAL_DELAY_MS)).await;
        } else {
            tokio::time::sleep(std::time::Duration::from_millis(RETRY_DELAY_MS)).await;
        }
        match clipboard.read_text().await {
            Ok(text) if !text.is_empty() => return Ok(text),
            Ok(text) => {
                // Vide : soit contenu réellement vide, soit formats pas encore prêts.
                if attempt + 1 == ATTEMPTS {
                    return Ok(text);
                }
            }
            Err(err) => last_err = Some(err),
        }
    }
    Err(last_err.unwrap_or_else(|| RunaError::Clipboard {
        message: "lecture presse-papiers échouée après retries".into(),
    }))
}

#[cfg(test)]
mod mock_tests {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use async_trait::async_trait;
    use shared_types::{AppConfig, ScreenPoint, WindowToken};

    use super::*;
    use crate::clipboard::{ClipboardImpulse, ClipboardWatcher};
    use crate::focus::WindowFocus;
    use crate::injector::TextInjector;
    use crate::PlatformStack;

    #[derive(Default)]
    struct MockShared {
        text: Mutex<String>,
        impulse_tx: Mutex<Option<mpsc::Sender<ClipboardImpulse>>>,
        focus_order: Mutex<Vec<&'static str>>,
    }

    struct MockClipboard {
        shared: Arc<MockShared>,
    }

    #[async_trait]
    impl ClipboardWatcher for MockClipboard {
        async fn start(&self, tx: mpsc::Sender<ClipboardImpulse>) -> Result<(), RunaError> {
            *self.shared.impulse_tx.lock().expect("lock") = Some(tx);
            Ok(())
        }

        async fn stop(&self) -> Result<(), RunaError> {
            *self.shared.impulse_tx.lock().expect("lock") = None;
            Ok(())
        }

        async fn read_text(&self) -> Result<String, RunaError> {
            self.shared
                .focus_order
                .lock()
                .expect("lock")
                .push("read_text");
            Ok(self.shared.text.lock().expect("lock").clone())
        }

        async fn write_text(&self, text: &str) -> Result<(), RunaError> {
            *self.shared.text.lock().expect("lock") = text.to_string();
            Ok(())
        }
    }

    struct MockFocus {
        shared: Arc<MockShared>,
    }

    #[async_trait]
    impl WindowFocus for MockFocus {
        async fn capture_foreground(&self) -> Result<WindowToken, RunaError> {
            self.shared
                .focus_order
                .lock()
                .expect("lock")
                .push("focus");
            Ok(WindowToken::Unknown)
        }

        async fn restore(&self, _token: &WindowToken) -> Result<(), RunaError> {
            Ok(())
        }

        async fn capture_caret(&self) -> Result<Option<ScreenPoint>, RunaError> {
            self.shared
                .focus_order
                .lock()
                .expect("lock")
                .push("caret");
            Ok(Some(ScreenPoint { x: 10, y: 20 }))
        }
    }

    struct MockInjector;

    #[async_trait]
    impl TextInjector for MockInjector {
        async fn paste_clipboard(&self) -> Result<(), RunaError> {
            Ok(())
        }
    }

    fn mock_stack(shared: Arc<MockShared>) -> Arc<PlatformStack> {
        Arc::new(PlatformStack {
            clipboard: Box::new(MockClipboard {
                shared: Arc::clone(&shared),
            }),
            focus: Box::new(MockFocus { shared }),
            injector: Box::new(MockInjector),
        })
    }

    async fn send_impulse(shared: &MockShared) {
        let tx = shared
            .impulse_tx
            .lock()
            .expect("lock")
            .as_ref()
            .expect("impulse channel")
            .clone();
        tx.send(ClipboardImpulse {
            sequence_hint: None,
            immediate: false,
        })
        .await
        .expect("send impulse");
    }

    async fn recv_preview(rx: &mut broadcast::Receiver<TriggerEvent>) -> TriggerEvent {
        let event = tokio::time::timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("timeout preview")
            .expect("recv preview");
        assert!(
            event.clipboard_text.is_empty(),
            "preview doit avoir un texte vide"
        );
        event
    }

    async fn recv_hydrated(rx: &mut broadcast::Receiver<TriggerEvent>) -> TriggerEvent {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            let event = tokio::time::timeout(remaining, rx.recv())
                .await
                .expect("timeout hydrated")
                .expect("recv hydrated");
            if !event.clipboard_text.is_empty() {
                return event;
            }
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn focus_captured_before_trigger_preview_emit() {
        let shared = Arc::new(MockShared::default());
        *shared.text.lock().expect("lock") = "hello".into();
        let engine = create_trigger_engine(mock_stack(Arc::clone(&shared)), AppConfig::default());
        let mut rx = engine.subscribe();
        engine.start().await.expect("start");

        send_impulse(&shared).await;
        send_impulse(&shared).await;

        let _preview = recv_preview(&mut rx).await;
        {
            let order = shared.focus_order.lock().expect("lock").clone();
            assert_eq!(
                order.first().copied(),
                Some("focus"),
                "capture_foreground avant preview ; order={order:?}"
            );
        }

        let hydrated = recv_hydrated(&mut rx).await;
        assert_eq!(hydrated.clipboard_text, "hello");
        let order = shared.focus_order.lock().expect("lock").clone();
        let focus_i = order.iter().position(|s| *s == "focus").expect("focus");
        let caret_i = order.iter().position(|s| *s == "caret").expect("caret");
        let read_i = order.iter().position(|s| *s == "read_text").expect("read");
        assert!(focus_i < caret_i, "focus avant caret ; order={order:?}");
        assert!(caret_i < read_i, "caret avant read_text ; order={order:?}");

        engine.stop().await.expect("stop");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn sequence_fires_on_identical_hash_impulses_via_mock_clipboard() {
        let shared = Arc::new(MockShared::default());
        *shared.text.lock().expect("lock") = "même contenu".into();
        let engine = create_trigger_engine(mock_stack(Arc::clone(&shared)), AppConfig::default());
        let mut rx = engine.subscribe();
        engine.start().await.expect("start");

        send_impulse(&shared).await;
        send_impulse(&shared).await;

        let hydrated = recv_hydrated(&mut rx).await;
        assert_eq!(hydrated.clipboard_text, "même contenu");
        assert!(
            hydrated.sequence_ms <= 400,
            "sequence_ms={} > 400",
            hydrated.sequence_ms
        );
        assert!(hydrated.caret.is_some());

        engine.stop().await.expect("stop");
    }
}

#[cfg(all(test, target_os = "windows"))]
#[allow(unsafe_code)]
mod windows_smoke {
    use std::sync::Arc;
    use std::time::Duration;

    use shared_types::AppConfig;
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
    };
    use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};

    use super::*;
    use crate::create_platform_stack;

    fn set_clipboard_utf16(text: &str) {
        let mut wide: Vec<u16> = text.encode_utf16().collect();
        wide.push(0);
        let bytes = wide.len() * 2;
        unsafe {
            let hmem = GlobalAlloc(GMEM_MOVEABLE, bytes).expect("GlobalAlloc");
            let ptr = GlobalLock(hmem);
            assert!(!ptr.is_null());
            std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr as *mut u16, wide.len());
            let _ = GlobalUnlock(hmem);
            OpenClipboard(None).expect("OpenClipboard");
            let _ = EmptyClipboard();
            SetClipboardData(13, Some(HANDLE(hmem.0))).expect("SetClipboardData");
            CloseClipboard().expect("CloseClipboard");
        }
    }

    #[tokio::test(flavor = "current_thread")]
    #[ignore = "smoke OS — vrai clipboard Windows ; lancer manuellement (cargo test -p platform-engine -- --ignored)"]
    async fn double_clipboard_write_fires_trigger_with_text() {
        let _guard = crate::windows::clipboard_test_lock();
        let config = AppConfig::default();
        let stack = Arc::new(create_platform_stack(&config.trigger).expect("stack"));
        let engine = create_trigger_engine(stack, config);
        let mut rx = engine.subscribe();
        engine.start().await.expect("start trigger");

        tokio::time::sleep(Duration::from_millis(80)).await;
        set_clipboard_utf16("bonjour S1");
        tokio::time::sleep(Duration::from_millis(80)).await;
        set_clipboard_utf16("bonjour S1 encore");

        let event = recv_hydrated_trigger(&mut rx).await;
        assert_eq!(event.clipboard_text, "bonjour S1 encore");
        assert!(event.sequence_ms <= 400);

        engine.stop().await.expect("stop trigger");
    }

    #[tokio::test(flavor = "current_thread")]
    #[ignore = "smoke OS — vrai clipboard Windows ; lancer manuellement (cargo test -p platform-engine -- --ignored)"]
    async fn double_identical_clipboard_write_fires_trigger() {
        let _guard = crate::windows::clipboard_test_lock();
        let config = AppConfig::default();
        let stack = Arc::new(create_platform_stack(&config.trigger).expect("stack"));
        let engine = create_trigger_engine(stack, config);
        let mut rx = engine.subscribe();
        engine.start().await.expect("start trigger");

        tokio::time::sleep(Duration::from_millis(80)).await;
        set_clipboard_utf16("bonjour S1 identique");
        tokio::time::sleep(Duration::from_millis(80)).await;
        set_clipboard_utf16("bonjour S1 identique");

        let event = recv_hydrated_trigger(&mut rx).await;
        assert_eq!(event.clipboard_text, "bonjour S1 identique");
        assert!(event.sequence_ms <= 400);

        engine.stop().await.expect("stop trigger");
    }

    /// Ignore le preview (texte vide) et attend l'événement hydraté.
    async fn recv_hydrated_trigger(
        rx: &mut broadcast::Receiver<TriggerEvent>,
    ) -> TriggerEvent {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            let event = tokio::time::timeout(remaining, rx.recv())
                .await
                .expect("timeout trigger")
                .expect("recv trigger");
            if !event.clipboard_text.is_empty() {
                return event;
            }
        }
    }
}
