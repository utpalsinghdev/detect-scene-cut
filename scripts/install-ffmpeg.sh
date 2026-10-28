#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
if [[ "$(uname -s)" != Linux || "$(uname -m)" != x86_64 ]]; then
  echo 'Automatic media setup supports Linux x86_64. Set FFMPEG_BIN and FFPROBE_BIN to a matching full-codec build on other platforms.' >&2
  exit 1
fi
mkdir -p .tools
work="$(mktemp -d .tools/media-install.XXXXXX)"
trap 'rm -rf "$work"' EXIT
curl --fail --silent --show-error --location --retry 2 \
  https://api.github.com/repos/BtbN/FFmpeg-Builds/releases/tags/latest > "$work/release.json"
node --input-type=module - "$work/release.json" > "$work/asset" <<'JS'
import { readFileSync } from 'node:fs';
const release = JSON.parse(readFileSync(process.argv[2], 'utf8'));
const asset = release.assets?.find((asset) => asset.name === 'ffmpeg-n8.1-latest-linux64-gpl-8.1.tar.xz');
if (!asset || !/^sha256:[a-f0-9]{64}$/.test(asset.digest ?? '') || !asset.browser_download_url.startsWith('https://github.com/BtbN/FFmpeg-Builds/releases/download/')) {
  throw new Error('Expected FFmpeg 8.1 release and SHA-256 digest were not found. No binary installed.');
}
console.log(asset.browser_download_url);
console.log(asset.digest.slice(7));
JS
mapfile -t asset < "$work/asset"
echo 'Downloading codec-complete FFmpeg and ffprobe for this project...'
curl --fail --silent --show-error --location --retry 2 "${asset[0]}" -o "$work/media.tar.xz"
printf '%s  %s\n' "${asset[1]}" "$work/media.tar.xz" | sha256sum --check --status
tar -xJf "$work/media.tar.xz" -C "$work" --no-same-owner
extracted="$work/ffmpeg-n8.1-latest-linux64-gpl-8.1"
"$extracted/bin/ffmpeg" -version | head -1
"$extracted/bin/ffprobe" -version | head -1
if [[ -e .tools/ffmpeg ]]; then
  echo '.tools/ffmpeg already exists; refusing to overwrite an installed build.' >&2
  exit 1
fi
mv "$extracted" .tools/ffmpeg
printf '%s\n' "${asset[0]}" "sha256:${asset[1]}" > .tools/ffmpeg/SOURCE.txt
echo 'Installed matched FFmpeg and ffprobe in .tools/ffmpeg/bin.'
