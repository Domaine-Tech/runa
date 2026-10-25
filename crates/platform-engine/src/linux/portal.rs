//! Raccourci alternatif via `ashpd` GlobalShortcuts (≠ Ctrl+C).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use ashpd::desktop::global_shortcuts::{GlobalShortcuts, NewShortcut};
use futures_util::StreamExt;
use shared_types::RunaError;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use crate::clipboard::ClipboardImpulse;
use crate::linux_select::shortcut_is_ctrl_c;

/// Lie GlobalShortcuts puis lance l'écoute sur la runtime Tokio courante.
///
/// Spec invalide (Ctrl+C) → `Err(Config)` sans appeler le portail.
/// `create_session` + `bind_shortcuts` réussissent avant `Ok` ; la boucle
/// `receive_activated` tourne ensuite en tâche de fond.
pub async fn spawn_portal_listener(
    shortcut: &str,
    impulse_tx: mpsc::Sender<ClipboardImpulse>,
) -> Result<Arc<AtomicBool>, RunaError> {
    if shortcut_is_ctrl_c(shortcut) {
        return Err(RunaError::Config {
            message: format!(
                "raccourci portail interdit (avalerait la copie): {shortcut} — utiliser p.ex. CTRL+SHIFT+SPACE"
            ),
        });
    }

    let proxy = GlobalShortcuts::new()
        .await
        .map_err(|e| RunaError::PlatformPermission {
            message: format!("portail GlobalShortcuts indisponible: {e}"),
        })?;

    let session = proxy
        .create_session()
        .await
        .map_err(|e| RunaError::PlatformPermission {
            message: format!("create_session GlobalShortcuts: {e}"),
        })?;

    let new_shortcut =
        NewShortcut::new("runa-trigger", "Ouvrir Runa").preferred_trigger(Some(shortcut));

    proxy
        .bind_shortcuts(&session, &[new_shortcut], None)
        .await
        .map_err(|e| RunaError::PlatformPermission {
            message: format!("bind_shortcuts: {e}"),
        })?;

    info!(%shortcut, "GlobalShortcuts lié");

    let stop = Arc::new(AtomicBool::new(false));
    let stop_task = Arc::clone(&stop);

    tokio::spawn(async move {
        if let Err(err) = listen_activated(proxy, impulse_tx, stop_task).await {
            warn!(?err, "GlobalShortcuts portail arrêté");
        }
    });

    Ok(stop)
}

pub fn stop_portal(stop: &AtomicBool) {
    stop.store(true, Ordering::SeqCst);
}

async fn listen_activated(
    proxy: GlobalShortcuts<'_>,
    impulse_tx: mpsc::Sender<ClipboardImpulse>,
    stop: Arc<AtomicBool>,
) -> Result<(), RunaError> {
    let mut activated = proxy.receive_activated().await.map_err(|e| {
        RunaError::PlatformPermission {
            message: format!("receive_activated: {e}"),
        }
    })?;

    while !stop.load(Ordering::SeqCst) {
        tokio::select! {
            item = activated.next() => {
                match item {
                    Some(_activated) => {
                        let impulse = ClipboardImpulse {
                            sequence_hint: None,
                            immediate: true,
                        };
                        if let Err(err) = impulse_tx.try_send(impulse) {
                            debug!(?err, "impulsion portail non envoyée");
                        } else {
                            debug!("GlobalShortcuts → impulsion immédiate");
                        }
                    }
                    None => break,
                }
            }
            _ = tokio::time::sleep(std::time::Duration::from_millis(200)) => {}
        }
    }

    Ok(())
}