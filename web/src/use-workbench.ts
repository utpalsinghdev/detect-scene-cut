import { useCallback, useEffect, useRef, useState } from 'react';
import { isProcessing, requestJob, uploadVideo } from './api';
import type { Job } from './api';

type Workbench =
  | { kind: 'empty' }
  | { kind: 'uploading'; name: string; progress: number }
  | { kind: 'loaded'; job: Job };

const message = (error: unknown) => error instanceof Error ? error.message : 'Something went wrong. Try again.';

export function useWorkbench() {
  const [state, setState] = useState<Workbench>({ kind: 'empty' });
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const upload = useRef<AbortController | null>(null);
  const requestLock = useRef(false);

  useEffect(() => {
    localStorage.removeItem('scene-split-job');
  }, []);

  const job = state.kind === 'loaded' ? state.job : null;
  const jobId = job?.id;
  const active = job ? isProcessing(job) : false;

  useEffect(() => {
    if (!jobId || !active) return;
    const controller = new AbortController();
    let timer: ReturnType<typeof setTimeout>;
    const poll = async () => {
      try {
        const next = await requestJob(`/api/jobs/${jobId}`, { signal: controller.signal });
        setState({ kind: 'loaded', job: next });
        setError(null);
      } catch (error) {
        if (controller.signal.aborted) return;
        setError(`${message(error)} Reconnecting automatically.`);
      }
      if (!controller.signal.aborted) timer = setTimeout(() => void poll(), 1000);
    };
    void poll();
    return () => { controller.abort(); clearTimeout(timer); };
  }, [jobId, active]);

  useEffect(() => () => upload.current?.abort(), []);

  const choose = useCallback(async (file: File) => {
    if (requestLock.current) return;
    requestLock.current = true;
    setError(null);
    const controller = new AbortController();
    upload.current = controller;
    setState({ kind: 'uploading', name: file.name, progress: 0 });
    try {
      const job = await uploadVideo(file, (progress) => setState({ kind: 'uploading', name: file.name, progress }), controller.signal);
      setState({ kind: 'loaded', job });
    } catch (error) {
      if (!controller.signal.aborted) setError(message(error));
      setState({ kind: 'empty' });
    } finally {
      requestLock.current = false;
      upload.current = null;
    }
  }, []);

  const split = async (sensitivity: number) => {
    if (!job || requestLock.current) return;
    requestLock.current = true;
    setPending(true);
    setError(null);
    try {
      const next = await requestJob(`/api/jobs/${job.id}/split`, {
        method: 'POST', headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ threshold: (100 - sensitivity) / 100 }),
      });
      setState({ kind: 'loaded', job: next });
    } catch (error) { setError(message(error)); }
    finally { setPending(false); requestLock.current = false; }
  };

  const remove = async () => {
    if (!job || requestLock.current) return;
    requestLock.current = true;
    setPending(true);
    setError(null);
    try {
      const response = await fetch(`/api/jobs/${job.id}`, { method: 'DELETE' });
      if (!response.ok) throw new Error('Could not delete this upload. Try again.');
      setState({ kind: 'empty' });
    } catch (error) { setError(message(error)); }
    finally { setPending(false); requestLock.current = false; }
  };

  const clear = () => {
    if (active || requestLock.current) return;
    setState({ kind: 'empty' });
    setError(null);
  };

  return { state, job, active, error, pending, choose, split, remove, clear, cancel: () => upload.current?.abort() };
}
