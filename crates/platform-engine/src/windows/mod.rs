//! Adapters Windows 11 (clipboard, focus, injector).

mod clipboard;
mod focus;
mod injector;

pub use clipboard::WindowsClipboard;
pub use focus::WindowsFocus;
pub use injector::WindowsInjector;

#[cfg(test)]
pub(crate) fn clipboard_test_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}
