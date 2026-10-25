//! Adapter Linux Wayland / X11 — chaîne data-control → XFixes → evdev → portail.

mod evdev;
mod portal;
mod probe;
mod session;
mod wayland;
mod x11;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::Sender as StdSender;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use shared_types::{RunaError, ScreenPoint, TriggerConfig, WindowToken};
use tokio::sync::mpsc;
use tracing::{info, warn};

use crate::clipboard::{ClipboardImpulse, ClipboardWatcher};
use crate::focus::WindowFocus;
use crate::injector::TextInjector;
use crate::linux_select::{
    select_clipboard_backend, ClipboardBackend, ProbeInputs, SelectFailure,
};

const SUPPRESS_WRITE_MS: u64 = 100;

#[derive(Clone)]
enum ClipCmdTx {
    Wayland(StdSender<wayland::ClipCmd>),
    X11(StdSender<x11::ClipCmd>),
}

struct ClipboardRuntime {
    cmd: Option<ClipCmdTx>,
    clip_join: Option<JoinHandle<()>>,
    evdev_stop: Option<Arc<std::sync::atomic::AtomicBool>>,
    evdev_join: Option<JoinHandle<()>>,
    portal_stop: Option<Arc<std::sync::atomic::AtomicBool>>,
}

pub struct LinuxClipboard {
    evdev_opt_in: bool,
    portal_shortcut: String,
    suppress_until_ms: Arc<AtomicU64>,
    runtime: Mutex<Option<ClipboardRuntime>>,
    starting: AtomicU64, // 0/1 as bool to avoid AtomicBool + extra
}

impl LinuxClipboard {
    pub fn new(trigger: &TriggerConfig) -> Self {
        Self {
            evdev_opt_in: trigger.linux_evdev_opt_in,
            portal_shortcut: trigger.linux_portal_shortcut.clone(),
            suppress_until_ms: Arc::new(AtomicU64::new(0)),
            runtime: Mutex::new(None),
            starting: AtomicU64::new(0),
        }
    }

    fn arm_suppress(&self) {
        let until = now_epoch_ms().saturating_add(SUPPRESS_WRITE_MS);
        self.suppress_until_ms.store(until, Ordering::SeqCst);
    }
}

impl std::fmt::Debug for LinuxClipboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LinuxClipboard")
            .field("evdev_opt_in", &self.evdev_opt_in)
            .finish_non_exhaustive()
    }
}

fn now_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[async_trait]
impl ClipboardWatcher for LinuxClipboard {
    async fn start(&self, tx: mpsc::Sender<ClipboardImpulse>) -> Result<(), RunaError> {
        if self
            .starting
            .compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Err(RunaError::Clipboard {
                message: "listener Linux déjà en cours de démarrage".into(),
            });
        }
        let result = self.start_inner(tx).await;
        self.starting.store(0, Ordering::SeqCst);
        result
    }

    async fn stop(&self) -> Result<(), RunaError> {
        let runtime = {
            let mut guard = self.runtime.lock().map_err(|_| RunaError::Internal {
                message: "mutex clipboard Linux empoisonné".into(),
            })?;
            guard.take()
        };
        let Some(runtime) = runtime else {
            return Ok(());
        };

        if let Some(stop) = runtime.portal_stop {
            portal::stop_portal(&stop);
        }
        if let Some(stop) = &runtime.evdev_stop {
            if let Some(join) = runtime.evdev_join {
                evdev::stop_listener(stop, join);
            }
        }
        match (runtime.cmd, runtime.clip_join) {
            (Some(ClipCmdTx::Wayland(tx)), Some(join)) => {
                wayland::stop_watcher(tx, join);
            }
            (Some(ClipCmdTx::X11(tx)), Some(join)) => {
                x11::stop_watcher(tx, join);
            }
            (_, Some(join)) => {
                let _ = join.join();
            }
            _ => {}
        }
        Ok(())
    }

    async fn read_text(&self) -> Result<String, RunaError> {
        let cmd = {
            let guard = self.runtime.lock().map_err(|_| RunaError::Internal {
                message: "mutex clipboard Linux empoisonné".into(),
            })?;
            guard.as_ref().and_then(|r| match &r.cmd {
                Some(ClipCmdTx::Wayland(tx)) => Some(ClipCmdTx::Wayland(tx.clone())),
                Some(ClipCmdTx::X11(tx)) => Some(ClipCmdTx::X11(tx.clone())),
                None => None,
            })
        };
        match cmd {
            Some(ClipCmdTx::Wayland(tx)) => {
                tokio::task::spawn_blocking(move || wayland::cmd_read(&tx))
                    .await
                    .map_err(|e| RunaError::Internal {
                        message: format!("lecture wayland: {e}"),
                    })?
            }
            Some(ClipCmdTx::X11(tx)) => {
                tokio::task::spawn_blocking(move || x11::cmd_read(&tx))
                    .await
                    .map_err(|e| RunaError::Internal {
                        message: format!("lecture x11: {e}"),
                    })?
            }
            None => Err(RunaError::Clipboard {
                message: "pas de backend clipboard pour la lecture (evdev seul / non démarré)"
                    .into(),
            }),
        }
    }

    async fn write_text(&self, text: &str) -> Result<(), RunaError> {
        self.arm_suppress();
        let owned = text.to_owned();
        let cmd = {
            let guard = self.runtime.lock().map_err(|_| RunaError::Internal {
                message: "mutex clipboard Linux empoisonné".into(),
            })?;
            guard.as_ref().and_then(|r| match &r.cmd {
                Some(ClipCmdTx::Wayland(tx)) => Some(ClipCmdTx::Wayland(tx.clone())),
                Some(ClipCmdTx::X11(tx)) => Some(ClipCmdTx::X11(tx.clone())),
                None => None,
            })
        };
        match cmd {
            Some(ClipCmdTx::Wayland(tx)) => {
                tokio::task::spawn_blocking(move || wayland::cmd_write(&tx, owned))
                    .await
                    .map_err(|e| RunaError::Internal {
                        message: format!("écriture wayland: {e}"),
                    })?
            }
            Some(ClipCmdTx::X11(tx)) => {
                tokio::task::spawn_blocking(move || x11::cmd_write(&tx, owned))
                    .await
                    .map_err(|e| RunaError::Internal {
                        message: format!("écriture x11: {e}"),
                    })?
            }
            None => Err(RunaError::Clipboard {
                message: "pas de backend clipboard pour l'écriture".into(),
            }),
        }
    }
}

impl LinuxClipboard {
    async fn start_inner(&self, tx: mpsc::Sender<ClipboardImpulse>) -> Result<(), RunaError> {
        {
            let guard = self.runtime.lock().map_err(|_| RunaError::Internal {
                message: "mutex clipboard Linux empoisonné".into(),
            })?;
            if guard.is_some() {
                return Err(RunaError::Clipboard {
                    message: "listener Linux déjà démarré".into(),
                });
            }
        }

        let session = session::detect_session();
        let (ext, wlr) = probe::probe_data_control();
        let inputs = ProbeInputs {
            session,
            ext_data_control: ext,
            wlr_data_control: wlr,
            linux_evdev_opt_in: self.evdev_opt_in,
        };
        let backend_result = select_clipboard_backend(inputs);
        match &backend_result {
            Err(SelectFailure::Unsupported) => {
                return Err(RunaError::unsupported(
                    "session Linux sans affichage ni fallback clipboard",
                ));
            }
            Err(SelectFailure::NeedEvdevOptIn) => {
                info!(
                    ?session,
                    "aucun data-control / X11 — tentative portail GlobalShortcuts"
                );
            }
            Ok(backend) => {
                info!(?backend, ?session, "backend clipboard Linux sélectionné");
            }
        }

        let suppress = Arc::clone(&self.suppress_until_ms);
        let mut runtime = ClipboardRuntime {
            cmd: None,
            clip_join: None,
            evdev_stop: None,
            evdev_join: None,
            portal_stop: None,
        };
        let mut clipboard_ok = false;

        match backend_result {
            Ok(
                backend @ (ClipboardBackend::ExtDataControl | ClipboardBackend::WlrDataControl),
            ) => match wayland::spawn_watcher(backend, tx.clone(), suppress) {
                Ok((cmd, join)) => {
                    runtime.cmd = Some(ClipCmdTx::Wayland(cmd));
                    runtime.clip_join = Some(join);
                    clipboard_ok = true;
                }
                Err(err) => {
                    warn!(?err, "watcher Wayland data-control non démarré");
                }
            },
            Ok(ClipboardBackend::XFixes) => match x11::spawn_watcher(tx.clone(), suppress) {
                Ok((cmd, join)) => {
                    runtime.cmd = Some(ClipCmdTx::X11(cmd));
                    runtime.clip_join = Some(join);
                    clipboard_ok = true;
                }
                Err(err) => {
                    warn!(?err, "watcher XFixes non démarré");
                }
            },
            Ok(ClipboardBackend::Evdev) => match evdev::spawn_listener(tx.clone()) {
                Ok((stop, join)) => {
                    runtime.evdev_stop = Some(stop);
                    runtime.evdev_join = Some(join);
                    clipboard_ok = true;
                }
                Err(err) => {
                    warn!(
                        ?err,
                        "evdev non démarré — portail GlobalShortcuts peut prendre le relais"
                    );
                }
            },
            Err(SelectFailure::NeedEvdevOptIn) => {}
            Err(SelectFailure::Unsupported) => {}
        }

        let portal_ok = match portal::spawn_portal_listener(&self.portal_shortcut, tx).await {
            Ok(stop) => {
                runtime.portal_stop = Some(stop);
                true
            }
            Err(err @ RunaError::Config { .. }) => {
                warn!(?err, "portail GlobalShortcuts ignoré (raccourci invalide)");
                false
            }
            Err(err) => {
                warn!(?err, "portail GlobalShortcuts non démarré");
                false
            }
        };

        if !clipboard_ok && !portal_ok {
            return Err(RunaError::PlatformPermission {
                message: "aucun data-control / X11 : activez `linux_evdev_opt_in` (groupe input) ou le portail GlobalShortcuts".into(),
            });
        }

        let mut guard = self.runtime.lock().map_err(|_| RunaError::Internal {
            message: "mutex clipboard Linux empoisonné".into(),
        })?;
        *guard = Some(runtime);
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct LinuxFocus;

impl LinuxFocus {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl WindowFocus for LinuxFocus {
    async fn capture_foreground(&self) -> Result<WindowToken, RunaError> {
        if !session::has_x11_display() {
            return Ok(WindowToken::Unknown);
        }
        tokio::task::spawn_blocking(x11::capture_foreground_pid)
            .await
            .map_err(|e| RunaError::Internal {
                message: format!("capture focus: {e}"),
            })?
    }

    async fn restore(&self, token: &WindowToken) -> Result<(), RunaError> {
        if !session::can_xtest_inject() {
            return Ok(());
        }
        let token = token.clone();
        tokio::task::spawn_blocking(move || x11::restore_focus(&token))
            .await
            .map_err(|e| RunaError::Internal {
                message: format!("restore focus: {e}"),
            })?
    }

    async fn capture_caret(&self) -> Result<Option<ScreenPoint>, RunaError> {
        Ok(None)
    }
}

#[derive(Debug, Default)]
pub struct LinuxInjector;

impl LinuxInjector {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl TextInjector for LinuxInjector {
    async fn paste_clipboard(&self) -> Result<(), RunaError> {
        if !session::can_xtest_inject() {
            return Err(RunaError::InjectionFallback {
                message: "session Wayland : collage manuel (Ctrl+V) — texte déjà dans le presse-papiers".into(),
            });
        }
        tokio::task::spawn_blocking(x11::paste_ctrl_v)
            .await
            .map_err(|e| RunaError::Internal {
                message: format!("injection: {e}"),
            })?
    }
}
