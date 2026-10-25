//! Détection de session d'affichage Linux.

use crate::linux_select::SessionKind;

/// `DISPLAY` → X11/XWayland ; sinon `WAYLAND_DISPLAY` → Wayland pur.
pub fn detect_session() -> SessionKind {
    if std::env::var_os("DISPLAY").is_some() {
        SessionKind::X11OrXWayland
    } else if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        SessionKind::Wayland
    } else {
        SessionKind::Unknown
    }
}

/// Injection XTest uniquement hors session Wayland (S3 : Wayland → repli clipboard).
pub fn can_xtest_inject() -> bool {
    std::env::var_os("WAYLAND_DISPLAY").is_none() && std::env::var_os("DISPLAY").is_some()
}

/// Capture / restore focus X11 si `DISPLAY` est défini.
pub fn has_x11_display() -> bool {
    std::env::var_os("DISPLAY").is_some()
}
