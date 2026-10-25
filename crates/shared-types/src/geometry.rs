use serde::{Deserialize, Serialize};
use specta::Type;

/// Point écran en pixels physiques (origine haut-gauche).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ScreenPoint {
    pub x: i32,
    pub y: i32,
}
