import { execFile } from "node:child_process";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);

export type VerifiedVideo = {
  readonly durationMs: number;
  readonly codec: string;
  readonly pixelFormat: string;
  readonly audioCodec: string | null;
};

export async function verifyVideoOutput(
  path: string,
  options: { readonly expectAudio?: boolean } = {},
): Promise<VerifiedVideo> {
  const { stdout } = await execFileAsync("ffprobe", [
    "-v",
    "error",
    "-show_entries",
    "stream=codec_type,codec_name,width,height,r_frame_rate,pix_fmt,duration:format=duration",
    "-of",
    "json",
    path,
  ]);
  const parsed = JSON.parse(stdout) as {
    readonly streams?: readonly {
      readonly codec_name?: string;
      readonly codec_type?: string;
      readonly width?: number;
      readonly height?: number;
      readonly r_frame_rate?: string;
      readonly pix_fmt?: string;
      readonly duration?: string;
    }[];
    readonly format?: { readonly duration?: string };
  };
  const stream = parsed.streams?.find(
    (candidate) => candidate.codec_type === "video",
  );
  const audio = parsed.streams?.find(
    (candidate) => candidate.codec_type === "audio",
  );
  if (
    !stream ||
    stream.codec_name !== "h264" ||
    stream.width !== 1920 ||
    stream.height !== 1080 ||
    stream.r_frame_rate !== "30/1" ||
    stream.pix_fmt !== "yuv420p"
  ) {
    throw new Error("Rendered MP4 does not match the required video profile.");
  }
  if (options.expectAudio && audio?.codec_name !== "aac") {
    throw new Error("Rendered MP4 is missing the required AAC audio stream.");
  }
  const durationMs = Number(parsed.format?.duration ?? stream.duration) * 1000;
  if (!Number.isFinite(durationMs) || durationMs <= 0) {
    throw new Error("Rendered MP4 has an invalid duration.");
  }
  return {
    durationMs,
    codec: stream.codec_name,
    pixelFormat: stream.pix_fmt,
    audioCodec: audio?.codec_name ?? null,
  };
}
