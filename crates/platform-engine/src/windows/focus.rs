//! Capture / restauration HWND + caret Windows (S1 capture, S2 restore + caret).
#![allow(unsafe_code)]

use async_trait::async_trait;
use shared_types::{RunaError, ScreenPoint, WindowToken};
use tracing::debug;
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_KEYUP,
    VIRTUAL_KEY, VK_MENU,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AllowSetForegroundWindow, BringWindowToTop, GetAncestor, GetForegroundWindow, GetGUIThreadInfo,
    GetWindowThreadProcessId, IsIconic, IsWindow, SetForegroundWindow, SetWindowPos, ShowWindow,
    SystemParametersInfoW, ASFW_ANY, GA_ROOT, GUITHREADINFO, HWND_NOTOPMOST, HWND_TOPMOST,
    SPI_GETFOREGROUNDLOCKTIMEOUT, SPI_SETFOREGROUNDLOCKTIMEOUT, SPIF_SENDCHANGE, SWP_NOMOVE,
    SWP_NOSIZE, SW_RESTORE, SW_SHOW,
};

use crate::focus::WindowFocus;

/// Adapter focus Windows.
#[derive(Debug, Default)]
pub struct WindowsFocus;

#[async_trait]
impl WindowFocus for WindowsFocus {
    async fn capture_foreground(&self) -> Result<WindowToken, RunaError> {
        let hwnd = unsafe { GetForegroundWindow() };
        if hwnd.is_invalid() || hwnd.0.is_null() {
            debug!("GetForegroundWindow: aucun HWND (Unknown)");
            return Ok(WindowToken::Unknown);
        }
        // Toujours le toplevel : un HWND enfant (edit) casse souvent SetForegroundWindow.
        let root = unsafe { GetAncestor(hwnd, GA_ROOT) };
        let target = if root.is_invalid() || root.0.is_null() {
            hwnd
        } else {
            root
        };
        let raw = target.0 as usize as u64;
        debug!(hwnd = raw, "HWND foreground capturé");
        Ok(WindowToken::Windows { hwnd: raw })
    }

    async fn restore(&self, token: &WindowToken) -> Result<(), RunaError> {
        let WindowToken::Windows { hwnd } = token else {
            return Err(RunaError::WindowFocus {
                message: "jeton de focus Windows attendu".into(),
            });
        };
        let hwnd_raw = *hwnd;
        tokio::task::spawn_blocking(move || restore_hwnd(hwnd_raw))
            .await
            .map_err(|e| RunaError::Internal {
                message: format!("restore focus: {e}"),
            })?
    }

    async fn capture_caret(&self) -> Result<Option<ScreenPoint>, RunaError> {
        tokio::task::spawn_blocking(capture_caret_screen)
            .await
            .map_err(|e| RunaError::Internal {
                message: format!("capture caret: {e}"),
            })
    }
}

fn restore_hwnd(hwnd_raw: u64) -> Result<(), RunaError> {
    let hwnd = HWND(hwnd_raw as *mut std::ffi::c_void);
    unsafe {
        if !IsWindow(Some(hwnd)).as_bool() {
            return Err(RunaError::WindowFocus {
                message: "HWND invalide ou détruit".into(),
            });
        }

        let root = GetAncestor(hwnd, GA_ROOT);
        let hwnd = if root.is_invalid() || root.0.is_null() {
            hwnd
        } else {
            root
        };

        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        } else {
            let _ = ShowWindow(hwnd, SW_SHOW);
        }

        // WebView2 détient souvent le FG : notre process n'est pas « foreground ».
        // On déverrouille via Alt + timeout FG + AllowSetForegroundWindow.
        tap_alt_key();
        let _ = AllowSetForegroundWindow(ASFW_ANY);
        let lock_guard = ForegroundLockGuard::clear();

        let target_tid = GetWindowThreadProcessId(hwnd, None);
        let current_tid = GetCurrentThreadId();
        let foreground = GetForegroundWindow();
        let fg_tid = if foreground.is_invalid() || foreground.0.is_null() {
            0
        } else {
            GetWindowThreadProcessId(foreground, None)
        };

        let mut attached_fg = false;
        if fg_tid != 0 && fg_tid != current_tid {
            attached_fg = AttachThreadInput(current_tid, fg_tid, true).as_bool();
        }
        let mut attached_target = false;
        if target_tid != 0 && target_tid != current_tid && target_tid != fg_tid {
            attached_target = AttachThreadInput(current_tid, target_tid, true).as_bool();
        }

        // Astuce topmost : force l'activation même si SetForegroundWindow refuse.
        let _ = SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE,
        );
        let _ = SetWindowPos(
            hwnd,
            Some(HWND_NOTOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE,
        );

        let _ = BringWindowToTop(hwnd);
        let mut ok = SetForegroundWindow(hwnd).as_bool();

        if !ok {
            tap_alt_key();
            let _ = BringWindowToTop(hwnd);
            ok = SetForegroundWindow(hwnd).as_bool();
        }

        if attached_target {
            let _ = AttachThreadInput(current_tid, target_tid, false);
        }
        if attached_fg {
            let _ = AttachThreadInput(current_tid, fg_tid, false);
        }

        drop(lock_guard);

        let now = GetForegroundWindow();
        let now_root = if now.is_invalid() || now.0.is_null() {
            now
        } else {
            let r = GetAncestor(now, GA_ROOT);
            if r.is_invalid() || r.0.is_null() {
                now
            } else {
                r
            }
        };
        let focused = !now_root.is_invalid() && now_root.0 == hwnd.0;
        if !ok && !focused {
            return Err(RunaError::WindowFocus {
                message: "SetForegroundWindow échoué".into(),
            });
        }
    }
    debug!(hwnd = hwnd_raw, "focus restauré");
    Ok(())
}

/// Remet `SPI_FOREGROUNDLOCKTIMEOUT` à 0 le temps du restore, puis restaure.
struct ForegroundLockGuard {
    previous: u32,
    active: bool,
}

impl ForegroundLockGuard {
    fn clear() -> Self {
        let mut previous: u32 = 0;
        let ok_get = unsafe {
            SystemParametersInfoW(
                SPI_GETFOREGROUNDLOCKTIMEOUT,
                0,
                Some((&raw mut previous).cast()),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS_ZERO,
            )
        }
        .is_ok();
        if !ok_get {
            return Self {
                previous: 0,
                active: false,
            };
        }
        let ok_set = unsafe {
            SystemParametersInfoW(
                SPI_SETFOREGROUNDLOCKTIMEOUT,
                0,
                Some(std::ptr::null_mut()),
                SPIF_SENDCHANGE,
            )
        }
        .is_ok();
        Self {
            previous,
            active: ok_set,
        }
    }
}

impl Drop for ForegroundLockGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        let timeout = self.previous;
        let _ = unsafe {
            SystemParametersInfoW(
                SPI_SETFOREGROUNDLOCKTIMEOUT,
                0,
                Some(timeout as usize as *mut _),
                SPIF_SENDCHANGE,
            )
        };
    }
}

/// Flag nul pour GET (pas de broadcast).
const SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS_ZERO: windows::Win32::UI::WindowsAndMessaging::SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS =
    windows::Win32::UI::WindowsAndMessaging::SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0);

/// Pulse Alt pour débloquer les restrictions SetForegroundWindow.
fn tap_alt_key() {
    let inputs = [
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(VK_MENU.0),
                    wScan: 0,
                    dwFlags: KEYBD_EVENT_FLAGS(0),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(VK_MENU.0),
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
    ];
    let _ = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
}

fn capture_caret_screen() -> Option<ScreenPoint> {
    unsafe {
        let mut info = GUITHREADINFO {
            cbSize: std::mem::size_of::<GUITHREADINFO>() as u32,
            ..Default::default()
        };
        if GetGUIThreadInfo(0, &mut info).is_err() {
            debug!("GetGUIThreadInfo échoué");
            return None;
        }
        if info.hwndCaret.is_invalid() || info.hwndCaret.0.is_null() {
            debug!("pas de hwndCaret");
            return None;
        }
        let mut pt = POINT {
            x: info.rcCaret.left,
            y: info.rcCaret.top,
        };
        if !ClientToScreen(info.hwndCaret, &mut pt).as_bool() {
            debug!("ClientToScreen caret échoué");
            return None;
        }
        debug!(x = pt.x, y = pt.y, "caret écran capturé");
        Some(ScreenPoint { x: pt.x, y: pt.y })
    }
}
