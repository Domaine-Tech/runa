//! Auto-update (S5) — endpoint GitHub Releases / latest.json uniquement.
//! Aucun texte utilisateur / presse-papiers n’est envoyé.

use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_updater::UpdaterExt;
use tracing::{info, warn};

/// Check réseau au démarrage : log seulement, jamais d’install silencieuse.
pub fn spawn_startup_check(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        match check_for_update(&app).await {
            Ok(Some(version)) => {
                info!(%version, "mise à jour disponible (tray → Vérifier les mises à jour)");
            }
            Ok(None) => {
                info!("pas de mise à jour (startup check)");
            }
            Err(err) => {
                warn!(error = %err, "check MAJ au démarrage échoué — app continue");
            }
        }
    });
}

/// Tray : vérifier, confirmer, télécharger + installer, relancer.
pub fn spawn_tray_check(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        if let Err(err) = check_prompt_and_install(&app).await {
            warn!(error = %err, "vérification / installation MAJ échouée");
            let _ = app
                .dialog()
                .message(format!("Impossible de vérifier ou installer la mise à jour.\n{err}"))
                .title("Mise à jour")
                .kind(MessageDialogKind::Error)
                .blocking_show();
        }
    });
}

async fn check_for_update(app: &AppHandle) -> Result<Option<String>, String> {
    let updater = app
        .updater()
        .map_err(|e| format!("plugin updater: {e}"))?;
    match updater.check().await {
        Ok(Some(update)) => Ok(Some(update.version.clone())),
        Ok(None) => Ok(None),
        Err(e) => Err(format!("check: {e}")),
    }
}

async fn check_prompt_and_install(app: &AppHandle) -> Result<(), String> {
    let updater = app
        .updater()
        .map_err(|e| format!("plugin updater: {e}"))?;
    let Some(update) = updater
        .check()
        .await
        .map_err(|e| format!("check: {e}"))?
    else {
        let _ = app
            .dialog()
            .message("Runa est à jour.")
            .title("Mise à jour")
            .kind(MessageDialogKind::Info)
            .blocking_show();
        return Ok(());
    };

    let version = update.version.clone();
    let confirmed = app
        .dialog()
        .message(format!(
            "Une nouvelle version est disponible : {version}\n\nInstaller maintenant ?"
        ))
        .title("Mise à jour Runa")
        .kind(MessageDialogKind::Info)
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Installer".into(),
            "Plus tard".into(),
        ))
        .blocking_show();

    if !confirmed {
        info!(%version, "mise à jour reportée par l'utilisateur");
        return Ok(());
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
}
