import { z } from 'zod';

const clipSchema = z.object({
  name: z.string(), start: z.number(), end: z.number(), bytes: z.number(),
});

export const jobSchema = z.object({
  id: z.string().uuid(),
  name: z.string(),
  video: z.object({
    duration: z.number(), width: z.number(), height: z.number(), fps: z.number(),
    codec: z.string(), pixelFormat: z.string(), audioTracks: z.number(), bytes: z.number(),
  }),
  cuts: z.array(z.number()),
  status: z.discriminatedUnion('stage', [
    z.object({ stage: z.literal('ready') }),
    z.object({ stage: z.literal('queued') }),
    z.object({ stage: z.literal('detecting'), progress: z.number() }),
    z.object({ stage: z.literal('exporting'), progress: z.number() }),
    z.object({ stage: z.literal('packaging') }),
    z.object({ stage: z.literal('complete'), clips: z.array(clipSchema), bytes: z.number() }),
    z.object({ stage: z.literal('failed'), message: z.string() }),
  ]),
});

export type Job = z.infer<typeof jobSchema>;
export type Clip = z.infer<typeof clipSchema>;
const errorSchema = z.object({ error: z.string() });

export function isProcessing(job: Job): boolean {
  return ['queued', 'detecting', 'exporting', 'packaging'].includes(job.status.stage);
}

export async function requestJob(path: string, options?: RequestInit): Promise<Job> {
  const response = await fetch(path, options);
  const body: unknown = await response.json();
  if (!response.ok) {
    const parsed = errorSchema.safeParse(body);
    throw new Error(parsed.success ? parsed.data.error : `Request failed (${response.status}).`);
  }
  return jobSchema.parse(body);
}

export function uploadVideo(file: File, onProgress: (progress: number) => void, signal: AbortSignal): Promise<Job> {
  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest();
    const abort = () => xhr.abort();
    signal.addEventListener('abort', abort, { once: true });
    xhr.open('POST', '/api/uploads');
    xhr.responseType = 'json';
    xhr.upload.onprogress = (event) => {
      if (event.lengthComputable) onProgress(event.loaded / event.total);
    };
    xhr.onloadend = () => signal.removeEventListener('abort', abort);
    xhr.onerror = () => reject(new Error('Upload connection lost. Check your connection and try again.'));
    xhr.onabort = () => reject(new DOMException('Upload cancelled.', 'AbortError'));
    xhr.onload = () => {
      const body: unknown = xhr.response;
      if (xhr.status < 200 || xhr.status >= 300) {
        const parsed = errorSchema.safeParse(body);
        reject(new Error(parsed.success ? parsed.data.error : `Upload failed (${xhr.status}).`));
        return;
      }
      const parsed = jobSchema.safeParse(body);
      if (parsed.success) resolve(parsed.data);
      else reject(new Error('Server returned an unreadable video response. Reload and try again.'));
    };
    const data = new FormData();
    data.append('video', file);
    xhr.send(data);
    if (signal.aborted) abort();
  });
}

export function bytes(value: number): string {
  if (value < 1024) return `${value} B`;
  const unit = Math.min(Math.floor(Math.log(value) / Math.log(1024)), 4);
  return `${(value / 1024 ** unit).toFixed(1)} ${['B', 'KB', 'MB', 'GB', 'TB'][unit]}`;
}

export function time(value: number): string {
  const minutes = Math.floor(value / 60);
  const seconds = (value % 60).toFixed(2).padStart(5, '0');
  return `${minutes.toString().padStart(2, '0')}:${seconds}`;
}
