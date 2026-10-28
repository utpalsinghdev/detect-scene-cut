import { useRef, useState } from 'react';
import { ArrowDownToLine, ArrowUpRight, Check, ChevronRight, Clapperboard, Scissors, ShieldCheck, Trash2, X } from 'lucide-react';
import { bytes, time } from './api';
import type { Job } from './api';
import { useWorkbench } from './use-workbench';
import { UploadZone } from './Upload';
import { Scenes } from './Scenes';

function stageLabel(job: Job): string {
  switch (job.status.stage) {
    case 'ready': return 'Ready to split';
    case 'queued': return 'Queued for processing';
    case 'detecting': return 'Finding scene changes';
    case 'exporting': return 'Writing lossless clips';
    case 'packaging': return 'Packing your ZIP';
    case 'complete': return 'Your clips are ready';
    case 'failed': return 'Processing stopped';
  }
}

export default function App() {
  const { state, job, active, error, pending, choose, split, remove, clear, cancel } = useWorkbench();
  const [sensitivity, setSensitivity] = useState(65);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [previewError, setPreviewError] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const player = useRef<HTMLVideoElement>(null);
  const busy = active || pending || state.kind === 'uploading';
  const complete = job?.status.stage === 'complete';
  const progress = job && (job.status.stage === 'detecting' || job.status.stage === 'exporting') ? job.status.progress : null;

  function seek(time: number) {
    if (player.current) { player.current.currentTime = time; player.current.focus(); }
    setCurrentTime(time);
  }

  return (
    <div className="app-shell">
      <a className="skip-link" href="#main">Skip to workbench</a>
      <header className="topbar">
        <a className="brand" href="/" aria-label="Scene split home"><Clapperboard size={23} strokeWidth={1.6} /><span>scene<span className="brand-divider">/</span>split</span></a>
        <span className="topbar-note"><ShieldCheck size={15} /> On your server. Not in a cloud.</span>
      </header>
      <main id="main">
        <div className="page-heading">
          <div><h1>Every scene. Its own clip.</h1><p>Find the cuts. Keep the quality. Take the whole ZIP.</p></div>
          <span className="format-note">FFV1 lossless export</span>
        </div>
        <div className="workbench">
          <section className="source-panel" aria-label="Video source">
            {state.kind === 'empty' && <UploadZone disabled={busy} choose={(file) => { setPreviewError(false); setCurrentTime(0); void choose(file); }} />}
            {state.kind === 'uploading' && <div className="upload-zone loading-panel" aria-live="polite">
              <ArrowUpRight size={32} strokeWidth={1.3} /><h2>{state.progress >= 1 ? 'Reading your video…' : 'Uploading footage…'}</h2>
              <p className="upload-filename">{state.name}</p>
              <progress max={1} value={state.progress} aria-label="Upload progress" />
              <span className="muted">{Math.round(state.progress * 100)}% transferred</span>
              <button className="button quiet" onClick={cancel}><X size={16} /> Cancel upload</button>
            </div>}
            {job && <>
              <div className="source-heading"><span>Source footage</span><span>{time(currentTime)} <span className="range-separator">/</span> {time(job.video.duration)}</span></div>
              <div className="video-stage">
                <video key={job.id} ref={player} controls preload="metadata" playsInline
                  src={`/api/jobs/${job.id}/source`} poster={`/api/jobs/${job.id}/poster`}
                  onTimeUpdate={(event) => setCurrentTime(event.currentTarget.currentTime)}
                  onError={() => setPreviewError(true)} aria-label={`Source video: ${job.name}`} />
              </div>
              {previewError && <p className="preview-notice">Browser cannot preview this codec. Scene detection and lossless export still work.</p>}
              <div className="source-info"><div><strong title={job.name}>{job.name}</strong><span>{job.video.width} × {job.video.height}<span className="info-space" />{job.video.fps.toFixed(2)} fps<span className="info-space" />{bytes(job.video.bytes)}</span></div>
                <div className="source-actions">
                  <button className="button clear-button" disabled={busy} title="Clear this workspace without deleting server files" onClick={() => { clear(); setConfirmDelete(false); setPreviewError(false); setCurrentTime(0); }}><X size={15} />Clear video</button>
                  <button className="icon-button" disabled={busy} aria-label="Delete video and exported clips" onClick={() => setConfirmDelete(true)}><Trash2 size={17} /></button>
                </div>
              </div>
              {confirmDelete && <div className="delete-confirm"><p>Delete this upload and its exports from the server?</p><button className="button quiet" onClick={() => setConfirmDelete(false)}>Keep</button><button className="button danger" disabled={pending} onClick={() => { void remove(); setConfirmDelete(false); setPreviewError(false); }}>Delete files</button></div>}
            </>}
          </section>
          <aside className="controls" aria-labelledby="split-title">
            <div className="controls-heading"><Scissors size={19} /><h2 id="split-title">Split by scene</h2></div>
            <div className="setting">
              <div className="setting-label"><label htmlFor="sensitivity">Detection sensitivity</label><output htmlFor="sensitivity">{sensitivity}%</output></div>
              <input id="sensitivity" type="range" min="1" max="99" value={sensitivity} disabled={busy} onChange={(event) => setSensitivity(Number(event.target.value))} aria-describedby="sensitivity-help" />
              <div className="range-labels"><span>Fewer cuts</span><span>More cuts</span></div>
              <p id="sensitivity-help">Higher values catch subtler visual changes.</p>
            </div>
            <div className="output-spec"><div><span>Video</span><strong>FFV1 / original resolution</strong></div><div><span>Audio</span><strong>Uncompressed / all tracks</strong></div><div><span>Download</span><strong>.mkv clips in one .zip</strong></div></div>
            <p className="quality-note"><ShieldCheck size={16} /><span>No added video quality loss. Lossless files can be much larger than the original.</span></p>
            <div className="action-area">
              {complete && job ? <a className="button primary full" href={`/api/jobs/${job.id}/archive`} download><ArrowDownToLine size={18} />Download ZIP</a> :
                <button className="button primary full" disabled={!job || busy} aria-busy={active || pending} onClick={() => void split(sensitivity)}><Scissors size={18} />{active ? 'Processing video…' : pending ? 'Starting…' : job?.status.stage === 'failed' ? 'Retry split' : 'Split video'}{!active && <ChevronRight size={17} className="button-end" />}</button>}
              {complete && <button className="button text-button" disabled={busy} onClick={() => void split(sensitivity)}>Split again with these settings</button>}
              {!job && <span className="action-hint">Upload a video to begin</span>}
              {job && <div className={`job-status ${complete ? 'success' : ''}`} role="status">{complete && <Check size={16} />}<span>{stageLabel(job)}</span>{progress !== null && <span>{Math.round(progress * 100)}%</span>}</div>}
              {progress !== null && <progress value={progress} max={1} aria-label={job ? stageLabel(job) : 'Progress'} />}
            </div>
          </aside>
        </div>
        {(error || job?.status.stage === 'failed') && <div role="alert" className="error-message"><strong>{job?.status.stage === 'failed' ? job.status.message : error}</strong><span>{job?.status.stage === 'failed' ? 'Adjust sensitivity or try another source, then retry.' : 'Your original file is unchanged.'}</span></div>}
        <Scenes job={job} currentTime={currentTime} seek={seek} />
        <details className="technical-notes"><summary>About cuts and quality</summary><div><p>Each detected change starts a new clip, so 3 cuts produce 4 clips. Sensitivity controls a visual difference threshold, not semantic understanding. Fades and camera motion may need different settings.</p><p>Video uses lossless FFV1 at the source resolution and supported pixel precision. Audio uses uncompressed 64-bit PCM. Audio packets can straddle cut boundaries. Only the primary video and all audio tracks are exported; subtitles, attachments, and dynamic HDR metadata are not preserved. Unsupported lossless pixel formats fail rather than silently reduce quality.</p><p>Files stay on this server until you delete them. Upload size has no app-imposed cap; disk space and processing time still apply. Downloaded MKV clips open in VLC and FFV1-compatible editors.</p></div></details>
      </main>
      <footer><span>scene / split</span><span>Original footage stays untouched.</span></footer>
    </div>
  );
}
