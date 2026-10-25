//! Auto-update (S5) — endpoint GitHub Releases / latest.json uniquement.
//! Aucun texte utilisateur / presse-papiers n’est envoyé.
//! Jamais d’install / restart sans confirmation ; garde si inférence active.

use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_updater::UpdaterExt;
use tracing::{info, warn};

use crate::state::AppState;

/// Check au démarrage : si MAJ dispo → dialogue (pas d’install silencieuse).
pub fn spawn_startup_check(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        match offer_update_if_available(&app, false).await {
            Ok(UpdateOffer::None) => info!("pas de mise à jour (startup check)"),
            Ok(UpdateOffer::Deferred { version }) => {
                info!(%version, "MAJ reportée au démarrage");
            }
            Err(err) => {
                warn!(error = %err, "check MAJ au démarrage échoué — app continue");
            }
        }
    });
}

/// Tray : vérifier, confirmer, éventuellement installer.
pub fn spawn_tray_check(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        match offer_update_if_available(&app, true).await {
            Ok(UpdateOffer::None) => {
                let _ = app
                    .dialog()
                    .message("Runa est à jour.")
                    .title("Mise à jour")
                    .kind(MessageDialogKind::Info)
                    .blocking_show();
            }
            Ok(UpdateOffer::Deferred { version }) => {
                info!(%version, "MAJ reportée (tray)");
            }
            Err(err) => {
                warn!(error = %err, "vérification / installation MAJ échouée");
                let _ = app
                    .dialog()
                    .message(format!(
                        "Impossible de vérifier ou installer la mise à jour.\n{err}"
                    ))
                    .title("Mise à jour")
                    .kind(MessageDialogKind::Error)
                    .blocking_show();
            }
        }
    });
}

enum UpdateOffer {
    None,
    Deferred { version: String },
}

async fn offer_update_if_available(
    app: &AppHandle,
    _from_tray: bool,
) -> Result<UpdateOffer, String> {
    let updater = app
        .updater()
        .map_err(|e| format!("plugin updater: {e}"))?;
    let Some(update) = updater
        .check()
        .await
        .map_err(|e| format!("check: {e}"))?
    else {
        return Ok(UpdateOffer::None);
    };

    let version = update.version.clone();
    let confirmed = app
        .dialog()
        .message(format!(
            "Une nouvelle version est disponible : {version}\n\nInstaller maintenant ?\n(L’application redémarrera.)"
        ))
        .title("Mise à jour Runa")
        .kind(MessageDialogKind::Info)
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Installer".into(),
            "Plus tard".into(),
        ))
        .blocking_show();

    if !confirmed {
        return Ok(UpdateOffer::Deferred { version });
    }

    if inference_busy(app) {
        let interrupt = app
            .dialog()
            .message(
                "Une correction ou traduction est en cours.\n\n\
                 Mettre à jour maintenant annulera cette tâche et redémarrera Runa.",
            )
            .title("Travail en cours")
            .kind(MessageDialogKind::Warning)
            .buttons(MessageDialogButtons::OkCancelCustom(
                "Mettre à jour".into(),
                "Annuler".into(),
            ))
            .blocking_show();
        if !interrupt {
            return Ok(UpdateOffer::Deferred { version });
        }
        if let Some(state) = app.try_state::<AppState>() {
            if let Err(err) = state.ai.cancel().await {
                warn!(?err, "annulation inférence avant MAJ échouée");
            }
        }
    }

    info!(%version, "téléchargement mise à jour");
    update
        .download_and_install(
            |_chunk_len, _content_len| {},
            || {
                info!("téléchargement MAJ terminé — installation");
            },
        )
        .await
        .map_err(|e| format!("download_and_install: {e}"))?;

    info!(%version, "mise à jour installée — redémarrage");
    app.restart();
    // `restart` ne revient en principe pas ; repli pour le type.
    #[allow(unreachable_code)]
    {
        Ok(UpdateOffer::Deferred { version })
    }
}

fn inference_busy(app: &AppHandle) -> bool {
    app.try_state::<AppState>()
        .map(|s| s.ai.is_busy())
        .unwrap_or(false)
}
