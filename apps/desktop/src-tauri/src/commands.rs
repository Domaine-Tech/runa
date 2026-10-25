use shared_types::{
    AccentPreset, ApplyReplacementRequest, AppConfig, CorrectRequest, DiffSpan, Language, RunaError,
    ThemeMode, Tone, TranslateRequest, UiConfig, UiDensity, WindowToken, WritingContext,
};
use shared_utils::{detect_language, resolve_translate_target, save_config, word_diff};
use tauri::{AppHandle, Emitter, Manager, State};
use tracing::{debug, info};

use crate::state::AppState;

#[tauri::command]
#[specta::specta]
pub async fn download_model(state: State<'_, AppState>) -> Result<(), RunaError> {
    state.model_download.start()
}

#[tauri::command]
#[specta::specta]
pub async fn correct(
    state: State<'_, AppState>,
    text: String,
    tone: Tone,
    context: WritingContext,
) -> Result<u64, RunaError> {
    let generation_id = state.next_generation();
    state
        .ai
        .correct(CorrectRequest {
            text,
            tone,
            context,
            generation_id,
        })
        .await?;
    Ok(generation_id)
}

#[tauri::command]
#[specta::specta]
pub async fn translate(
    state: State<'_, AppState>,
    text: String,
    target: Language,
) -> Result<u64, RunaError> {
    let source = detect_language(&text);
    let resolved_target = resolve_translate_target(source, target);
    let generation_id = state.next_generation();
    state
        .ai
        .translate(TranslateRequest {
            text,
            source,
            target: resolved_target,
            generation_id,
        })
        .await?;
    Ok(generation_id)
}

#[tauri::command]
#[specta::specta]
pub async fn apply_replacement(
    app: AppHandle,
    state: State<'_, AppState>,
    text: String,
) -> Result<(), RunaError> {
    let origin = state.last_origin.read().await.clone();
    let _req = ApplyReplacementRequest {
        text: text.clone(),
        origin: origin.clone(),
    };

    // 1. Écrire le presse-papiers (échec → popup reste visible).
    state.stack.clipboard.write_text(&text).await?;
    debug!(len = text.len(), "clipboard préparé pour remplacement");

    // 2. Masquer la popup d'abord : avec WebView2 / alwaysOnTop, SetForegroundWindow
    //    échoue tant que la popup a le FG. Hide → Windows réactive souvent l'origine.
    if let Some(window) = app.get_webview_window("popup") {
        let _ = window.set_always_on_top(false);
        window.hide().map_err(|e| RunaError::Internal {
            message: format!("hide popup: {e}"),
        })?;
    }

    // Laisser l'OS réactiver la fenêtre précédente.
    tokio::time::sleep(std::time::Duration::from_millis(120)).await;

    // 3. Forcer le focus sur l'origine si besoin (best-effort : on paste quand même).
    if let Err(err) = state.stack.focus.restore(&origin).await {
        info!(?err, "restore focus best-effort échoué — tentative paste quand même");
    } else {
        tokio::time::sleep(std::time::Duration::from_millis(40)).await;
    }

    // 4. Injecter Ctrl+V.
    match state.stack.injector.paste_clipboard().await {
        Ok(()) => Ok(()),
        Err(RunaError::InjectionFallback { message }) => {
            // S5-6 : ne jamais logger le contenu presse-papiers en info (message = motif OS).
            info!("injection refusée — repli presse-papiers (détail en debug)");
            debug!(%message, "InjectionFallback");
            Err(RunaError::InjectionFallback { message })
        }
        Err(err) => {
            info!(kind = "other", "injection échouée — repli presse-papiers");
            debug!(?err, "injection erreur");
            Err(RunaError::InjectionFallback {
                message: format!("injection refusée — texte copié, collez avec Ctrl+V ({err})"),
            })
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn copy_text(state: State<'_, AppState>, text: String) -> Result<(), RunaError> {
    state.stack.clipboard.write_text(&text).await
}

#[tauri::command]
#[specta::specta]
pub async fn dismiss(app: AppHandle) -> Result<(), RunaError> {
    if let Some(window) = app.get_webview_window("popup") {
        window.hide().map_err(|e| RunaError::Internal {
            message: format!("hide popup: {e}"),
        })?;
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn cancel_inference(state: State<'_, AppState>) -> Result<(), RunaError> {
    state.ai.cancel().await
}

#[tauri::command]
#[specta::specta]
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, RunaError> {
    Ok(state.config.read().await.clone())
}

/// Active / désactive le lancement au démarrage de la session OS.
#[tauri::command]
#[specta::specta]
pub async fn set_launch_at_login(
    app: AppHandle,
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<bool, RunaError> {
    {
        let mut config = state.config.write().await;
        config.launch_at_login = enabled;
        save_config(&config)?;
    }
    apply_autostart(&app, enabled)?;
    Ok(enabled)
}

pub(crate) fn apply_autostart(app: &AppHandle, enabled: bool) -> Result<(), RunaError> {
    use tauri_plugin_autostart::ManagerExt;
    let launcher = app.autolaunch();
    let result = if enabled {
        launcher.enable()
    } else {
        launcher.disable()
    };
    result.map_err(|e| RunaError::Internal {
        message: format!("autostart: {e}"),
    })
}

/// Met à jour le thème UI, persiste le TOML et notifie toutes les fenêtres.
#[tauri::command]
#[specta::specta]
pub async fn set_ui_theme(
    app: AppHandle,
    state: State<'_, AppState>,
    theme_mode: ThemeMode,
    accent: AccentPreset,
    density: UiDensity,
) -> Result<UiConfig, RunaError> {
    let ui = {
        let mut config = state.config.write().await;
        config.ui.theme_mode = theme_mode;
        config.ui.accent = accent;
        config.ui.density = density;
        save_config(&config)?;
        config.ui.clone()
    };
    app.emit("ui_theme_changed", &ui)
        .map_err(|e| RunaError::Internal {
            message: format!("emit ui_theme_changed: {e}"),
        })?;
    Ok(ui)
}

#[tauri::command]
#[specta::specta]
pub async fn word_diff_cmd(before: String, after: String) -> Result<Vec<DiffSpan>, RunaError> {
    Ok(word_diff(&before, &after))
}

#[tauri::command]
#[specta::specta]
pub async fn has_replace_origin(state: State<'_, AppState>) -> Result<bool, RunaError> {
    let origin = state.last_origin.read().await.clone();
    Ok(matches!(origin, WindowToken::Windows { .. }))
}
