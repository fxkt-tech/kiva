"use client";

import { Download, ExternalLink, Radio, Square } from "lucide-react";
import { useEffect, useRef, useState } from "react";

type RecordingControlsProps = {
  readonly cleanPreviewHref: string;
};

type RecordingStatus = "idle" | "recording" | "ready" | "failed";

export function RecordingControls({
  cleanPreviewHref,
}: RecordingControlsProps) {
  const recorderRef = useRef<MediaRecorder | null>(null);
  const streamRef = useRef<MediaStream | null>(null);
  const chunksRef = useRef<Blob[]>([]);
  const [status, setStatus] = useState<RecordingStatus>("idle");
  const [downloadUrl, setDownloadUrl] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    return () => {
      if (downloadUrl) {
        URL.revokeObjectURL(downloadUrl);
      }
      stopStream(streamRef.current);
    };
  }, [downloadUrl]);

  async function startRecording() {
    setError(null);

    if (!navigator.mediaDevices?.getDisplayMedia || !window.MediaRecorder) {
      setStatus("failed");
      setError("Browser recording is not supported.");
      return;
    }

    if (downloadUrl) {
      URL.revokeObjectURL(downloadUrl);
      setDownloadUrl(null);
    }

    try {
      const stream = await navigator.mediaDevices.getDisplayMedia({
        audio: false,
        video: {
          frameRate: 30,
        },
      });
      const mimeType = preferredMimeType();
      const recorder = new MediaRecorder(
        stream,
        mimeType ? { mimeType } : undefined,
      );

      chunksRef.current = [];
      streamRef.current = stream;
      recorderRef.current = recorder;

      recorder.ondataavailable = (event) => {
        if (event.data.size > 0) {
          chunksRef.current.push(event.data);
        }
      };
      recorder.onstop = () => {
        const blob = new Blob(chunksRef.current, {
          type: recorder.mimeType || "video/webm",
        });
        stopStream(stream);
        streamRef.current = null;
        recorderRef.current = null;
        setDownloadUrl(URL.createObjectURL(blob));
        setStatus("ready");
      };

      recorder.start();
      setStatus("recording");
    } catch (caught) {
      stopStream(streamRef.current);
      streamRef.current = null;
      recorderRef.current = null;
      setStatus("failed");
      setError(caught instanceof Error ? caught.message : String(caught));
    }
  }

  function stopRecording() {
    const recorder = recorderRef.current;
    if (recorder && recorder.state !== "inactive") {
      recorder.stop();
    }
  }

  const recording = status === "recording";

  return (
    <div className="flex flex-wrap items-center gap-2">
      <a
        className="inline-flex items-center gap-1.5 border border-zinc-700 px-3 py-2 text-xs text-zinc-200 transition hover:border-zinc-500 hover:bg-zinc-900"
        href={cleanPreviewHref}
        rel="noreferrer"
        target="_blank"
      >
        <ExternalLink aria-hidden="true" size={14} />
        Open clean preview
      </a>
      <button
        className="inline-flex items-center gap-1.5 border border-red-900/70 px-3 py-2 text-xs text-red-200 transition hover:border-red-500 hover:bg-red-950/40 disabled:cursor-not-allowed disabled:border-zinc-900 disabled:text-zinc-700"
        disabled={recording}
        onClick={startRecording}
        type="button"
      >
        <Radio aria-hidden="true" size={14} />
        Start recording
      </button>
      <button
        className="inline-flex items-center gap-1.5 border border-zinc-700 px-3 py-2 text-xs text-zinc-200 transition hover:border-zinc-500 hover:bg-zinc-900 disabled:cursor-not-allowed disabled:border-zinc-900 disabled:text-zinc-700"
        disabled={!recording}
        onClick={stopRecording}
        type="button"
      >
        <Square aria-hidden="true" size={14} />
        Stop
      </button>
      {downloadUrl ? (
        <a
          className="inline-flex items-center gap-1.5 border border-emerald-800 px-3 py-2 text-xs text-emerald-200 transition hover:border-emerald-500 hover:bg-emerald-950/40"
          download="kiva-playback.webm"
          href={downloadUrl}
        >
          <Download aria-hidden="true" size={14} />
          Download WebM
        </a>
      ) : null}
      {recording ? (
        <span className="text-xs text-red-300">Recording</span>
      ) : null}
      {status === "ready" ? (
        <span className="text-xs text-emerald-300">Ready</span>
      ) : null}
      {error ? <span className="text-xs text-red-300">{error}</span> : null}
    </div>
  );
}

function preferredMimeType(): string | undefined {
  for (const mimeType of [
    "video/webm;codecs=vp9",
    "video/webm;codecs=vp8",
    "video/webm",
  ]) {
    if (MediaRecorder.isTypeSupported(mimeType)) {
      return mimeType;
    }
  }

  return undefined;
}

function stopStream(stream: MediaStream | null): void {
  stream?.getTracks().forEach((track) => {
    track.stop();
  });
}
