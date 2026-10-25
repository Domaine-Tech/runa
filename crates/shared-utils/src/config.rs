use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;
use shared_types::{AppConfig, RunaError};

const QUALIFIER: &str = "app";
const ORGANIZATION: &str = "runa";
const APPLICATION: &str = "Runa";

pub fn config_path() -> Result<PathBuf, RunaError> {
    let dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION).ok_or_else(|| {
        RunaError::Config {
            message: "impossible de résoudre le répertoire de configuration".into(),
        }
    })?;
    Ok(dirs.config_dir().join("config.toml"))
}

pub fn load_config() -> Result<AppConfig, RunaError> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let raw = fs::read_to_string(&path).map_err(|e| RunaError::Config {
        message: format!("lecture {}: {e}", path.display()),
    })?;
    toml::from_str(&raw).map_err(|e| RunaError::Config {
        message: format!("parse TOML: {e}"),
    })
}

pub fn save_config(config: &AppConfig) -> Result<(), RunaError> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| RunaError::Config {
            message: format!("mkdir {}: {e}", parent.display()),
        })?;
    }
    let raw = toml::to_string_pretty(config).map_err(|e| RunaError::Config {
        message: format!("serialize TOML: {e}"),
    })?;
    fs::write(&path, raw).map_err(|e| RunaError::Config {
        message: format!("écriture {}: {e}", path.display()),
    })
}
