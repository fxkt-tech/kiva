import { bundle } from "@remotion/bundler";
import { enableTailwind } from "@remotion/tailwind-v4";
import {
  makeCancelSignal,
  renderMedia,
  selectComposition,
} from "@remotion/renderer";
import { resolve } from "node:path";
import type { VideoCompositionInput } from "@/components/preview-v2/composition/types";
import { VIDEO_SPEC } from "@/components/preview-v2/composition/video-spec";

export type VideoRenderProgress = {
  readonly progress: number;
  readonly stage: "rendering" | "encoding" | "muxing";
};

export type VideoRenderHandle = {
  readonly promise: Promise<void>;
  cancel(): void;
};

let bundlePromise: Promise<string> | null = null;

export function renderVideo(input: {
  readonly composition: VideoCompositionInput;
  readonly outputPath: string;
  readonly publicDir?: string;
  readonly onProgress?: (progress: VideoRenderProgress) => void;
}): VideoRenderHandle {
  const { cancel, cancelSignal } = makeCancelSignal();

  return {
    cancel,
    promise: render(input, cancelSignal),
  };
}

async function render(
  input: {
    readonly composition: VideoCompositionInput;
    readonly outputPath: string;
    readonly publicDir?: string;
    readonly onProgress?: (progress: VideoRenderProgress) => void;
  },
  cancelSignal: ReturnType<typeof makeCancelSignal>["cancelSignal"],
): Promise<void> {
  const serveUrl = await getBundle(input.publicDir);
  const composition = await selectComposition({
    serveUrl,
    id: VIDEO_SPEC.compositionId,
    inputProps: input.composition,
  });

  await renderMedia({
    serveUrl,
    composition,
    inputProps: input.composition,
    codec: VIDEO_SPEC.codec,
    audioCodec: VIDEO_SPEC.audioCodec,
    pixelFormat: VIDEO_SPEC.pixelFormat,
    outputLocation: input.outputPath,
    overwrite: false,
    cancelSignal,
    concurrency: 1,
    imageFormat: "png",
    x264Preset: "veryfast",
    onProgress: ({ progress, stitchStage }) => {
      input.onProgress?.({
        progress,
        stage:
          stitchStage === "muxing"
            ? "muxing"
            : progress >= 0.9
              ? "encoding"
              : "rendering",
      });
    },
  });
}

function getBundle(publicDir?: string): Promise<string> {
  if (!bundlePromise) {
    bundlePromise = bundle({
      entryPoint: resolve(process.cwd(), "src/video/index.ts"),
      publicDir: publicDir ?? null,
      webpackOverride: (configuration) => {
        const withTailwind = enableTailwind(configuration);
        return {
          ...withTailwind,
          cache: false,
          resolve: {
            ...withTailwind.resolve,
            alias: {
              ...withTailwind.resolve?.alias,
              "@": resolve(process.cwd(), "src"),
            },
          },
        };
      },
    }).catch((error: unknown) => {
      bundlePromise = null;
      throw error;
    });
  }

  return bundlePromise;
}
