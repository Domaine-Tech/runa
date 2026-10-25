use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;

/// Erreur unifiée de l'application. Seule surface d'erreur publique inter-crates et IPC.
#[derive(Debug, Clone, Error, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RunaError {
    #[error("opération non supportée sur cette plateforme ou configuration: {message}")]
    Unsupported { message: String },

    #[error("permission plateforme manquante: {message}")]
    PlatformPermission { message: String },

    #[error("presse-papiers: {message}")]
    Clipboard { message: String },

    #[error("focus fenêtre: {message}")]
    WindowFocus { message: String },

    #[error("injection clavier refusée; texte laissé dans le presse-papiers: {message}")]
    InjectionFallback { message: String },

    #[error("moteur IA: {message}")]
    AiEngine { message: String },

    #[error("modèle indisponible: {message}")]
    ModelUnavailable { message: String },

    #[error("configuration: {message}")]
    Config { message: String },

    #[error("entrée invalide: {message}")]
    InvalidInput { message: String },

    #[error("annulé")]
    Cancelled,

    #[error("interne: {message}")]
    Internal { message: String },
}

impl RunaError {
    pub fn unsupported(msg: impl Into<String>) -> Self {
        Self::Unsupported {
            message: msg.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runa_error_serde_kind_cancelled() {
        let err = RunaError::Cancelled;
        let json = serde_json::to_string(&err).expect("serialize");
        assert_eq!(json, r#"{"kind":"cancelled"}"#);
        let back: RunaError = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(back, RunaError::Cancelled));
    }

    #[test]
    fn runa_error_serde_kind_clipboard() {
        let err = RunaError::Clipboard {
            message: "lock".into(),
        };
        let json = serde_json::to_string(&err).expect("serialize");
        assert!(json.contains(r#""kind":"clipboard""#));
        assert!(json.contains("lock"));
        let back: RunaError = serde_json::from_str(&json).expect("deserialize");
        match back {
            RunaError::Clipboard { message } => assert_eq!(message, "lock"),
            other => panic!("attendu Clipboard, reçu {other:?}"),
        }
    }

    #[test]
    fn runa_error_all_kinds_serde_roundtrip_for_ipc() {
        let cases: Vec<(RunaError, &str)> = vec![
            (
                RunaError::Unsupported {
                    message: "m".into(),
                },
                "unsupported",
            ),
            (
                RunaError::PlatformPermission {
                    message: "m".into(),
                },
                "platform_permission",
            ),
            (
                RunaError::Clipboard {
                    message: "m".into(),
                },
                "clipboard",
            ),
            (
                RunaError::WindowFocus {
                    message: "m".into(),
                },
                "window_focus",
            ),
            (
                RunaError::InjectionFallback {
                    message: "m".into(),
                },
                "injection_fallback",
            ),
            (
                RunaError::AiEngine {
                    message: "m".into(),
                },
                "ai_engine",
            ),
            (
                RunaError::ModelUnavailable {
                    message: "m".into(),
                },
                "model_unavailable",
            ),
            (
                RunaError::Config {
                    message: "m".into(),
                },
                "config",
            ),
            (
                RunaError::InvalidInput {
                    message: "m".into(),
                },
                "invalid_input",
            ),
            (RunaError::Cancelled, "cancelled"),
            (
                RunaError::Internal {
                    message: "m".into(),
                },
                "internal",
            ),
        ];

        for (err, kind) in cases {
            let json = serde_json::to_string(&err).expect("serialize");
            assert!(
                json.contains(&format!(r#""kind":"{kind}""#)),
                "json={json} kind={kind}"
            );
            let back: RunaError = serde_json::from_str(&json).expect("deserialize");
            let back_json = serde_json::to_string(&back).expect("re-serialize");
            assert_eq!(json, back_json, "round-trip kind={kind}");
        }
    }
}
