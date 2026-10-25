use async_trait::async_trait;
use shared_types::RunaError;
use tokio::sync::mpsc;

/// Impulsion de changement presse-papiers / entrée clavier filtrée.
#[derive(Debug, Clone)]
pub struct ClipboardImpulse {
    pub sequence_hint: Option<u64>,
    /// Si vrai : ouverture immédiate (raccourci portail), sans double-tap.
    pub immediate: bool,
}

/// Port de surveillance du presse-papiers.
#[async_trait]
pub trait ClipboardWatcher: Send + Sync {
    /// Démarre l'écoute et envoie des impulsions sur le canal.
    async fn start(&self, tx: mpsc::Sender<ClipboardImpulse>) -> Result<(), RunaError>;

    async fn stop(&self) -> Result<(), RunaError>;

    /// Lecture du texte au moment du déclenchement uniquement.
    async fn read_text(&self) -> Result<String, RunaError>;

    /// Écriture du texte (remplacement / copier). Peut supprimer les impulsions internes.
    async fn write_text(&self, text: &str) -> Result<(), RunaError>;
}
