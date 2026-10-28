# Scene / split

Upload a video, split at detected scene changes, and download every clip in one ZIP. React and TypeScript frontend, Rust/Axum backend, FFmpeg processing.

## Run

Requires Node.js 22+, npm, and Rust 1.88+. On Linux x86_64, first startup downloads a matched codec-complete FFmpeg 8.1/ffprobe pair into `.tools/ffmpeg/`, verifies the release SHA-256 digest, and retains the build's license files. Setup needs `curl`, `tar`, `xz`, and `sha256sum`. System packages are not changed.

Rust prefers this project-local pair. To use another build or platform, set both `FFMPEG_BIN` and `FFPROBE_BIN` to matching executables. Startup checks full H.264/HEVC decoders, FFV1/PCM encoders, and required filters. Restricted distribution builds such as Fedora's `ffmpeg-free` may omit HEVC and cannot handle all uploaded footage.

```sh
npm ci
npm start
```

Open **http://localmachine:1900** from the same Tailscale network. `npm start` builds both apps and binds Rust to this machine's Tailscale IPv4 address. It does not expose private footage on every network interface. Stop with Ctrl+C. When the systemd service is running, use the service commands below instead of starting a duplicate process.

Without Tailscale, explicitly choose an approved private listening address:

```sh
APP_HOST=192.168.1.10 PORT=1900 npm start
```

The app has no login. Use a trusted private network. Do not expose it to the public internet without authentication and a reverse proxy.

For frontend development, run the Rust server first, then `npm run dev -- --port 5173`. The Vite proxy in `vite.config.ts` points at the current server's Tailscale address; update that address when moving machines.

## Reboot-persistent service

This machine runs `scene-split.service` through the user systemd manager. It is enabled for boot, and user lingering is already enabled so login is not required. The runtime launcher resolves the current Tailscale address; if that address is not ready during boot, systemd retries every five seconds. The service runs prebuilt artifacts without reinstalling packages or rebuilding at boot.

```sh
systemctl --user status scene-split.service
systemctl --user restart scene-split.service
journalctl --user -u scene-split.service -n 50
```

Source unit: `scripts/scene-split.service`. Installed unit: `~/.config/systemd/user/scene-split.service`.

To deploy later code changes, build before restarting:

```sh
npm run build
cargo build --release
systemctl --user restart scene-split.service
```

## Use

1. Upload or drop a video. Uploads stream to disk without an app-imposed size cap.
2. Adjust detection sensitivity. Higher values detect smaller visual differences.
3. Click **Split video**. Processing runs in the background and survives a browser refresh.
4. Download the ZIP, named after the source video: `holiday.mp4` becomes `holiday.zip`. It contains numbered `.mkv` clips and `manifest.json` with source metadata and scene ranges.
5. Use **Clear video** to return to the upload screen without deleting server files. Refresh also starts with an empty workspace; the previous video is not restored.
6. Use the separate trash control to delete the currently selected upload and generated files from this server.

Each cut starts a new clip: two cuts produce three clips. A video with no detected changes produces one clip. Detection uses FFmpeg's frame-difference score, not semantic recognition; fades, flashes, and motion may need different settings.

## Quality and storage

- FFV1 lossless video, source resolution and supported pixel precision. Cuts use source frame indices, including variable-frame-rate footage.
- All audio tracks become uncompressed 64-bit PCM. Audio packets can straddle scene boundaries.
- Primary video only. Subtitles, attachments, and dynamic HDR metadata are not preserved. This is not an archival copy of every source stream or metadata field.
- Unsupported lossless pixel formats fail rather than silently convert to a lower-quality format.
- MKV/FFV1 playback needs a compatible player or editor, such as VLC. A browser may not preview some sources even when server-side splitting works.
- Lossless exports can be much larger than the input. The source, clips, and ZIP coexist on disk. Available disk space, browser/network resources, and processing time still apply.
- Nothing overwrites the uploaded source. A JPEG poster is preview-only and never used for export.

Files live under `data/`, excluded from version control. Jobs persist in per-upload JSON files. Clearing the workspace or refreshing does not delete files or cancel server-side processing; save your download before leaving the completed workspace. One worker processes jobs at a time; other jobs queue. Completed jobs restore after a server restart. Interrupted jobs become retryable failures rather than remaining stuck in progress. There is no automatic expiry.

## Checks

```sh
npm run build
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

No automated tests were added, following local project policy. `cargo test` currently discovers zero tests. Real browser and decoded-media verification is recorded in `VALIDATION.md`.

Hallmark UI sweep, with Chromium installed:

```sh
CHROME_PATH=/usr/bin/chromium-browser node /home/utpal/.pi/agent/skills/hallmark/scripts/sloplint.mjs web/src --scope app --genre modern-minimal --render --url http://localmachine:1900
```

## Layout

- `web/src/`: upload workbench, preview, timeline, download, and job polling.
- `server/src/routes.rs`: streamed upload, job actions, range-enabled source and archive downloads.
- `server/src/processing.rs`: scene detection, frame-index segmentation, and ZIP64 packaging.
- `server/src/media.rs`: media probing and lossless pixel-format selection.
- `server/src/model.rs`: typed job states and persistence.
- `server/src/toolchain.rs`: shared executable resolution and media capability checks.
- `scripts/install-ffmpeg.sh`: installs the matched project-local media tools.
- `scripts/start.sh`: builds the app, then invokes the shared runtime launcher.
- `scripts/serve.sh`: starts prebuilt artifacts on the approved private interface.
- `scripts/scene-split.service`: reboot-persistent user service on port 1900.
- `DESIGN.md`: frontend design system derived from the supplied editorial reference.

Environment variables: `APP_HOST`, `PORT` (default `1900`), `DATA_DIR` (default `data`), `FFMPEG_BIN`, and `FFPROBE_BIN`.
