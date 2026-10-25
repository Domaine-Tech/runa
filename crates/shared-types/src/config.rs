use serde::{Deserialize, Serialize};
use specta::Type;

fn default_launch_at_login() -> bool {
    true
}

/// Configuration applicative (chargée depuis TOML via `shared-utils`).
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AppConfig {
    pub trigger: TriggerConfig,
    pub ai: AiConfig,
    pub ui: UiConfig,
    /// Lancer Runa au démarrage de la session OS (défaut : oui).
    #[serde(default = "default_launch_at_login")]
    pub launch_at_login: bool,
}

fn default_linux_portal_shortcut() -> String {
    "CTRL+SHIFT+SPACE".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TriggerConfig {
    /// Fenêtre maximale entre deux impulsions (ms).
    pub sequence_window_ms: u64,
    /// Nombre d'impulsions requises.
    pub required_hits: u8,
    /// Polling macOS changeCount (ms).
    pub macos_poll_ms: u64,
    /// Active le fallback evdev sur Linux (opt-in).
    pub linux_evdev_opt_in: bool,
    /// Raccourci alternatif ashpd GlobalShortcuts (≠ Ctrl+C).
    #[serde(default = "default_linux_portal_shortcut")]
    pub linux_portal_shortcut: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AiConfig {
    pub model_path: String,
    pub n_ctx: u32,
    /// −1 = auto GPU layers.
    pub n_gpu_layers: i32,
    /// `None` = modèle résident. `#[serde(default)]` : configs TOML
    /// sans la clé (toml 0.8 omet `None` à l’écriture) restent valides.
    #[serde(default)]
    pub keep_alive_ms: Option<u64>,
    pub max_tokens: u32,
    pub temperature_correct: f32,
    pub temperature_translate: f32,
}

/// Mode d'apparence (clair / sombre / système).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    System,
    #[default]
    Dark,
    Light,
}

/// Accent de marque (presets, pas de couleur libre).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "snake_case")]
pub enum AccentPreset {
    #[default]
    Indigo,
    Slate,
    Emerald,
    Amber,
}

/// Densité de l'UI (la popup force toujours `compact` côté front).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "snake_case")]
pub enum UiDensity {
    Compact,
    #[default]
    Comfortable,
}

/// `#[serde(default)]` au niveau struct : configs TOML sans les clés thème
/// restent valides (remplies via `Default`) ; specta exporte des champs requis.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct UiConfig {
    pub popup_width: u32,
    pub popup_height: u32,
    pub theme_mode: ThemeMode,
    pub accent: AccentPreset,
    pub density: UiDensity,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            popup_width: 420,
            popup_height: 360,
            theme_mode: ThemeMode::Dark,
            accent: AccentPreset::Indigo,
            density: UiDensity::Comfortable,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            trigger: TriggerConfig {
                sequence_window_ms: 400,
                required_hits: 2,
                macos_poll_ms: 75,
                linux_evdev_opt_in: false,
                linux_portal_shortcut: default_linux_portal_shortcut(),
            },
            ai: AiConfig {
                model_path: String::new(),
                n_ctx: 2048,
                n_gpu_layers: -1,
                keep_alive_ms: None,
                max_tokens: 512,
                temperature_correct: 0.3,
                temperature_translate: 0.2,
            },
            ui: UiConfig::default(),
            launch_at_login: true,
        }
    }
}
