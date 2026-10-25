mod commands;
mod model_download;
mod state;
mod updater;

use std::sync::Arc;
use std::time::Instant;

use ai_engine::create_ai_engine;
use parking_lot::RwLock as SyncRwLock;
use platform_engine::{create_platform_stack, create_trigger_engine};
use shared_types::{AppConfig, InferenceEvent, ModelDownloadEvent, ScreenPoint, TriggerEvent};
use shared_utils::{init_tracing, load_config};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, PhysicalPosition, RunEvent, WebviewWindow, WindowEvent,
};
use tauri_specta::{collect_commands, Builder as SpectaBuilder};
use tracing::{error, info, warn};

use crate::commands::{
    apply_replacement, cancel_inference, copy_text, correct, dismiss, download_model, get_config,
    has_replace_origin, set_ui_theme, translate, word_diff_cmd,
};
use crate::model_download::ModelDownloadHub;
use crate::state::AppState;

#[cfg(debug_assertions)]
use specta_typescript::{BigIntExportBehavior, Typescript};

const EVENT_TRIGGER: &str = "trigger_fired";
const EVENT_INFERENCE: &str = "inference";
const EVENT_MODEL_DOWNLOAD: &str = "model_download";

pub fn run() {
    let _ = init_tracing("info,runa_desktop=debug");

    let config = load_config().unwrap_or_else(|err| {
        warn!(?err, "config absente ou invalide — défauts");
        AppConfig::default()
    });

    let specta = SpectaBuilder::<tauri::Wry>::new().commands(collect_commands![
        correct,
        translate,
        apply_replacement,
        copy_text,
        dismiss,
        cancel_inference,
        get_config,
        set_ui_theme,
        word_diff_cmd,
        has_replace_origin,
        download_model,
    ]);

    #[cfg(debug_assertions)]
    {
        let bindings_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../src/lib/bindings.ts");
        let tmp_path = bindings_path.with_extension("ts.tmp");
        match specta.export(
            Typescript::default()
                .header("// Généré par tauri-specta — ne pas éditer à la main\n")
                .bigint(BigIntExportBehavior::Number),
            &tmp_path,
        ) {
            Ok(()) => match std::fs::metadata(&tmp_path) {
                Ok(meta) if meta.len() > 0 => {
                    if let Err(err) = std::fs::rename(&tmp_path, &bindings_path) {
                        warn!(?err, "rename bindings.tmp échoué");
                        let _ = std::fs::remove_file(&tmp_path);
                    } else {
                        info!(path = %bindings_path.display(), "bindings specta exportés");
                    }
                }
                Ok(_) => {
                    warn!("export specta vide — conservation des bindings existants");
                    let _ = std::fs::remove_file(&tmp_path);
                }
                Err(err) => {
                    warn!(?err, "métadonnées bindings.tmp illisibles");
                    let _ = std::fs::remove_file(&tmp_path);
                }
            },
            Err(err) => {
                warn!(?err, "export specta bindings échoué");
                let _ = std::fs::remove_file(&tmp_path);
            }
        }
    }

    let stack = match create_platform_stack(&config.trigger) {
        Ok(s) => Arc::new(s),
        Err(err) => {
            error!(?err, "plateforme non supportée");
            std::process::exit(1);
        }
    };

    let trigger = create_trigger_engine(Arc::clone(&stack), config.clone());
    let ai_config = Arc::new(SyncRwLock::new(config.ai.clone()));
    let model_download = ModelDownloadHub::new();

    #[cfg(feature = "llama")]
    let ai = match ai_engine::create_llama_engine(Arc::clone(&ai_config)) {
        Ok(engine) => engine,
        Err(err) => {
            warn!(?err, "moteur llama indisponible — stub Unsupported");
            create_ai_engine(Arc::clone(&ai_config))
        }
    };
    #[cfg(not(feature = "llama"))]
    let ai = create_ai_engine(Arc::clone(&ai_config));

    let state = AppState::new(
        config,
        Arc::clone(&ai_config),
        Arc::clone(&stack),
        Arc::clone(&trigger),
        Arc::clone(&ai),
        model_download,
    );

    let trigger_for_setup = Arc::clone(&trigger);
    let ai_for_setup = Arc::clone(&ai);
    let download_rx = state.model_download.subscribe();

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(specta.invoke_handler())
        .setup(move |app| {
            spawn_event_bridges(
                app.handle().clone(),
                trigger_for_setup.subscribe(),
                ai_for_setup.subscribe(),
                download_rx,
            );

            if let Err(err) = tauri::async_runtime::block_on(trigger_for_setup.start()) {
                error!(?err, "trigger engine non démarré — tray actif sans Ctrl+C+C");
            }

            setup_tray(app.handle())?;
            updater::spawn_startup_check(app.handle().clone());
            info!("Runa prêt (popup + main préchargées, visible=false)");
            Ok(())
        })
        .on_window_event(|window, event| {
            let label = window.label();
            match event {
                WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    let _ = window.hide();
                }
                WindowEvent::Focused(false) if label == "popup" => {
                    let allow = window
                        .app_handle()
                        .try_state::<AppState>()
                        .map(|s| s.should_hide_on_blur())
                        .unwrap_or(true);
                    if allow {
                        let _ = window.hide();
                    }
                }
                _ => {}
            }
        });

    match builder.build(tauri::generate_context!()) {
        Ok(app) => {
            app.run(|app_handle, event| {
                if let RunEvent::Exit = event {
                    if let Some(state) = app_handle.try_state::<AppState>() {
                        if let Err(err) = tauri::async_runtime::block_on(state.trigger.stop()) {
                            warn!(?err, "arrêt trigger engine échoué");
                        }
                    }
                }
            });
        }
        Err(err) => {
            error!(?err, "erreur fatale au démarrage Tauri");
            std::process::exit(1);
        }
    }
}

fn spawn_event_bridges(
    app: AppHandle,
    mut triggers: tokio::sync::broadcast::Receiver<TriggerEvent>,
    mut inference: tokio::sync::broadcast::Receiver<InferenceEvent>,
    mut downloads: tokio::sync::broadcast::Receiver<ModelDownloadEvent>,
) {
    let app_trigger = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            match triggers.recv().await {
                Ok(event) => {
                    let bridge_at = Instant::now();
                    let text_len = event.clipboard_text.len();
                    let is_preview = text_len == 0;
                    info!(
                        text_len,
                        sequence_ms = event.sequence_ms,
                        is_preview,
                        "bridge: TriggerEvent reçu"
                    );
                    if let Some(state) = app_trigger.try_state::<AppState>() {
                        *state.last_origin.write().await = event.origin.clone();
                    }
                    if let Some(state) = app_trigger.try_state::<AppState>() {
                        state.mark_popup_shown();
                    }
                    if let Some(window) = app_trigger.get_webview_window("popup") {
                        if is_preview {
                            // Show uniquement sur la preview (budget < 50 ms).
                            if let Err(err) = window.show() {
                                warn!(?err, "show popup échoué");
                            }
                            let show_ms = bridge_at.elapsed().as_millis() as u64;
                            info!(show_ms, "bridge: popup show() preview");
                            position_popup_near_caret(&window, event.caret.as_ref());
                            if let Err(err) = window.set_focus() {
                                warn!(?err, "set_focus popup échoué");
                            }
                            let _ = window.set_always_on_top(true);
                        } else if event.caret.is_some() {
                            // Hydraté : repositionner si caret dispo (capturé hors chemin show).
                            position_popup_near_caret(&window, event.caret.as_ref());
                        }
                    } else {
                        warn!("fenêtre popup introuvable au TriggerEvent");
                    }
                    if let Err(err) = app_trigger.emit(EVENT_TRIGGER, &event) {
                        warn!(?err, "emit trigger_fired échoué");
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    warn!(skipped = n, "bridge trigger en retard (événements perdus)");
                    continue;
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    let app_inf = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            match inference.recv().await {
                Ok(event) => {
                    let _ = app_inf.emit(EVENT_INFERENCE, event);
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    let app_dl = app;
    tauri::async_runtime::spawn(async move {
        loop {
            match downloads.recv().await {
                Ok(event) => {
                    if let ModelDownloadEvent::Done { ref path } = event {
                        if let Some(state) = app_dl.try_state::<AppState>() {
                            {
                                let mut ai = state.ai_config.write();
                                ai.model_path = path.clone();
                            }
                            if let Err(err) = state.persist_ai_from_mirror().await {
                                warn!(?err, "persistance model_path échouée");
                            }
                        }
                    }
                    let _ = app_dl.emit(EVENT_MODEL_DOWNLOAD, event);
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });
}

/// Place la popup au-dessus du caret ; repli en dessous puis centre moniteur.
fn position_popup_near_caret(window: &WebviewWindow, caret: Option<&ScreenPoint>) {
    let Ok(size) = window.outer_size() else {
        let _ = window.center();
        return;
    };
    let width = i32::try_from(size.width).unwrap_or(420);
    let height = i32::try_from(size.height).unwrap_or(360);
    let gap = 12;

    let Some(caret) = caret else {
        let _ = window.center();
        return;
    };

    let mut x = caret.x - width / 2;
    let mut y = caret.y - height - gap;

    if let Ok(Some(monitor)) = window.current_monitor() {
        let mpos = monitor.position();
        let msize = monitor.size();
        let left = mpos.x;
        let top = mpos.y;
        let right = left + i32::try_from(msize.width).unwrap_or(1920);
        let bottom = top + i32::try_from(msize.height).unwrap_or(1080);

        if y < top {
            y = caret.y + gap;
        }
        if y + height > bottom {
            y = (bottom - height).max(top);
        }
        x = x.clamp(left, (right - width).max(left));
    }

    if let Err(err) = window.set_position(PhysicalPosition::new(x, y)) {
        warn!(?err, "set_position popup échoué — centre");
        let _ = window.center();
    }
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    } else {
        warn!("fenêtre main introuvable");
    }
}

fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let quit = MenuItem::with_id(app, "quit", "Quitter", true, None::<&str>)?;
    let show = MenuItem::with_id(app, "show", "Afficher", true, None::<&str>)?;
    let check_update =
        MenuItem::with_id(app, "check_update", "Vérifier les mises à jour", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &check_update, &quit])?;

    let icon = Image::from_bytes(include_bytes!("../icons/icon.png"))?;

    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .tooltip("Runa")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => app.exit(0),
            "show" => show_main_window(app),
            "check_update" => updater::spawn_tray_check(app.clone()),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}
