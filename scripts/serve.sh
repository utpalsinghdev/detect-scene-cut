#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
if [[ -z "${APP_HOST:-}" ]]; then
  APP_HOST="$(/usr/bin/tailscale ip -4 | head -1)"
  if [[ -z "$APP_HOST" ]]; then
    echo 'Tailscale address is not ready. Set APP_HOST to an approved private listening address.' >&2
    exit 1
  fi
fi
export APP_HOST
export PORT="${PORT:-1900}"
exec ./target/release/scene-split
