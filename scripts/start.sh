#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
if [[ -z "${FFMPEG_BIN:-}" && ! -x .tools/ffmpeg/bin/ffmpeg ]]; then
  bash scripts/install-ffmpeg.sh
fi
npm run build
cargo build --release
exec bash scripts/serve.sh
