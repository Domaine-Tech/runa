//! Ports OS et sélection d'adapter selon la plateforme cible.

pub mod clipboard;
pub mod focus;
pub mod injector;
pub mod linux_select;
pub mod trigger;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

pub use clipboard::ClipboardWatcher;
pub use focus::WindowFocus;
pub use injector::TextInjector;
pub use trigger::{create_trigger_engine, TriggerEngine};

use shared_types::{RunaError, TriggerConfig};

/// Construit les adapters natifs pour l'OS courant.
///
/// `trigger` est lu sur Linux (evdev opt-in, raccourci portail) ; ignoré ailleurs.
pub fn create_platform_stack(trigger: &TriggerConfig) -> Result<PlatformStack, RunaError> {
    #[cfg(target_os = "windows")]
    {
        let _ = trigger;
        Ok(PlatformStack {
            clipboard: Box::new(windows::WindowsClipboard::default()),
            focus: Box::new(windows::WindowsFocus),
            injector: Box::new(windows::WindowsInjector),
        })
    }

    #[cfg(target_os = "macos")]
    {
        let _ = trigger;
        return Ok(PlatformStack {
            clipboard: Box::new(macos::MacosClipboard::default()),
            focus: Box::new(macos::MacosFocus::default()),
            injector: Box::new(macos::MacosInjector::default()),
        });
    }

    #[cfg(target_os = "linux")]
    {
        return Ok(PlatformStack {
            clipboard: Box::new(linux::LinuxClipboard::new(trigger)),
            focus: Box::new(linux::LinuxFocus::new()),
            injector: Box::new(linux::LinuxInjector::new()),
        });
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        let _ = trigger;
        Err(RunaError::unsupported("plateforme non supportée"))
    }
}

pub struct PlatformStack {
    pub clipboard: Box<dyn ClipboardWatcher>,
    pub focus: Box<dyn WindowFocus>,
    pub injector: Box<dyn TextInjector>,
}
