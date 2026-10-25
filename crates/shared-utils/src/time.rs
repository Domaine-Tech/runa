use std::time::{Duration, Instant};

/// Horodatage monotone pour le détecteur de séquence (testable).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstantMillis(u64);

impl InstantMillis {
    pub const fn from_millis(ms: u64) -> Self {
        Self(ms)
    }

    pub fn now() -> Self {
        // Epoch relative process-safe via Instant cache would be nicer;
        // for production impulses we use Instant-based conversion at call site.
        Self(instant_to_ms(Instant::now()))
    }

    pub const fn as_millis(self) -> u64 {
        self.0
    }
}

fn instant_to_ms(instant: Instant) -> u64 {
    // Instant n'a pas d'epoch absolue; on utilise une base lazy thread-local.
    use std::sync::OnceLock;
    static BASE: OnceLock<Instant> = OnceLock::new();
    let base = BASE.get_or_init(Instant::now);
    instant
        .checked_duration_since(*base)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64
}

pub fn elapsed_ms(start: InstantMillis, end: InstantMillis) -> u64 {
    end.as_millis().saturating_sub(start.as_millis())
}

/// Debounce temporel pour notifications clipboard (double notif / Empty+Set).
///
/// Toute notif dans `coalesce_ms` après la dernière impulsion émise est ignorée.
pub fn should_emit_impulse(
    last_emitted: bool,
    last_at: Instant,
    now: Instant,
    coalesce_ms: u64,
) -> bool {
    if !last_emitted {
        return true;
    }
    now.saturating_duration_since(last_at) >= Duration::from_millis(coalesce_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_impulse_always_emitted() {
        let t0 = Instant::now();
        assert!(should_emit_impulse(false, t0, t0, 40));
    }

    #[test]
    fn coalesces_within_window() {
        let t0 = Instant::now();
        let t1 = t0 + Duration::from_millis(10);
        assert!(!should_emit_impulse(true, t0, t1, 40));
    }

    #[test]
    fn emits_after_window() {
        let t0 = Instant::now();
        let t1 = t0 + Duration::from_millis(50);
        assert!(should_emit_impulse(true, t0, t1, 40));
    }
}
