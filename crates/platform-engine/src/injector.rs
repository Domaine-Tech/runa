use async_trait::async_trait;
use shared_types::RunaError;

/// Port d'injection clavier (Ctrl+V / Cmd+V) après restauration du focus.
#[async_trait]
pub trait TextInjector: Send + Sync {
    /// Colle le contenu déjà présent dans le presse-papiers.
    async fn paste_clipboard(&self) -> Result<(), RunaError>;
}
