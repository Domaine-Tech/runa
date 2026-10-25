use shared_types::{Language, Tone, WritingContext};

/// Prompt système pour correction / reformulation (ton + contexte).
pub fn correction_system(tone: Tone, context: WritingContext) -> String {
    format!(
        "Tu es un assistant de rédaction local. Corrige et reformule le texte utilisateur.\n\
         \n\
         Contexte : {context_label}.\n\
         {context_rules}\n\
         \n\
         Ton : {tone_label}.\n\
         {tone_rules}\n\
         \n\
         Règles communes :\n\
         - Conserve la langue source.\n\
         - Ne renvoie que le texte corrigé, sans guillemets, sans commentaire, sans markdown.\n\
         - Ne change pas le sens ni les faits.",
        context_label = context.as_prompt_label(),
        context_rules = context_instructions(context),
        tone_label = tone.as_prompt_label(),
        tone_rules = tone_instructions(tone),
    )
}

fn context_instructions(context: WritingContext) -> &'static str {
    match context {
        WritingContext::Mail => {
            "- Canal e-mail professionnel ou perso.\n\
             - Phrases complètes ; conserve ou adapte poliment les salutations / formules de clôture si déjà présentes.\n\
             - Pas de jargon de messagerie instantanée ni d'abréviations type Slack."
        }
        WritingContext::Chat => {
            "- Canal chat (Slack, Teams, SMS, Discord).\n\
             - Message court et direct ; une idée par phrase si possible.\n\
             - Pas de formule de courrier (« Cordialement », long préambule)."
        }
        WritingContext::Note => {
            "- Note personnelle, mémo ou todo.\n\
             - Style clair et naturel ; listes et tirets autorisés si utiles.\n\
             - Pas de formalisme mail ni de signature."
        }
    }
}

fn tone_instructions(tone: Tone) -> &'static str {
    match tone {
        Tone::Formal => {
            "- Registre soutenu, précis, professionnel.\n\
             - Conserve le vouvoiement s'il est déjà utilisé ; sinon reste neutre et poli.\n\
             - Évite le langage familier et les contractions trop orales."
        }
        Tone::Concise => {
            "- Coupe le superflu, les répétitions et les formulations creuses.\n\
             - Garde toutes les informations utiles ; phrases plus courtes.\n\
             - Pas de fioritures ni d'introductions inutiles."
        }
        Tone::Fluid => {
            "- Naturel, rythmé, agréable à lire à voix haute.\n\
             - Transitions souples ; orthographe et grammaire soignées.\n\
             - Ni trop raide, ni télégraphique."
        }
    }
}

/// Prompt système pour traduction.
pub fn translation_system(source: Language, target: Language) -> String {
    format!(
        "Tu es un traducteur local. Traduis le texte vers {}.\n\
         Langue source détectée : {}.\n\
         Règles :\n\
         - Ne renvoie que la traduction, sans guillemets, sans commentaire, sans markdown.\n\
         - Conserve le ton et les sauts de ligne utiles.",
        target.as_prompt_label(),
        source.as_prompt_label()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompts_include_tone_context_and_langs() {
        for tone in [Tone::Formal, Tone::Concise, Tone::Fluid] {
            let c = correction_system(tone, WritingContext::Note);
            assert!(
                c.contains(tone.as_prompt_label()),
                "missing tone {}",
                tone.as_prompt_label()
            );
        }
        for ctx in [
            WritingContext::Mail,
            WritingContext::Chat,
            WritingContext::Note,
        ] {
            let c = correction_system(Tone::Fluid, ctx);
            assert!(
                c.contains(ctx.as_prompt_label()),
                "missing context {}",
                ctx.as_prompt_label()
            );
        }
        let mail_formal = correction_system(Tone::Formal, WritingContext::Mail);
        assert!(mail_formal.contains("e-mail") || mail_formal.contains("Mail"));
        assert!(mail_formal.contains("soutenu") || mail_formal.contains("Formel"));

        let chat_concise = correction_system(Tone::Concise, WritingContext::Chat);
        assert!(chat_concise.contains("Chat") || chat_concise.contains("Slack"));
        assert!(chat_concise.contains("superflu") || chat_concise.contains("Concis"));

        let t = translation_system(Language::Fr, Language::En);
        assert!(t.contains("anglais"));
        assert!(t.contains("français"));
    }

    #[test]
    fn prompts_forbid_markdown_instruction() {
        let correction = correction_system(Tone::Fluid, WritingContext::Note);
        assert!(
            correction.to_lowercase().contains("sans markdown"),
            "correction doit interdire le markdown: {correction}"
        );
        let translation = translation_system(Language::Fr, Language::En);
        assert!(
            translation.to_lowercase().contains("sans markdown"),
            "translation doit interdire le markdown: {translation}"
        );
    }
}
