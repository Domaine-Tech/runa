//! Listener presse-papiers Windows : fenêtre message-only + `AddClipboardFormatListener`.
#![allow(unsafe_code)]

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::JoinHandle;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use shared_types::RunaError;
use shared_utils::should_emit_impulse;
use tokio::sync::mpsc;
use tracing::{debug, warn};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{GetLastError, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::DataExchange::{
    AddClipboardFormatListener, CloseClipboard, EmptyClipboard, GetClipboardData,
    GetClipboardSequenceNumber, OpenClipboard, RemoveClipboardFormatListener, SetClipboardData,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
    GetWindowLongPtrW, PostMessageW, PostQuitMessage, RegisterClassExW, SetWindowLongPtrW,
    TranslateMessage, CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, GWLP_USERDATA, HWND_MESSAGE, MSG,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_CLIPBOARDUPDATE, WM_CREATE, WM_DESTROY, WM_NCCREATE,
    WM_USER, WNDCLASSEXW,
};

use crate::clipboard::{ClipboardImpulse, ClipboardWatcher};

/// Formats clipboard Win32 (`u32` attendu par `GetClipboardData`).
const CF_TEXT: u32 = 1;
const CF_UNICODETEXT: u32 = 13;

/// Fenêtre de coalescence pour double-notif d'un même `CloseClipboard` (ms).
const COALESCE_MS: u64 = 40;

/// Durée pendant laquelle les impulsions issues d'écritures Runa sont ignorées (ms).
const SUPPRESS_WRITE_MS: u64 = 100;

/// Message custom pour arrêter la pompe proprement.
const WM_RUNA_STOP: u32 = WM_USER + 42;

/// Classe fenêtre enregistrée une seule fois par process.
static CLASS_REGISTERED: OnceLock<Result<(), u32>> = OnceLock::new();

struct ListenerState {
    impulse_tx: mpsc::Sender<ClipboardImpulse>,
    /// `true` dès qu'au moins une impulsion a été émise.
    has_emitted: bool,
    last_at: Instant,
    /// Deadline epoch-ms : ignorer les impulsions jusqu'à cette date (écritures internes).
    suppress_until_ms: Arc<AtomicU64>,
}

/// `hwnd` stocké en `isize` pour rester `Send`/`Sync` (HWND raw n'est pas Send).
struct ClipboardRuntime {
    hwnd: isize,
    join: JoinHandle<()>,
}

/// Adapter clipboard Windows 11.
pub struct WindowsClipboard {
    runtime: Mutex<Option<ClipboardRuntime>>,
    starting: AtomicBool,
    suppress_until_ms: Arc<AtomicU64>,
}

impl Default for WindowsClipboard {
    fn default() -> Self {
        Self {
            runtime: Mutex::new(None),
            starting: AtomicBool::new(false),
            suppress_until_ms: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl std::fmt::Debug for WindowsClipboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowsClipboard").finish_non_exhaustive()
    }
}

#[async_trait]
impl ClipboardWatcher for WindowsClipboard {
    async fn start(&self, tx: mpsc::Sender<ClipboardImpulse>) -> Result<(), RunaError> {
        if self.starting.swap(true, Ordering::SeqCst) {
            return Err(RunaError::Clipboard {
                message: "listener clipboard déjà en cours de démarrage".into(),
            });
        }

        let result = self.start_inner(tx).await;
        self.starting.store(false, Ordering::SeqCst);
        result
    }

    async fn stop(&self) -> Result<(), RunaError> {
        let runtime = {
            let mut guard = self.runtime.lock().map_err(|_| RunaError::Internal {
                message: "mutex clipboard empoisonné".into(),
            })?;
            guard.take()
        };

        let Some(runtime) = runtime else {
            return Ok(());
        };

        if runtime.hwnd != 0 {
            let hwnd = HWND(runtime.hwnd as *mut std::ffi::c_void);
            let posted = unsafe { PostMessageW(Some(hwnd), WM_RUNA_STOP, WPARAM(0), LPARAM(0)) };
            if let Err(err) = posted {
                warn!(?err, "PostMessage stop clipboard échoué");
            }
        }

        tokio::task::spawn_blocking(move || {
            if let Err(err) = runtime.join.join() {
                warn!(?err, "join thread win-clipboard échoué");
            }
        })
        .await
        .map_err(|e| RunaError::Internal {
            message: format!("join clipboard: {e}"),
        })?;

        debug!("listener clipboard Windows arrêté");
        Ok(())
    }

    async fn read_text(&self) -> Result<String, RunaError> {
        tokio::task::spawn_blocking(read_clipboard_text)
            .await
            .map_err(|e| RunaError::Internal {
                message: format!("lecture clipboard: {e}"),
            })?
    }

    async fn write_text(&self, text: &str) -> Result<(), RunaError> {
        self.arm_suppress();
        let owned = text.to_owned();
        tokio::task::spawn_blocking(move || write_clipboard_utf16(&owned))
            .await
            .map_err(|e| RunaError::Internal {
                message: format!("écriture clipboard: {e}"),
            })?
    }
}

impl WindowsClipboard {
    fn arm_suppress(&self) {
        let until = now_epoch_ms().saturating_add(SUPPRESS_WRITE_MS);
        self.suppress_until_ms.store(until, Ordering::SeqCst);
    }
}

impl WindowsClipboard {
    async fn start_inner(&self, tx: mpsc::Sender<ClipboardImpulse>) -> Result<(), RunaError> {
        {
            let guard = self.runtime.lock().map_err(|_| RunaError::Internal {
                message: "mutex clipboard empoisonné".into(),
            })?;
            if guard.is_some() {
                return Err(RunaError::Clipboard {
                    message: "listener clipboard déjà démarré".into(),
                });
            }
        }

        let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<isize, RunaError>>();
        let suppress = Arc::clone(&self.suppress_until_ms);

        let join = std::thread::Builder::new()
            .name("win-clipboard".into())
            .spawn(move || clipboard_thread_main(tx, ready_tx, suppress))
            .map_err(|e| RunaError::Clipboard {
                message: format!("spawn win-clipboard: {e}"),
            })?;

        let hwnd = tokio::task::spawn_blocking(move || {
            ready_rx
                .recv_timeout(Duration::from_secs(5))
                .map_err(|_| RunaError::Clipboard {
                    message: "timeout démarrage listener clipboard".into(),
                })?
        })
        .await
        .map_err(|e| RunaError::Internal {
            message: format!("attente ready clipboard: {e}"),
        })??;

        let mut guard = self.runtime.lock().map_err(|_| RunaError::Internal {
            message: "mutex clipboard empoisonné".into(),
        })?;
        *guard = Some(ClipboardRuntime { hwnd, join });

        debug!("listener clipboard Windows démarré");
        Ok(())
    }
}

fn now_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn clipboard_thread_main(
    impulse_tx: mpsc::Sender<ClipboardImpulse>,
    ready_tx: std::sync::mpsc::Sender<Result<isize, RunaError>>,
    suppress_until_ms: Arc<AtomicU64>,
) {
    if let Err(err) = ensure_window_class() {
        let _ = ready_tx.send(Err(err));
        return;
    }

    let state = Box::new(ListenerState {
        impulse_tx,
        has_emitted: false,
        last_at: Instant::now(),
        suppress_until_ms,
    });
    let state_ptr = Box::into_raw(state);

    let class_name = windows::core::w!("RunaClipboardListener");
    let hwnd = unsafe {
        match CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            windows::core::w!(""),
            WINDOW_STYLE::default(),
            0,
            0,
            0,
            0,
            Some(HWND_MESSAGE),
            None,
            None,
            Some(state_ptr as *const std::ffi::c_void),
        ) {
            Ok(h) if !h.is_invalid() => h,
            Ok(_) => {
                drop(Box::from_raw(state_ptr));
                let _ = ready_tx.send(Err(RunaError::Clipboard {
                    message: "CreateWindowExW HWND_MESSAGE invalide".into(),
                }));
                return;
            }
            Err(err) => {
                drop(Box::from_raw(state_ptr));
                let _ = ready_tx.send(Err(RunaError::Clipboard {
                    message: format!("CreateWindowExW: {err}"),
                }));
                return;
            }
        }
    };

    if let Err(err) = unsafe { AddClipboardFormatListener(hwnd) } {
        let _ = unsafe { DestroyWindow(hwnd) };
        let _ = ready_tx.send(Err(RunaError::Clipboard {
            message: format!("AddClipboardFormatListener: {err}"),
        }));
        return;
    }

    let _ = ready_tx.send(Ok(hwnd.0 as isize));

    let mut msg = MSG::default();
    loop {
        let got = unsafe { GetMessageW(&mut msg, None, 0, 0) };
        if got.0 == -1 {
            warn!("GetMessageW erreur");
            break;
        }
        if got.0 == 0 {
            break;
        }

        if msg.message == WM_RUNA_STOP {
            let _ = unsafe { RemoveClipboardFormatListener(hwnd) };
            let _ = unsafe { DestroyWindow(hwnd) };
            continue;
        }

        let _ = unsafe { TranslateMessage(&msg) };
        unsafe { DispatchMessageW(&msg) };
    }
}

fn ensure_window_class() -> Result<(), RunaError> {
    let cached = CLASS_REGISTERED.get_or_init(|| {
        let class_name = windows::core::w!("RunaClipboardListener");
        let hinstance = match unsafe { GetModuleHandleW(None) } {
            Ok(h) => h.into(),
            Err(err) => return Err(err.code().0 as u32),
        };

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(clipboard_wnd_proc),
            hInstance: hinstance,
            lpszClassName: class_name,
            ..Default::default()
        };

        let atom = unsafe { RegisterClassExW(&wc) };
        if atom == 0 {
            Err(unsafe { GetLastError().0 })
        } else {
            Ok(())
        }
    });

    match cached {
        Ok(()) => Ok(()),
        Err(code) => Err(RunaError::Clipboard {
            message: format!("RegisterClassExW échoué (code {code})"),
        }),
    }
}

unsafe extern "system" fn clipboard_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_NCCREATE => {
                let create = lparam.0 as *const CREATESTRUCTW;
                if create.is_null() {
                    return LRESULT(0);
                }
                let state_ptr = (*create).lpCreateParams;
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize);
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_CLIPBOARDUPDATE => {
                let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut ListenerState;
                if !state_ptr.is_null() {
                    let state = &mut *state_ptr;
                    let seq = GetClipboardSequenceNumber();
                    let epoch = now_epoch_ms();
                    if epoch < state.suppress_until_ms.load(Ordering::SeqCst) {
                        debug!(seq, "WM_CLIPBOARDUPDATE supprimé (écriture interne)");
                    } else {
                        let now = Instant::now();
                        if should_emit_impulse(state.has_emitted, state.last_at, now, COALESCE_MS)
                        {
                            state.has_emitted = true;
                            state.last_at = now;
                            let impulse = ClipboardImpulse {
                                sequence_hint: Some(u64::from(seq)),
                                immediate: false,
                            };
                            if let Err(err) = state.impulse_tx.try_send(impulse) {
                                debug!(
                                    ?err,
                                    "impulsion clipboard non envoyée (canal plein ou fermé)"
                                );
                            } else {
                                debug!(seq, "WM_CLIPBOARDUPDATE → impulsion");
                            }
                        } else {
                            debug!(seq, "WM_CLIPBOARDUPDATE coalescé");
                        }
                    }
                }
                LRESULT(0)
            }
            WM_DESTROY => {
                let state_ptr = SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0) as *mut ListenerState;
                if !state_ptr.is_null() {
                    drop(Box::from_raw(state_ptr));
                }
                PostQuitMessage(0);
                LRESULT(0)
            }
            WM_CREATE => LRESULT(0),
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

fn read_clipboard_text() -> Result<String, RunaError> {
    const ATTEMPTS: u32 = 3;
    let mut last_err: Option<RunaError> = None;

    for attempt in 0..ATTEMPTS {
        match try_read_clipboard_text() {
            Ok(text) => {
                debug!(len = text.len(), "texte presse-papiers lu");
                return Ok(text);
            }
            Err(err) => {
                last_err = Some(err);
                if attempt + 1 < ATTEMPTS {
                    std::thread::sleep(Duration::from_millis(15));
                }
            }
        }
    }

    Err(last_err.unwrap_or_else(|| RunaError::Clipboard {
        message: "lecture presse-papiers échouée".into(),
    }))
}

fn try_read_clipboard_text() -> Result<String, RunaError> {
    unsafe {
        OpenClipboard(None).map_err(|e| RunaError::Clipboard {
            message: format!("OpenClipboard: {e}"),
        })?;

        let result = read_clipboard_text_opened();
        let _ = CloseClipboard();
        result
    }
}

fn write_clipboard_utf16(text: &str) -> Result<(), RunaError> {
    let mut wide: Vec<u16> = text.encode_utf16().collect();
    wide.push(0);
    let bytes = wide.len() * 2;
    unsafe {
        let hmem = GlobalAlloc(GMEM_MOVEABLE, bytes).map_err(|e| RunaError::Clipboard {
            message: format!("GlobalAlloc: {e}"),
        })?;
        let ptr = GlobalLock(hmem);
        if ptr.is_null() {
            return Err(RunaError::Clipboard {
                message: "GlobalLock écriture clipboard échoué".into(),
            });
        }
        std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr as *mut u16, wide.len());
        let _ = GlobalUnlock(hmem);

        OpenClipboard(None).map_err(|e| RunaError::Clipboard {
            message: format!("OpenClipboard write: {e}"),
        })?;
        let result = (|| {
            EmptyClipboard().map_err(|e| RunaError::Clipboard {
                message: format!("EmptyClipboard: {e}"),
            })?;
            SetClipboardData(
                CF_UNICODETEXT,
                Some(windows::Win32::Foundation::HANDLE(hmem.0)),
            )
            .map_err(|e| RunaError::Clipboard {
                message: format!("SetClipboardData: {e}"),
            })?;
            Ok(())
        })();
        let _ = CloseClipboard();
        result?;
    }
    debug!(len = text.len(), "texte écrit dans le presse-papiers");
    Ok(())
}

unsafe fn read_clipboard_text_opened() -> Result<String, RunaError> {
    unsafe {
        if let Ok(handle) = GetClipboardData(CF_UNICODETEXT)
            && !handle.0.is_null()
        {
            let hglobal = windows::Win32::Foundation::HGLOBAL(handle.0);
            let ptr = GlobalLock(hglobal);
            if ptr.is_null() {
                return Err(RunaError::Clipboard {
                    message: "GlobalLock CF_UNICODETEXT échoué".into(),
                });
            }
            let wide = PCWSTR(ptr as *const u16);
            let text = match wide.to_string() {
                Ok(s) => s,
                Err(_) => {
                    let _ = GlobalUnlock(hglobal);
                    return Err(RunaError::Clipboard {
                        message: "UTF-16 presse-papiers invalide".into(),
                    });
                }
            };
            let _ = GlobalUnlock(hglobal);
            return Ok(text);
        }

        if let Ok(handle) = GetClipboardData(CF_TEXT)
            && !handle.0.is_null()
        {
            let hglobal = windows::Win32::Foundation::HGLOBAL(handle.0);
            let ptr = GlobalLock(hglobal) as *const u8;
            if ptr.is_null() {
                return Err(RunaError::Clipboard {
                    message: "GlobalLock CF_TEXT échoué".into(),
                });
            }
            let mut len = 0usize;
            while *ptr.add(len) != 0 {
                len += 1;
                if len > 8 * 1024 * 1024 {
                    let _ = GlobalUnlock(hglobal);
                    return Err(RunaError::Clipboard {
                        message: "CF_TEXT trop long".into(),
                    });
                }
            }
            let bytes = std::slice::from_raw_parts(ptr, len);
            let text = String::from_utf8_lossy(bytes).into_owned();
            let _ = GlobalUnlock(hglobal);
            return Ok(text);
        }

        Ok(String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set_clipboard_utf16(text: &str) -> Result<(), String> {
        write_clipboard_utf16(text).map_err(|e| e.to_string())
    }

    #[tokio::test(flavor = "current_thread")]
    #[ignore = "smoke OS — vrai clipboard Windows ; lancer manuellement (cargo test -p platform-engine -- --ignored)"]
    async fn listener_emits_on_two_clipboard_writes() {
        let _guard = crate::windows::clipboard_test_lock();
        let watcher = WindowsClipboard::default();
        let (tx, mut rx) = mpsc::channel(16);
        watcher.start(tx).await.expect("start listener");

        tokio::time::sleep(Duration::from_millis(50)).await;

        set_clipboard_utf16("runa-s1-a").expect("set clipboard a");
        tokio::time::sleep(Duration::from_millis(80)).await;
        set_clipboard_utf16("runa-s1-b").expect("set clipboard b");

        let first = tokio::time::timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("timeout impulse 1")
            .expect("impulse 1");
        let second = tokio::time::timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("timeout impulse 2")
            .expect("impulse 2");

        assert!(first.sequence_hint.is_some());
        assert!(second.sequence_hint.is_some());

        let text = watcher.read_text().await.expect("read text");
        assert_eq!(text, "runa-s1-b");

        watcher.stop().await.expect("stop listener");
    }

    #[tokio::test(flavor = "current_thread")]
    #[ignore = "smoke OS — vrai clipboard Windows ; lancer manuellement (cargo test -p platform-engine -- --ignored)"]
    async fn write_text_suppresses_impulse() {
        let _guard = crate::windows::clipboard_test_lock();
        let watcher = WindowsClipboard::default();
        let (tx, mut rx) = mpsc::channel(16);
        watcher.start(tx).await.expect("start listener");
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Drain any startup noise.
        while rx.try_recv().is_ok() {}

        watcher
            .write_text("runa-s2-suppress")
            .await
            .expect("write_text");
        tokio::time::sleep(Duration::from_millis(SUPPRESS_WRITE_MS + 20)).await;

        assert!(
            rx.try_recv().is_err(),
            "écriture interne ne doit pas émettre d'impulsion"
        );

        let text = watcher.read_text().await.expect("read");
        assert_eq!(text, "runa-s2-suppress");

        watcher.stop().await.expect("stop");
    }
}
