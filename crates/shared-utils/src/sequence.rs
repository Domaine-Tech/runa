use crate::time::{elapsed_ms, InstantMillis};

/// Résultat d'une impulsion enregistrée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SequenceOutcome {
    /// Impulsion enregistrée, séquence incomplete.
    Armed { hits: u8 },
    /// Séquence complète (ex. 2 hits en < window_ms).
    Fired { sequence_ms: u64 },
}

/// Détecteur de séquence d'impulsions (Ctrl+C+C).
///
/// Compte les notifications même si le contenu presse-papiers est identique.
#[derive(Debug, Clone)]
pub struct SequenceDetector {
    window_ms: u64,
    required_hits: u8,
    hits: u8,
    first_hit: Option<InstantMillis>,
    last_hit: Option<InstantMillis>,
}

impl SequenceDetector {
    pub fn new(window_ms: u64, required_hits: u8) -> Self {
        Self {
            window_ms,
            required_hits: required_hits.max(1),
            hits: 0,
            first_hit: None,
            last_hit: None,
        }
    }

    /// Constructeur pour Ctrl+C+C (2 hits / 400 ms).
    pub fn ctrl_c_c() -> Self {
        Self::new(400, 2)
    }

    pub fn reset(&mut self) {
        self.hits = 0;
        self.first_hit = None;
        self.last_hit = None;
    }

    pub fn record(&mut self, now: InstantMillis) -> SequenceOutcome {
        match self.first_hit {
            None => {
                self.hits = 1;
                self.first_hit = Some(now);
                self.last_hit = Some(now);
                SequenceOutcome::Armed { hits: 1 }
            }
            Some(first) => {
                let since_first = elapsed_ms(first, now);
                if since_first > self.window_ms {
                    self.hits = 1;
                    self.first_hit = Some(now);
                    self.last_hit = Some(now);
                    return SequenceOutcome::Armed { hits: 1 };
                }

                self.hits = self.hits.saturating_add(1);
                self.last_hit = Some(now);

                if self.hits >= self.required_hits {
                    let sequence_ms = since_first;
                    self.reset();
                    SequenceOutcome::Fired { sequence_ms }
                } else {
                    SequenceOutcome::Armed { hits: self.hits }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fires_on_two_hits_within_window() {
        let mut det = SequenceDetector::ctrl_c_c();
        let t0 = InstantMillis::from_millis(1_000);
        assert_eq!(det.record(t0), SequenceOutcome::Armed { hits: 1 });
        let t1 = InstantMillis::from_millis(1_250);
        assert_eq!(
            det.record(t1),
            SequenceOutcome::Fired { sequence_ms: 250 }
        );
    }

    #[test]
    fn resets_when_window_exceeded() {
        let mut det = SequenceDetector::ctrl_c_c();
        let t0 = InstantMillis::from_millis(0);
        assert_eq!(det.record(t0), SequenceOutcome::Armed { hits: 1 });
        let t1 = InstantMillis::from_millis(500);
        assert_eq!(det.record(t1), SequenceOutcome::Armed { hits: 1 });
        let t2 = InstantMillis::from_millis(700);
        assert_eq!(
            det.record(t2),
            SequenceOutcome::Fired { sequence_ms: 200 }
        );
    }

    #[test]
    fn counts_identical_content_impulses() {
        let mut det = SequenceDetector::ctrl_c_c();
        let t0 = InstantMillis::from_millis(10);
        let t1 = InstantMillis::from_millis(20);
        assert!(matches!(det.record(t0), SequenceOutcome::Armed { .. }));
        assert!(matches!(det.record(t1), SequenceOutcome::Fired { .. }));
    }

    #[test]
    fn sequence_boundary_at_window_ms_still_fires() {
        // `since_first > window_ms` → Armed ; égalité exacte → encore dans la fenêtre → Fired.
        let mut det = SequenceDetector::new(400, 2);
        let t0 = InstantMillis::from_millis(0);
        assert_eq!(det.record(t0), SequenceOutcome::Armed { hits: 1 });
        let t1 = InstantMillis::from_millis(400);
        assert_eq!(
            det.record(t1),
            SequenceOutcome::Fired { sequence_ms: 400 }
        );
    }

    #[test]
    fn sequence_three_hits_required() {
        let mut det = SequenceDetector::new(400, 3);
        let t0 = InstantMillis::from_millis(0);
        let t1 = InstantMillis::from_millis(50);
        let t2 = InstantMillis::from_millis(100);
        assert_eq!(det.record(t0), SequenceOutcome::Armed { hits: 1 });
        assert_eq!(det.record(t1), SequenceOutcome::Armed { hits: 2 });
        assert_eq!(
            det.record(t2),
            SequenceOutcome::Fired { sequence_ms: 100 }
        );
    }
}
