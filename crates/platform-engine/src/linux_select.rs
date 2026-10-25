//! Sélection pure du backend clipboard Linux (testable hors `cfg(linux)`).

/// Nature de la session d'affichage détectée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionKind {
    /// Wayland pur (pas de `DISPLAY` utile pour XFixes).
    Wayland,
    /// X11 natif ou XWayland (`DISPLAY` présent).
    X11OrXWayland,
    /// Aucun affichage détecté.
    Unknown,
}

/// Résultat d'une sonde protocole / capacité.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeResult {
    Available,
    Missing,
}

/// Backend clipboard choisi après la chaîne de sondes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardBackend {
    ExtDataControl,
    WlrDataControl,
    XFixes,
    Evdev,
}

/// Entrées observées pour la sélection (injectables en test).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProbeInputs {
    pub session: SessionKind,
    pub ext_data_control: ProbeResult,
    pub wlr_data_control: ProbeResult,
    pub linux_evdev_opt_in: bool,
}

/// Raison d'échec quand aucun backend n'est disponible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectFailure {
    /// Wayland sans data-control et evdev désactivé.
    NeedEvdevOptIn,
    /// Session inconnue / sans chemin viable.
    Unsupported,
}

/// Chaîne : ext → wlr → XFixes (si DISPLAY) → evdev opt-in.
pub fn select_clipboard_backend(
    inputs: ProbeInputs,
) -> Result<ClipboardBackend, SelectFailure> {
    if inputs.ext_data_control == ProbeResult::Available {
        return Ok(ClipboardBackend::ExtDataControl);
    }
    if inputs.wlr_data_control == ProbeResult::Available {
        return Ok(ClipboardBackend::WlrDataControl);
    }
    match inputs.session {
        SessionKind::X11OrXWayland => Ok(ClipboardBackend::XFixes),
        SessionKind::Wayland | SessionKind::Unknown => {
            if inputs.linux_evdev_opt_in {
                Ok(ClipboardBackend::Evdev)
            } else if matches!(inputs.session, SessionKind::Unknown) {
                Err(SelectFailure::Unsupported)
            } else {
                Err(SelectFailure::NeedEvdevOptIn)
            }
        }
    }
}

/// Détecte si un raccourci GlobalShortcuts normalise vers Ctrl+C (interdit).
///
/// Accepte les séparateurs `+`, `-`, espace ; ignore la casse et l'ordre des modificateurs.
pub fn shortcut_is_ctrl_c(spec: &str) -> bool {
    let normalized = spec.trim().to_ascii_uppercase().replace('-', "+");
    if normalized.is_empty() {
        return false;
    }
    let parts: Vec<&str> = normalized
        .split(|c: char| c == '+' || c.is_whitespace())
        .filter(|p| !p.is_empty())
        .collect();

    let mut has_ctrl = false;
    let mut has_c = false;
    let mut has_extra_mod = false;
    let mut has_extra_key = false;

    for part in parts {
        match part {
            "CTRL" | "CONTROL" | "PRIMARY" => has_ctrl = true,
            "C" => {
                if has_c {
                    has_extra_key = true;
                } else {
                    has_c = true;
                }
            }
            "SHIFT" | "ALT" | "META" | "SUPER" | "HYPER" | "MOD1" | "MOD2" | "MOD3" | "MOD4"
            | "MOD5" | "LOGO" => has_extra_mod = true,
            _ => has_extra_key = true,
        }
    }

    has_ctrl && has_c && !has_extra_mod && !has_extra_key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kde_wlr_preferred_over_evdev() {
        let backend = select_clipboard_backend(ProbeInputs {
            session: SessionKind::Wayland,
            ext_data_control: ProbeResult::Missing,
            wlr_data_control: ProbeResult::Available,
            linux_evdev_opt_in: true,
        });
        assert_eq!(backend, Ok(ClipboardBackend::WlrDataControl));
    }

    #[test]
    fn ext_beats_wlr() {
        let backend = select_clipboard_backend(ProbeInputs {
            session: SessionKind::Wayland,
            ext_data_control: ProbeResult::Available,
            wlr_data_control: ProbeResult::Available,
            linux_evdev_opt_in: false,
        });
        assert_eq!(backend, Ok(ClipboardBackend::ExtDataControl));
    }

    #[test]
    fn gnome_without_protocol_needs_evdev_or_fails() {
        let fail = select_clipboard_backend(ProbeInputs {
            session: SessionKind::Wayland,
            ext_data_control: ProbeResult::Missing,
            wlr_data_control: ProbeResult::Missing,
            linux_evdev_opt_in: false,
        });
        assert_eq!(fail, Err(SelectFailure::NeedEvdevOptIn));

        let ok = select_clipboard_backend(ProbeInputs {
            session: SessionKind::Wayland,
            ext_data_control: ProbeResult::Missing,
            wlr_data_control: ProbeResult::Missing,
            linux_evdev_opt_in: true,
        });
        assert_eq!(ok, Ok(ClipboardBackend::Evdev));
    }

    #[test]
    fn x11_uses_xfixes() {
        let backend = select_clipboard_backend(ProbeInputs {
            session: SessionKind::X11OrXWayland,
            ext_data_control: ProbeResult::Missing,
            wlr_data_control: ProbeResult::Missing,
            linux_evdev_opt_in: false,
        });
        assert_eq!(backend, Ok(ClipboardBackend::XFixes));
    }

    #[test]
    fn unknown_without_evdev_unsupported() {
        let fail = select_clipboard_backend(ProbeInputs {
            session: SessionKind::Unknown,
            ext_data_control: ProbeResult::Missing,
            wlr_data_control: ProbeResult::Missing,
            linux_evdev_opt_in: false,
        });
        assert_eq!(fail, Err(SelectFailure::Unsupported));
    }

    #[test]
    fn shortcut_ctrl_c_variants_forbidden() {
        assert!(shortcut_is_ctrl_c("CTRL+C"));
        assert!(shortcut_is_ctrl_c("ctrl-c"));
        assert!(shortcut_is_ctrl_c("Control+C"));
        assert!(shortcut_is_ctrl_c("PRIMARY+C"));
        assert!(shortcut_is_ctrl_c("  Ctrl + C  "));
    }

    #[test]
    fn shortcut_with_shift_allowed() {
        assert!(!shortcut_is_ctrl_c("CTRL+SHIFT+SPACE"));
        assert!(!shortcut_is_ctrl_c("CTRL+SHIFT+C"));
        assert!(!shortcut_is_ctrl_c("ALT+C"));
        assert!(!shortcut_is_ctrl_c(""));
    }
}
