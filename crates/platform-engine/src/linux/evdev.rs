//! Fallback opt-in `/dev/input` — filtre Ctrl+C uniquement, aucune persistance.

#![allow(unsafe_code)]

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self as std_mpsc, Sender};
use std::sync::Arc;
use std::time::Duration;

use shared_types::RunaError;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use crate::clipboard::ClipboardImpulse;

/// Démarre un thread `lin-evdev` qui émet une impulsion à chaque Ctrl+C (press C avec Ctrl).
///
/// Retourne seulement après ouverture réussie des devices (canal `ready`), sinon
/// `PlatformPermission` (groupe `input`, etc.).
pub fn spawn_listener(
    impulse_tx: mpsc::Sender<ClipboardImpulse>,
) -> Result<(Arc<AtomicBool>, std::thread::JoinHandle<()>), RunaError> {
    let stop = Arc::new(AtomicBool::new(false));
    let stop_thread = Arc::clone(&stop);
    let (ready_tx, ready_rx) = std_mpsc::channel::<Result<(), RunaError>>();

    let join = std::thread::Builder::new()
        .name("lin-evdev".into())
        .spawn(move || {
            if let Err(err) = run_loop(impulse_tx, stop_thread, &ready_tx) {
                let _ = ready_tx.send(Err(err));
            }
        })
        .map_err(|e| RunaError::PlatformPermission {
            message: format!("spawn lin-evdev: {e}"),
        })?;

    ready_rx
        .recv_timeout(Duration::from_secs(5))
        .map_err(|_| RunaError::PlatformPermission {
            message: "timeout démarrage lin-evdev".into(),
        })??;

    Ok((stop, join))
}

pub fn stop_listener(stop: &AtomicBool, join: std::thread::JoinHandle<()>) {
    stop.store(true, Ordering::SeqCst);
    if let Err(err) = join.join() {
        warn!(?err, "join lin-evdev échoué");
    }
}

fn run_loop(
    impulse_tx: mpsc::Sender<ClipboardImpulse>,
    stop: Arc<AtomicBool>,
    ready_tx: &Sender<Result<(), RunaError>>,
) -> Result<(), RunaError> {
    let devices = list_keyboard_devices()?;
    if devices.is_empty() {
        return Err(RunaError::PlatformPermission {
            message: "aucun périphérique clavier evdev (groupe `input` ?)".into(),
        });
    }

    let mut handles = Vec::new();
    for path in &devices {
        match evdev::Device::open(path) {
            Ok(d) => {
                // Ne garder que les devices EV_KEY (claviers) — ignorer souris / touchpad / etc.
                if !d.supported_events().contains(evdev::EventType::KEY) {
                    debug!(path = %path.display(), "evdev: pas EV_KEY, ignoré");
                    continue;
                }
                // O_NONBLOCK via libc — Device 0.12 n'expose pas set_nonblocking.
                let fd = std::os::fd::AsRawFd::as_raw_fd(&d);
                unsafe {
                    let flags = libc::fcntl(fd, libc::F_GETFL);
                    if flags >= 0 {
                        let _ = libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
                    }
                }
                handles.push(d);
            }
            Err(err) => {
                let io = err.raw_os_error();
                if io == Some(13) {
                    return Err(RunaError::PlatformPermission {
                        message: format!(
                            "accès refusé à {} — ajouter l'utilisateur au groupe `input`",
                            path.display()
                        ),
                    });
                }
                debug!(path = %path.display(), ?err, "evdev open ignoré");
            }
        }
    }
    if handles.is_empty() {
        return Err(RunaError::PlatformPermission {
            message: "impossible d'ouvrir un clavier evdev (permissions)".into(),
        });
    }

    info!(count = handles.len(), "lin-evdev: claviers ouverts");
    ready_tx
        .send(Ok(()))
        .map_err(|_| RunaError::PlatformPermission {
            message: "canal ready lin-evdev fermé".into(),
        })?;

    let mut ctrl_left = false;
    let mut ctrl_right = false;

    while !stop.load(Ordering::SeqCst) {
        let mut progressed = false;
        for device in &mut handles {
            match device.fetch_events() {
                Ok(events) => {
                    for ev in events {
                        progressed = true;
                        if let evdev::InputEventKind::Key(key) = ev.kind() {
                            let value = ev.value();
                            match key {
                                evdev::Key::KEY_LEFTCTRL => {
                                    ctrl_left = value != 0;
                                }
                                evdev::Key::KEY_RIGHTCTRL => {
                                    ctrl_right = value != 0;
                                }
                                evdev::Key::KEY_C if value == 1 && (ctrl_left || ctrl_right) => {
                                    let impulse = ClipboardImpulse {
                                        sequence_hint: None,
                                        immediate: false,
                                    };
                                    if let Err(err) = impulse_tx.try_send(impulse) {
                                        debug!(?err, "impulsion evdev non envoyée");
                                    } else {
                                        debug!("evdev Ctrl+C → impulsion");
                                    }
                                }
                                _ => {
                                    // Autres touches : jetées, non loguées, non stockées.
                                }
                            }
                        }
                    }
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(err) => {
                    debug!(?err, "evdev fetch_events");
                }
            }
        }
        if !progressed {
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    Ok(())
}

fn list_keyboard_devices() -> Result<Vec<PathBuf>, RunaError> {
    let dir = std::fs::read_dir("/dev/input").map_err(|e| {
        if e.raw_os_error() == Some(13) {
            RunaError::PlatformPermission {
                message: "accès refusé à /dev/input — groupe `input` requis".into(),
            }
        } else {
            RunaError::PlatformPermission {
                message: format!("/dev/input: {e}"),
            }
        }
    })?;

    let mut out = Vec::new();
    for entry in dir.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if !name.starts_with("event") {
            continue;
        }
        // Ouverture tentative différée dans run_loop ; ici on liste seulement.
        out.push(path);
    }
    Ok(out)
}
