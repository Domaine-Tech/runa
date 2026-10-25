use shared_types::Language;
use whatlang::detect;

/// Heuristique locale de langue source (avant appel LLM traduction).
pub fn detect_language(text: &str) -> Language {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Language::Auto;
    }

    match detect(trimmed).map(|info| info.lang()) {
        Some(whatlang::Lang::Fra) => Language::Fr,
        Some(whatlang::Lang::Eng) => Language::En,
        Some(whatlang::Lang::Spa) => Language::Es,
        Some(whatlang::Lang::Deu) => Language::De,
        _ => Language::Auto,
    }
}

/// Résout la langue cible lorsque l’UI demande `Auto` (bascule FR↔EN, sinon EN).
pub fn resolve_translate_target(source: Language, target: Language) -> Language {
    match (source, target) {
        (Language::Fr, Language::Auto) => Language::En,
        (Language::En, Language::Auto) => Language::Fr,
        (_, Language::Auto) => Language::En,
        (_, t) => t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_language_empty_is_auto() {
        assert_eq!(detect_language(""), Language::Auto);
        assert_eq!(detect_language("   \n\t  "), Language::Auto);
    }

    #[test]
    fn detect_language_french_sample() {
        let text = "Bonjour, je voudrais corriger cette phrase en français s'il vous plaît.";
        assert_eq!(detect_language(text), Language::Fr);
    }

    #[test]
    fn detect_language_unknown_falls_back_auto() {
        // Chiffres / bruit : whatlang ne mappe pas vers Fr/En/Es/De.
        assert_eq!(detect_language("12345 67890 !!!"), Language::Auto);
    }

    #[test]
    fn resolve_translate_target_auto_fr_to_en() {
        assert_eq!(
            resolve_translate_target(Language::Fr, Language::Auto),
            Language::En
        );
    }

    #[test]
    fn resolve_translate_target_auto_en_to_fr() {
        assert_eq!(
            resolve_translate_target(Language::En, Language::Auto),
            Language::Fr
        );
    }

    #[test]
    fn resolve_translate_target_explicit_preserved() {
        assert_eq!(
            resolve_translate_target(Language::Fr, Language::Es),
            Language::Es
        );
    }
}
