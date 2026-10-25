# Backlog — Runa

File d’attente des **features / améliorations** non planifiées dans le sprint courant.  
Permet d’ajouter librement des idées **sans polluer** [SPRINTS.md](./SPRINTS.md).

## Comment ajouter une feature

1. Copier le modèle ci-dessous.
2. Remplir tous les champs.
3. Laisser le statut `Proposed`.
4. En planification de sprint : passer à `Ready`, puis créer des tâches dans [SPRINTS.md](./SPRINTS.md).

### Modèle (à copier)

```markdown
### BL-XXX — Titre court
- **Statut :** Proposed | Ready | Scheduled | Done | Rejected
- **Priorité :** P0 (bloquant) | P1 | P2 | P3
- **Effort :** S | M | L | XL
- **Version cible suggérée :** 0.x / 1.x / unscheduled
- **Problème :** …
- **Proposition :** …
- **Hors scope / risques :** …
- **Liens :** issues, docs…
```

---

## Items

### BL-001 — Historique des corrections
- **Statut :** Proposed
- **Priorité :** P2
- **Effort :** M
- **Version cible suggérée :** 1.x
- **Problème :** Impossible de revoir les textes déjà traités.
- **Proposition :** Store local chiffré optionnel (désactivé par défaut) des N derniers résultats.
- **Hors scope / risques :** PII — opt-in strict, purge.
- **Liens :** —

### BL-002 — Raccourci custom (non Ctrl+C)
- **Statut :** Done
- **Priorité :** P1
- **Effort :** M
- **Version cible suggérée :** 0.4
- **Problème :** Sur GNOME strict, Ctrl+C+C peut être fragile.
- **Proposition :** GlobalShortcuts configurable (ex. Ctrl+Shift+Space) en complément du trigger clipboard.
- **Hors scope / risques :** Ne jamais binder Ctrl+C seul (avalerait la copie).
- **Liens :** docs/OS_COMPATIBILITY.md, S3-4 (`linux_portal_shortcut`, défaut `CTRL+SHIFT+SPACE`)

### BL-003 — Plus de langues (IT, PT, JA…)
- **Statut :** Proposed
- **Priorité :** P3
- **Effort :** S
- **Version cible suggérée :** 0.5+
- **Problème :** Uniquement FR/EN/ES/DE.
- **Proposition :** Étendre `Language` + prompts ; détection `whatlang`.
- **Hors scope / risques :** Qualité modèle selon langue.
- **Liens :** crates/shared-types

### BL-004 — Mode « correction inline » sans popup
- **Statut :** Proposed
- **Priorité :** P3
- **Effort :** L
- **Version cible suggérée :** unscheduled
- **Problème :** La popup casse le flow pour les power users.
- **Proposition :** Option : corriger et coller directement après Ctrl+C+C.
- **Hors scope / risques :** UX confirmation / undo.
- **Liens :** —

### BL-005 — GPU selector (Vulkan / CUDA / Metal)
- **Statut :** Proposed
- **Priorité :** P2
- **Effort :** M
- **Version cible suggérée :** 0.5
- **Problème :** Auto GPU layers parfois sous-optimal.
- **Proposition :** Setting `n_gpu_layers` + backend exposé dans config TOML / UI.
- **Hors scope / risques :** Build matrices CI.
- **Liens :** docs/AI_ENGINE.md

### BL-006 — Snippets / tons personnalisés
- **Statut :** Proposed
- **Priorité :** P3
- **Effort :** M
- **Version cible suggérée :** 1.x
- **Problème :** Les presets natifs (Contexte Mail/Chat/Note + Ton Formel/Concis/Fluide, voir BL-014) ne couvrent pas tous les styles métier.
- **Proposition :** Tons / contextes *user-defined* (fragments de prompt) dans la config TOML, après les presets natifs.
- **Hors scope / risques :** Injection de prompt malveillant en local seulement.
- **Liens :** BL-014

### BL-007 — Tests d’intégration OS automatisés
- **Statut :** Proposed
- **Priorité :** P2
- **Effort :** L
- **Version cible suggérée :** 1.0
- **Problème :** Matrice OS manuelle seulement.
- **Proposition :** Harness CI par OS (smoke clipboard mock).
- **Hors scope / risques :** Runners macOS/Linux Wayland coûteux.
- **Liens :** docs/OS_COMPATIBILITY.md

---

## Rejetés (archive courte)

| ID | Titre | Raison |
| --- | --- | --- |
| — | Inférence cloud | Contredit l’objectif 100 % local |

---

### BL-008 — Fenêtre principale type Reverso (tray)
- **Statut :** Done (S2-6 / 0.3.0)
- **Priorité :** P0
- **Effort :** L
- **Version cible suggérée :** 0.3 (S2-6)
- **Problème :** Le tray ouvre la même popup frameless ; pas de surface pour écrire librement, voir le résultat et copier/coller comme Reverso.
- **Proposition :** Seconde fenêtre Tauri `main` (décorée, taskbar) : éditeur source + résultat, onglets Correction / Traduction / Reformulation, boutons Copier / Remplacer. Clic tray / « Afficher » → `main` ; `Ctrl+C+C` → `popup` uniquement.
- **Hors scope / risques :** Ne pas fusionner les deux surfaces ; garder la popup légère.
- **Liens :** docs/ARCHITECTURE.md §6, capture Reverso de référence

### BL-009 — Positionner la popup près de la sélection
- **Statut :** Done (S2-7 / 0.3.0 — GetGUIThreadInfo ; UI Automation avancée hors scope)
- **Priorité :** P1
- **Effort :** M
- **Version cible suggérée :** 0.3 (S2-7)
- **Problème :** La popup s’affiche centrée, pas au-dessus du texte sélectionné.
- **Proposition :** Après capture HWND, estimer caret / selection rect (UI Automation / GetGUIThreadInfo) et `set_position` de la popup.
- **Hors scope / risques :** Fallbacks multi-écrans ; apps sans caret accessible → centre écran.
- **Liens :** docs/OS_COMPATIBILITY.md

---

### BL-010 — Personnalisation avancée (couleur libre, polices, i18n)
- **Statut :** Proposed
- **Priorité :** P3
- **Effort :** L
- **Version cible suggérée :** 1.x
- **Problème :** Les presets mode / accent / densité (0.3.1) ne couvrent pas une teinte libre, des polices custom ni une UI multilingue.
- **Proposition :** Sélecteur accent hex (ou HSL), polices système / import local, i18n chaînes UI + messages `RunaError`.
- **Hors scope / risques :** Ne pas gonfler le CSS runtime ; rester 100 % local (pas de CDN de fonts).
- **Liens :** docs/ARCHITECTURE.md §6 Thème, S2.1

### BL-011 — Injection clavier Wayland (RemoteDesktop / libei)
- **Statut :** Proposed
- **Priorité :** P1
- **Effort :** L
- **Version cible suggérée :** 0.4+ / 1.0
- **Problème :** Sous Wayland, S3 laisse le texte dans le presse-papiers (`InjectionFallback`) sans Ctrl+V synthétique.
- **Proposition :** Portail XDG RemoteDesktop ou libei pour injecter Ctrl+V après restore focus ; garder le repli clipboard.
- **Hors scope / risques :** Consentement portail utilisateur ; SELinux ; pas tous les compositors.
- **Liens :** docs/OS_COMPATIBILITY.md §4, S3-5

### BL-012 — Caret / sélection AT-SPI (Linux)
- **Statut :** Proposed
- **Priorité :** P2
- **Effort :** M
- **Version cible suggérée :** 1.0
- **Problème :** `capture_caret` Linux renvoie `None` — popup centrée.
- **Proposition :** AT-SPI2 (ou équivalent) pour positionner la popup près de la sélection, comme Win `GetGUIThreadInfo`.
- **Hors scope / risques :** Apps non accessibles ; permissions.
- **Liens :** docs/OS_COMPATIBILITY.md, BL-009

---

### BL-014 — Axes reformulation Contexte + Ton
- **Statut :** Done
- **Priorité :** P1
- **Effort :** S
- **Version cible suggérée :** 0.5
- **Problème :** Les 3 tons (Formel / Concis / Fluide) étaient trop faibles côté prompt ; pas de distinction mail / chat / note.
- **Proposition :** Axe `WritingContext` (Mail / Chat / Note) + ton existant ; prompts empilés ; chips UI ; IPC `correct(text, tone, context)`.
- **User stories :**
  - **US-R1** — Mail + Formel → mail poli structuré, pas de jargon chat.
  - **US-R2** — Chat + Concis → message court Slack/Teams, pas de « Cordialement ».
  - **US-R3** — Note + Fluide → mémo / todo naturel, listes OK.
  - **US-R4** — Changer contexte ou ton puis Lancer → nouveau couple dans le prompt.
  - **US-R5** — `Ctrl+C+C` auto-correct utilise le dernier couple (défaut Note + Fluide).
- **Hors scope / risques :** Tons custom (→ BL-006) ; détection auto du canal.
- **Liens :** crates/ai-engine/src/prompts.rs, shared-types WritingContext

### BL-015 — Code-signing Authenticode / notarisation
- **Statut :** Proposed
- **Priorité :** P2
- **Effort :** M
- **Version cible suggérée :** 1.0+ / 1.1
- **Problème :** Sans certificat Authenticode (Win) / notarisation (macOS), SmartScreen / Gatekeeper affichent des avertissements même si l’updater Tauri est signé (minisign).
- **Proposition :** Intégrer signature OS des installeurs dans la CI release (secrets certificat) ; documenter le process de renouvellement.
- **Hors scope / risques :** Coût certificat ; hors bloquant S5 si absent (updater minisign reste requis).
- **Liens :** S5, docs/COMMANDS.md § Release, Prompt 13

---

## Prochaine ID libre

`BL-016`
