import { Download, Play } from 'lucide-react';
import { bytes, time } from './api';
import type { Job } from './api';

export function Scenes({ job, seek, currentTime }: { job: Job | null; seek: (time: number) => void; currentTime: number }) {
  const boundaries = job ? [0, ...job.cuts, job.video.duration] : [];
  const detected = job && ['exporting', 'packaging', 'complete'].includes(job.status.stage);
  const ranges = detected ? boundaries.slice(0, -1).map((start, index) => ({ start, end: boundaries[index + 1] ?? start })) : [];
  return (
    <section className="scenes" aria-labelledby="scenes-title">
      <div className="section-heading">
        <h2 id="scenes-title">Scene timeline</h2>
        <span>{detected ? `${ranges.length} ${ranges.length === 1 ? 'clip' : 'clips'} / ${job.cuts.length} cuts` : 'Waiting for scene detection'}</span>
      </div>
      {ranges.length > 0 && job ? <>
        <div className="timeline" aria-label="Detected scenes">
          {ranges.map((range, index) => <button key={range.start}
            style={{ flexGrow: Math.max(range.end - range.start, 0.01) }}
            className={currentTime >= range.start && currentTime < range.end ? 'timeline-scene selected' : 'timeline-scene'}
            aria-label={`Seek to scene ${index + 1}, ${time(range.start)}`}
            title={`Scene ${index + 1}: ${time(range.start)} to ${time(range.end)}`}
            onClick={() => seek(range.start)}><span>{String(index + 1).padStart(2, '0')}</span></button>)}
        </div>
        <div className="time-ruler"><span>{time(0)}</span><span>{time(job.video.duration)}</span></div>
      </> : <div className="timeline-empty"><span>{job ? 'Your scene boundaries will appear here after splitting.' : 'One video in. Every scene out.'}</span></div>}
      {job?.status.stage === 'complete' && <>
        <div className="results-heading"><h3>Exported clips</h3><span>{bytes(job.status.bytes)} total ZIP</span></div>
        <div className="clip-table-wrap">
          <table className="clip-table">
            <thead><tr><th scope="col">Clip</th><th scope="col">Source range</th><th scope="col">Duration</th><th scope="col">Size</th><th scope="col"><span className="visually-hidden">Preview</span></th></tr></thead>
            <tbody>{job.status.clips.map((clip, index) => <tr key={clip.name}>
              <td><span className="clip-index">{String(index + 1).padStart(2, '0')}</span>{clip.name}</td>
              <td>{time(clip.start)} <span className="range-separator">/</span> {time(clip.end)}</td>
              <td>{time(clip.end - clip.start)}</td><td>{bytes(clip.bytes)}</td>
              <td><button className="icon-button" aria-label={`Preview scene ${index + 1} in source video`} title="Preview in source" onClick={() => seek(clip.start)}><Play size={15} /></button></td>
            </tr>)}</tbody>
          </table>
        </div>
        <a className="download-inline" href={`/api/jobs/${job.id}/archive`} download><Download size={16} /> Download all clips</a>
      </>}
    </section>
  );
}
