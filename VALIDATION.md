# Local validation

## Build and static checks

- `npm run build`: TypeScript and Vite production build passed.
- `cargo fmt --check`: passed after installing the missing rustfmt component.
- `cargo clippy -- -D warnings`: passed.
- `cargo test`: passed, zero tests discovered. New automated tests were not added under the local JStack configuration.

## Real media verification

Generated footage was uploaded through the running Rust API. Exported clips were decoded with FFmpeg and compared against decoded source frames. Frame order, counts, and pixel hashes matched in every case below on the final frame-index segmentation implementation.

- H.264/AAC MP4, 6 seconds, 144 frames: cuts at 2 and 4 seconds, three clips. Decoded video hashes and all decoded audio sample bytes matched.
- Silent constant scene, 12 frames: one clip, no detected cuts, identical frame hashes.
- 10-bit YUV, 24 frames: two clips, identical 10-bit frame hashes.
- RGB rawvideo source, 12 frames: one clip, identical RGB frame hashes after lossless packing conversion.
- Full-range MJPEG, 12 frames: one clip, identical full-range decoded frame hashes.
- H.264 MKV with two audio tracks at different sample rates, 144 frames: three clips. Video hashes and both audio tracks' decoded sample bytes matched.
- A 60-second source with timestamp gaps between repeated sequences: 29 cuts, 30 clips. Every clip contained exactly its expected 48 frames. Initial timestamp-based segmentation produced some 47/49-frame clips; source-frame-index segmentation fixed this. Encoder time base follows the demuxer to avoid average-frame-rate timestamp quantization.

Synthetic H.264 fixture generation initially tried unavailable `libx264`; regenerated successfully using installed `libopenh264`. This did not require an app dependency change.

## Browser and API

Checked the real app at `http://localmachine:3000` in Chromium:

- Uploaded MP4 through the file chooser, clicked Split, observed processing and completion, downloaded ZIP by clicking the browser link.
- Browser-downloaded ZIP passed `unzip -t`, containing all three clips and `manifest.json`.
- Scene timeline seek moved the source player to the selected scene start.
- Refresh restored the completed job. Restart restored completed jobs from disk.
- Delete confirmation supported Keep and Delete. Deletion returned the UI to the upload state and the removed API job returned 404.
- Invalid non-video upload returned 422 and removed its partial files.
- Out-of-range detection threshold returned 400.
- Duplicate split and delete-during-processing returned 409.
- Cross-origin mutation returned 403.
- Source downloads support HTTP byte ranges (206) and MP4 returns `video/mp4`.
- No browser runtime errors observed during the final flow.

Attempted an interruption-recovery check, but processing completed before termination. Completed-job restore passed; actual mid-job restart recovery remains unexercised, not claimed as a pass.

## UI review

Hallmark static and Chromium-rendered sweep covered 1280×800, 1440×900, 1920×1080, and a 375px-wide full page. Final result: zero FAIL, no open REFLEX, one waived gate (absolute black/white house palette, reported at 11 locations), one WARN. The 30ch mobile caption measure warning is intentionally retained for a short sentence, not paragraph text.

A first rendered sweep found an invisible file-input focus target. Removed it from the tab sequence; the visible Upload video button remains keyboard-accessible. Confirming render sweep passed focus checks.

375px completed state had no document-level horizontal overflow. The clips table scrolls inside its own wrapper.

Code review used sequential standards/spec passes, not independent subagents. No Git baseline existed in the initially empty workspace. Review found and fixed VFR cut drift, full-range pixel handling, source MIME handling, and queue persistence ordering so a failed disk write cannot leave an unstarted job marked queued.

Finish review was in-thread. App hierarchy, empty state, loaded state, responsive table, focus rings, and restrained controls accepted. No marketing material or fabricated metrics. No external review claimed.

## Limits

These checks establish the tested codecs and sources, not universal codec/HDR/editor compatibility. Files larger than 4 GB were not exercised, although uploads are streamed, body-size limiting is disabled, and ZIP entries use ZIP64. No automated suite exists yet. No other device was used to verify the Tailscale route.

## Independent final review

Sol 6.1 with medium reasoning reviewed the Rust backend, relevant frontend files, README, and this validation record. No blockers found. Reviewer independently checked the running health endpoint; existing browser/media evidence was inspected, not rerun. Limits above remain unverified. No implementation changes were needed after this review.

## Upload codec failure and archive naming fix

Reproduced the reported `Video pixel format is missing.` response exactly by uploading a generated HEVC Main10 MP4 through the original running API. System `ffprobe` returned `codec_name: hevc` without `pix_fmt`; a codec-complete matched probe returned `yuv420p10le`. Fedora's restricted media packages omitted HEVC and full native H.264 decoding.

Installed project-local BtbN FFmpeg/ffprobe 8.1 builds using the GitHub release's SHA-256 digest. Both runtime commands and startup checks now share one executable resolver. Startup rejects builds without required software decoders, lossless encoders, or filters. Probe stderr is retained in bounded server diagnostics; missing pixel precision produces a codec-specific error instead of a fabricated default. No system package or default pixel format was changed.

Verification after restart:

- Same HEVC Main10 fixture uploads and splits into two 12-frame FFV1 clips. Output remains `yuv420p10le`; all 24 decoded frame hashes match the source.
- H.264 High10 also uploads and splits successfully, preserving all decoded 10-bit frame hashes.
- Existing H.264/AAC fixture still produces three clips at 2s/4s; all video hashes and decoded audio samples match using the new runtime.
- Explicitly selecting restricted system binaries now fails startup with a capability error before opening a listener.
- ZIP download response derives its name from the source basename, with safe ASCII fallback and UTF-8 `filename*`. `holiday.mp4` downloaded through Chromium as `holiday.zip` without a forced destination filename. Spaces and multiple dots preserved in `My trip.v2.zip`; Unicode names safely encoded in the HTTP header.
- Browser-downloaded `holiday.zip` passed `unzip -t`.
- Existing completed user upload remained complete after restart and was not removed during fixture cleanup.
- TypeScript/Vite build, Rust format, Clippy, and `cargo test` passed. Test count remains zero under local policy. Both shell scripts passed `bash -n`.

The user's original rejected upload was not retained by the failed-upload cleanup path, so this verifies the exact reproduced error and affected codec families, not that specific source file. The diagnosis was independently reviewed by Sol 6.1 medium; implementation and media verification were performed in the parent session.

## Editorial redesign, persistent service, and workspace reset

Frontend now follows the supplied reference with white editorial space, self-hosted Space Grotesk/Inter, a stone upload region, deep-green controls, rounded media, and pill actions. Processing, probing, splitting, codecs, archives, and storage code are unchanged. The separately requested deployment change updates default/API proxy port to 1900.

- Systemd user service installed at `~/.config/systemd/user/scene-split.service`, enabled for `default.target`. Existing `Linger=yes` permits startup at boot without login. Runtime resolves the Tailscale address and retries on failure; artifacts are prebuilt, not rebuilt at boot.
- Service restart verified by PID change and journal stop/start records. Health GET succeeds through `http://localmachine:1900`. Port 3000 has no listener. Actual host reboot was not performed.
- Existing user upload remained complete across the move and restart. Only known synthetic fixture jobs are removed during cleanup.
- Real browser upload, split, and ZIP download passed on port 1900. ZIP integrity passed. Desktop and mobile empty/completed states captured under `.impeccable/review/`; self-hosted fonts loaded and no document-level horizontal overflow observed. The clips table scrolls within its wrapper.
- Rendered accessibility sweep found the blue focus ring insufficient against deep green. White focus rings inside controls fixed the issue; confirming render had zero Floor failures. Large functional green field retained under the user-pinned reference waiver. Two short mobile-caption measure warnings are documented, not hidden. Mechanical Impeccable scan reported no findings.
- Latest user request supersedes previous UI job restoration: a browser refresh starts empty and removes the legacy stored selection. A visible Clear video button also returns to upload without sending DELETE. Confirmed by querying the still-existing ready upload after Clear and the completed export after refresh. Trash remains separate. No backend deletion or cancellation occurs on workspace reset.
- Final TypeScript/Vite build passed. Rust build and format passed for the port-default change; no new tests were added. Active LSP probe reported no errors on changed React/hook paths, though one push-only server did not positively confirm clean; the TypeScript build provides the definitive compiler check.

Final independent frontend review by Sol 6.1 medium returned SHIP with no concrete blockers after inspecting all four final screenshots and current CSS/React/hook source. It did not independently repeat browser/runtime checks. Keyboard-only navigation, upload cancellation, network recovery, and touch-based table scrolling were not independently exercised by that reviewer. Initial review/documentation delegate failed at RPC startup; retry completed the review, while DESIGN.md and its schema-2 sidecar were documented in-thread from the implemented tokens.

Known synthetic upload jobs were removed after verification. Existing user upload remained complete. Service remains active/enabled on the Tailscale address at port 1900; port 3000 remains unused by this app.
