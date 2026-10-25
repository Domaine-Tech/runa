//! Téléchargement explicite du GGUF (thread dédié, hors llm-worker).

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use shared_types::{ModelDownloadEvent, RunaError};
use shared_utils::{
    blake3_file, default_model_path, models_dir, DEFAULT_MODEL_BLAKE3, DEFAULT_MODEL_URL,
};
use tokio::sync::broadcast;
use tracing::{info, warn};

static DOWNLOAD_IN_FLIGHT: AtomicBool = AtomicBool::new(false);

pub struct ModelDownloadHub {
    events: broadcast::Sender<ModelDownloadEvent>,
}

impl ModelDownloadHub {
    pub fn new() -> Self {
        let (events, _) = broadcast::channel(16);
        Self { events }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ModelDownloadEvent> {
        self.events.subscribe()
    }

    pub fn start(&self) -> Result<(), RunaError> {
        if DOWNLOAD_IN_FLIGHT
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Err(RunaError::InvalidInput {
                message: "téléchargement déjà en cours".into(),
            });
        }

        let dest = default_model_path()?;
        let events = self.events.clone();

        if let Err(err) = std::thread::Builder::new()
            .name("model-download".into())
            .spawn(move || {
                let result = download_model_file(&dest, &events);
                if let Err(err) = result {
                    warn!(?err, "téléchargement modèle échoué");
                    let _ = events.send(ModelDownloadEvent::Error { error: err });
                }
                DOWNLOAD_IN_FLIGHT.store(false, Ordering::SeqCst);
            })
        {
            DOWNLOAD_IN_FLIGHT.store(false, Ordering::SeqCst);
            return Err(RunaError::Internal {
                message: format!("spawn model-download: {err}"),
            });
        }

        Ok(())
    }
}

fn download_model_file(
    dest: &Path,
    events: &broadcast::Sender<ModelDownloadEvent>,
) -> Result<(), RunaError> {
    if dest.is_file() {
        let hash = blake3_file(dest)?;
        if hash == DEFAULT_MODEL_BLAKE3 {
            info!(path = %dest.display(), "modèle déjà présent (blake3 OK)");
            let _ = events.send(ModelDownloadEvent::Done {
                path: dest.display().to_string(),
            });
            return Ok(());
        }
        warn!(
            path = %dest.display(),
            "fichier présent mais hash blake3 inattendu — retéléchargement"
        );
    }

    let dir = models_dir()?;
    fs::create_dir_all(&dir).map_err(|e| RunaError::Internal {
        message: format!("mkdir models: {e}"),
    })?;

    let partial = partial_path(dest);
    if partial.exists() {
        let _ = fs::remove_file(&partial);
    }

    // Timeout global du GET (modèle ~2 Go) — évite un hang infini.
    const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(45 * 60);

    let client = reqwest::blocking::Client::builder()
        .timeout(DOWNLOAD_TIMEOUT)
        .build()
        .map_err(|e| RunaError::Internal {
            message: format!("client http: {e}"),
        })?;

    let mut response = client.get(DEFAULT_MODEL_URL).send().map_err(|e| {
        if e.is_timeout() {
            RunaError::ModelUnavailable {
                message: format!(
                    "téléchargement modèle interrompu (timeout {} min)",
                    DOWNLOAD_TIMEOUT.as_secs() / 60
                ),
            }
        } else {
            RunaError::Internal {
                message: format!("GET modèle: {e}"),
            }
        }
    })?;

    if !response.status().is_success() {
        return Err(RunaError::Internal {
            message: format!("HTTP {} pour le modèle", response.status()),
        });
    }

    let total = response.content_length();
    let _ = events.send(ModelDownloadEvent::Started {
        total_bytes: total,
    });

    let mut file = File::create(&partial).map_err(|e| RunaError::Internal {
        message: format!("création {}: {e}", partial.display()),
    })?;

    let mut hasher = blake3::Hasher::new();
    let mut buf = [0_u8; 64 * 1024];
    let mut downloaded: u64 = 0;
    let mut last_emit = 0_u64;

    loop {
        let n = response.read(&mut buf).map_err(|e| RunaError::Internal {
            message: format!("lecture flux: {e}"),
        })?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n]).map_err(|e| RunaError::Internal {
            message: format!("écriture partial: {e}"),
        })?;
        hasher.update(&buf[..n]);
        downloaded += n as u64;

        if downloaded.saturating_sub(last_emit) >= 256 * 1024 || Some(downloaded) == total {
            let _ = events.send(ModelDownloadEvent::Progress {
                downloaded_bytes: downloaded,
                total_bytes: total,
            });
            last_emit = downloaded;
        }
    }

    file.flush().map_err(|e| RunaError::Internal {
        message: format!("flush partial: {e}"),
    })?;
    drop(file);

    let hash = hasher.finalize().to_hex().to_string();
    if hash != DEFAULT_MODEL_BLAKE3 {
        let _ = fs::remove_file(&partial);
        return Err(RunaError::ModelUnavailable {
            message: format!(
                "hash blake3 invalide (obtenu {hash}, attendu {DEFAULT_MODEL_BLAKE3})"
            ),
        });
    }

    if dest.exists() {
        let _ = fs::remove_file(dest);
    }
    fs::rename(&partial, dest).map_err(|e| RunaError::Internal {
        message: format!("rename modèle: {e}"),
    })?;

    info!(path = %dest.display(), "modèle téléchargé et vérifié (blake3)");
    let _ = events.send(ModelDownloadEvent::Done {
        path: dest.display().to_string(),
    });
    Ok(())
}

fn partial_path(dest: &Path) -> PathBuf {
    let mut p = dest.as_os_str().to_os_string();
    p.push(".partial");
    PathBuf::from(p)
}

