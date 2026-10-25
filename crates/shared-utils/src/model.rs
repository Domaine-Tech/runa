use std::path::PathBuf;

use directories::ProjectDirs;
use shared_types::RunaError;

/// Fichier GGUF par défaut (Qwen2.5-3B-Instruct Q4_K_M).
pub const DEFAULT_MODEL_FILENAME: &str = "qwen2.5-3b-instruct-q4_k_m.gguf";

/// URL Hugging Face (téléchargement explicite uniquement).
pub const DEFAULT_MODEL_URL: &str = "https://huggingface.co/Qwen/Qwen2.5-3B-Instruct-GGUF/resolve/main/qwen2.5-3b-instruct-q4_k_m.gguf";

/// Blake3 hex du GGUF officiel `qwen2.5-3b-instruct-q4_k_m.gguf`
/// (SHA256 HF `626b4a6678b86442240e33df819e00132d3ba7dddfe1cdc4fbb18e0a9615c62d`).
pub const DEFAULT_MODEL_BLAKE3: &str =
    "04a7a2e6f6ea3e59da99acf27aa0bdaf55cb197853c68420f9bdb78f8c2f71d1";

/// Répertoire `{data_dir}/models`.
pub fn models_dir() -> Result<PathBuf, RunaError> {
    let dirs = ProjectDirs::from("app", "runa", "Runa").ok_or_else(|| RunaError::Config {
        message: "impossible de résoudre le répertoire data".into(),
    })?;
    Ok(dirs.data_dir().join("models"))
}

/// Chemin par défaut du GGUF local.
pub fn default_model_path() -> Result<PathBuf, RunaError> {
    Ok(models_dir()?.join(DEFAULT_MODEL_FILENAME))
}
