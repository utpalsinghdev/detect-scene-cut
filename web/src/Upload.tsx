import { useRef, useState } from 'react';
import { ArrowUpRight, Film, Upload } from 'lucide-react';

export function UploadZone({ choose, disabled }: { choose: (file: File) => void; disabled: boolean }) {
  const input = useRef<HTMLInputElement>(null);
  const [dragging, setDragging] = useState(false);
  return (
    <div className={`upload-zone ${dragging ? 'dragging' : ''}`}
      onDragOver={(event) => { event.preventDefault(); if (!disabled) setDragging(true); }}
      onDragLeave={(event) => { if (!event.currentTarget.contains(event.relatedTarget instanceof Node ? event.relatedTarget : null)) setDragging(false); }}
      onDrop={(event) => {
        event.preventDefault(); setDragging(false);
        const file = event.dataTransfer.files.item(0);
        if (file && !disabled) choose(file);
      }}>
      <div className="upload-corner"><Film size={18} /><span>Source footage</span></div>
      <div className="upload-content">
        <Upload className="upload-icon" size={36} strokeWidth={1.25} />
        <h2>Start with a video.</h2>
        <p>Drop your footage here, or choose a file.</p>
        <button className="button primary" onClick={() => input.current?.click()} disabled={disabled}>
          Upload video <ArrowUpRight size={18} />
        </button>
        <input ref={input} type="file" accept="video/*,.mkv,.mov,.avi,.mxf,.mts,.m2ts,.webm,.mp4" className="visually-hidden" aria-label="Choose video file" tabIndex={-1}
          disabled={disabled} onChange={(event) => {
            const file = event.target.files?.item(0);
            if (file) choose(file);
            event.target.value = '';
          }} />
      </div>
      <p className="upload-bottom">No upload size cap. Available server storage applies.</p>
    </div>
  );
}
