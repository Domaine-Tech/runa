use similar::{ChangeTag, TextDiff};
use shared_types::{DiffKind, DiffSpan};

/// Diff mot-à-mot pour l'affichage de correction.
pub fn word_diff(before: &str, after: &str) -> Vec<DiffSpan> {
    let diff = TextDiff::from_words(before, after);
    let mut spans = Vec::new();

    for change in diff.iter_all_changes() {
        let kind = match change.tag() {
            ChangeTag::Equal => DiffKind::Equal,
            ChangeTag::Insert => DiffKind::Insert,
            ChangeTag::Delete => DiffKind::Delete,
        };
        spans.push(DiffSpan {
            kind,
            // `Display` de `Change` ajoute un `\n` — utiliser `value()` pour garder la phrase inline.
            text: change.value().to_string(),
        });
    }

    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_insertion() {
        let spans = word_diff("bonjour", "bonjour monde");
        assert!(spans.iter().any(|s| s.kind == DiffKind::Insert));
    }

    #[test]
    fn spans_have_no_trailing_newlines() {
        let spans = word_diff("je veu", "Je veux");
        for span in &spans {
            assert!(
                !span.text.contains('\n'),
                "span ne doit pas contenir de newline: {:?}",
                span.text
            );
        }
        let joined: String = spans.iter().map(|s| s.text.as_str()).collect();
        assert!(!joined.contains('\n'));
    }

    #[test]
    fn word_diff_marks_delete_and_equal() {
        let spans = word_diff("bonjour monde", "bonjour");
        assert!(
            spans.iter().any(|s| s.kind == DiffKind::Delete),
            "attendu un span Delete: {spans:?}"
        );
        assert!(
            spans.iter().any(|s| s.kind == DiffKind::Equal),
            "attendu un span Equal: {spans:?}"
        );
        let first_equal = spans.iter().position(|s| s.kind == DiffKind::Equal);
        let first_delete = spans.iter().position(|s| s.kind == DiffKind::Delete);
        assert!(
            first_equal.is_some() && first_delete.is_some() && first_equal < first_delete,
            "Equal avant Delete attendu: {spans:?}"
        );
    }
}
