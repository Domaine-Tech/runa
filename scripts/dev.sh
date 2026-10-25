#!/usr/bin/env bash
# Lance Runa en mode développement avec watch + relance automatique.
# Usage:
#   ./scripts/dev.sh              # tauri dev + feature llama (défaut)
#   ./scripts/dev.sh --no-llama   # sans feature llama (stub IA)
#   ./scripts/dev.sh --frontend   # Vite seul (HMR)
#   ./scripts/dev.sh --once       # un seul lancement, sans watch externe

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DESKTOP="$ROOT/apps/desktop"
MODE="watch"
DEBOUNCE_SEC=1
NO_LLAMA=0

log() { printf '\033[1;34m[runa-dev]\033[0m %s\n' "$*"; }
err() { printf '\033[1;31m[runa-dev]\033[0m %s\n' "$*" >&2; }

usage() {
  cat <<'EOF'
Usage: ./scripts/dev.sh [options]

  (défaut)     npm run tauri dev --features llama + watch des crates Rust
  --no-llama   Désactive la feature llama (stub IA)
  --frontend   Vite seul (HMR frontend)
  --once       Lance tauri/vite une fois, sans watch externe
  -h, --help   Aide
EOF
}

for arg in "$@"; do
  case "$arg" in
    --frontend) MODE="frontend" ;;
    --once) MODE="once" ;;
    --no-llama) NO_LLAMA=1 ;;
    -h|--help) usage; exit 0 ;;
    *) err "option inconnue: $arg"; usage; exit 1 ;;
  esac
done

ensure_deps() {
  if [[ ! -d "$DESKTOP/node_modules" ]]; then
    log "npm install (apps/desktop)…"
    (cd "$DESKTOP" && npm install)
  fi
  if ! command -v cargo >/dev/null 2>&1; then
    err "cargo introuvable — installez Rust (https://rustup.rs)"
    exit 1
  fi
}

run_tauri() {
  if [[ "$NO_LLAMA" -eq 1 ]]; then
    log "feature llama désactivée (stub IA)"
    (cd "$DESKTOP" && npm run tauri -- dev)
  else
    log "feature llama active (CMake / toolchain C++ requis)"
    (cd "$DESKTOP" && npm run tauri -- dev --features llama)
  fi
}

# Commande shell pour cargo-watch / watchexec (même logique que run_tauri).
tauri_watch_cmd() {
  if [[ "$NO_LLAMA" -eq 1 ]]; then
    printf '%s' "cd apps/desktop && npm run tauri -- dev"
  else
    printf '%s' "cd apps/desktop && npm run tauri -- dev --features llama"
  fi
}

run_frontend() {
  (cd "$DESKTOP" && npm run dev)
}

kill_child() {
  if [[ -n "${CHILD_PID:-}" ]] && kill -0 "$CHILD_PID" 2>/dev/null; then
    log "arrêt du processus $CHILD_PID…"
    kill "$CHILD_PID" 2>/dev/null || true
    wait "$CHILD_PID" 2>/dev/null || true
  fi
}

trap 'kill_child; exit 0' INT TERM

watch_with_cargo_watch() {
  if ! command -v cargo-watch >/dev/null 2>&1 && ! cargo watch -V >/dev/null 2>&1; then
    return 1
  fi
  log "watch via cargo-watch (crates + src-tauri)…"
  (
    cd "$ROOT"
    cargo watch \
      -w crates \
      -w apps/desktop/src-tauri \
      -i 'target/**' \
      -i '**/node_modules/**' \
      -i '**/*.gguf' \
      -s "$(tauri_watch_cmd)"
  )
}

watch_with_watchexec() {
  if ! command -v watchexec >/dev/null 2>&1; then
    return 1
  fi
  log "watch via watchexec…"
  (
    cd "$ROOT"
    watchexec \
      --restart \
      --stop-timeout 2 \
      --debounce "${DEBOUNCE_SEC}s" \
      --watch crates \
      --watch apps/desktop/src-tauri \
      --ignore 'target/**' \
      --ignore '**/node_modules/**' \
      --ignore '**/*.gguf' \
      --shell=bash \
      -- "$(tauri_watch_cmd)"
  )
}

watch_with_inotify() {
  if ! command -v inotifywait >/dev/null 2>&1; then
    return 1
  fi
  log "watch via inotifywait (relance tauri)…"
  while true; do
    run_tauri &
    CHILD_PID=$!
    # Attend un changement puis debounce
    inotifywait -r -e modify,create,delete,move \
      --exclude '(target|node_modules|\.git)' \
      "$ROOT/crates" "$ROOT/apps/desktop/src-tauri" >/dev/null 2>&1 || true
    sleep "$DEBOUNCE_SEC"
    kill_child
    log "changement détecté — relance…"
  done
}

watch_poll() {
  log "watch par polling (2s) — installez cargo-watch pour un meilleur confort"
  local stamp fail_streak=0
  stamp="$(find "$ROOT/crates" "$ROOT/apps/desktop/src-tauri" \
    -type f \( -name '*.rs' -o -name '*.toml' -o -name '*.json' \) \
    -printf '%T@\n' 2>/dev/null | sort -n | tail -1 || echo 0)"

  run_tauri &
  CHILD_PID=$!

  while true; do
    sleep 2
    local now
    now="$(find "$ROOT/crates" "$ROOT/apps/desktop/src-tauri" \
      -type f \( -name '*.rs' -o -name '*.toml' -o -name '*.json' \) \
      -printf '%T@\n' 2>/dev/null | sort -n | tail -1 || echo 0)"
    if [[ "$now" != "$stamp" ]]; then
      stamp="$now"
      fail_streak=0
      log "changement détecté — relance…"
      kill_child
      sleep "$DEBOUNCE_SEC"
      run_tauri &
      CHILD_PID=$!
      continue
    fi
    if ! kill -0 "$CHILD_PID" 2>/dev/null; then
      fail_streak=$((fail_streak + 1))
      if [[ "$fail_streak" -ge 3 ]]; then
        err "échec répété ($fail_streak) — pause jusqu'à un changement de source"
        while true; do
          sleep 2
          now="$(find "$ROOT/crates" "$ROOT/apps/desktop/src-tauri" \
            -type f \( -name '*.rs' -o -name '*.toml' -o -name '*.json' \) \
            -printf '%T@\n' 2>/dev/null | sort -n | tail -1 || echo 0)"
          if [[ "$now" != "$stamp" ]]; then
            stamp="$now"
            fail_streak=0
            log "changement détecté — relance…"
            break
          fi
        done
      else
        log "processus terminé — backoff ${fail_streak}s ($fail_streak/3)…"
        sleep "$fail_streak"
      fi
      run_tauri &
      CHILD_PID=$!
    fi
  done
}

ensure_deps

case "$MODE" in
  frontend)
    log "Vite (frontend)…"
    run_frontend
    ;;
  once)
    log "tauri dev (sans watch externe)…"
    run_tauri
    ;;
  watch)
    log "mode watch — Ctrl+C pour quitter"
    watch_with_cargo_watch || watch_with_watchexec || watch_with_inotify || watch_poll
    ;;
esac
