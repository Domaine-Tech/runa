//! Injection Ctrl+V via SendInput (S2).
#![allow(unsafe_code)]

use async_trait::async_trait;
use shared_types::RunaError;
use tracing::debug;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_KEYUP,
    VIRTUAL_KEY, VK_CONTROL, VK_V,
};

use crate::injector::TextInjector;

/// Adapter injection clavier Windows.
#[derive(Debug, Default)]
pub struct WindowsInjector;

#[async_trait]
impl TextInjector for WindowsInjector {
    async fn paste_clipboard(&self) -> Result<(), RunaError> {
        tokio::task::spawn_blocking(send_ctrl_v)
            .await
            .map_err(|e| RunaError::Internal {
                message: format!("SendInput: {e}"),
            })?
    }
}

fn key_input(vk: VIRTUAL_KEY, up: bool) -> INPUT {
    let flags = if up {
        KEYEVENTF_KEYUP
    } else {
        KEYBD_EVENT_FLAGS(0)
    };
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn send_ctrl_v() -> Result<(), RunaError> {
    let inputs = [
        key_input(VK_CONTROL, false),
        key_input(VK_V, false),
        key_input(VK_V, true),
        key_input(VK_CONTROL, true),
    ];
    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    if sent as usize != inputs.len() {
        return Err(RunaError::InjectionFallback {
            message: format!("SendInput Ctrl+V incomplet ({sent}/{})", inputs.len()),
        });
    }
    debug!("SendInput Ctrl+V OK");
    Ok(())
}
