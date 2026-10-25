/// Nettoie la sortie brute du LLM (fences, guillemets, JSON parasite).
pub fn trim_model_output(raw: &str) -> String {
    let mut text = raw.trim().to_string();
    if text.is_empty() {
        return text;
    }

    text = strip_markdown_fence(&text);
    text = text.trim().to_string();
    text = strip_wrapping_quotes(&text);
    text = text.trim().to_string();

    if let Some(extracted) = extract_json_text_field(&text) {
        text = extracted.trim().to_string();
        text = strip_wrapping_quotes(&text);
        text = text.trim().to_string();
    }

    text
}

fn strip_markdown_fence(text: &str) -> String {
    let trimmed = text.trim();
    if !trimmed.starts_with("```") {
        return trimmed.to_string();
    }
    let mut lines = trimmed.lines();
    let first = lines.next().unwrap_or("");
    if !first.starts_with("```") {
        return trimmed.to_string();
    }
    let mut body: Vec<&str> = lines.collect();
    if body
        .last()
        .is_some_and(|line| line.trim().starts_with("```"))
    {
        body.pop();
    }
    body.join("\n").trim().to_string()
}

fn strip_wrapping_quotes(text: &str) -> String {
    let t = text.trim();
    let bytes = t.as_bytes();
    if bytes.len() >= 2 {
        let first = bytes[0];
        let last = bytes[bytes.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return t[1..t.len() - 1].to_string();
        }
    }
    // Guillemets typographiques “…” (U+201C / U+201D)
    if t.starts_with('\u{201c}') && t.ends_with('\u{201d}') {
        let inner = t.trim_start_matches('\u{201c}').trim_end_matches('\u{201d}');
        return inner.to_string();
    }
    t.to_string()
}

fn extract_json_text_field(text: &str) -> Option<String> {
    let t = text.trim();
    if !(t.starts_with('{') && t.ends_with('}')) {
        return None;
    }
    let value: serde_json::Value = serde_json::from_str(t).ok()?;
    let obj = value.as_object()?;
    for key in ["text", "correction", "translation"] {
        if let Some(s) = obj.get(key).and_then(|v| v.as_str()) {
            return Some(s.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_unchanged() {
        assert_eq!(trim_model_output("Bonjour le monde"), "Bonjour le monde");
    }

    #[test]
    fn strips_fences_and_quotes() {
        let raw = "```\n\"Hello\"\n```";
        assert_eq!(trim_model_output(raw), "Hello");
    }

    #[test]
    fn extracts_json_correction() {
        let raw = r#"{"correction": "Texte corrigé."}"#;
        assert_eq!(trim_model_output(raw), "Texte corrigé.");
    }

    #[test]
    fn extracts_json_translation() {
        let raw = "```json\n{\"translation\": \"Hello\"}\n```";
        assert_eq!(trim_model_output(raw), "Hello");
    }
}
