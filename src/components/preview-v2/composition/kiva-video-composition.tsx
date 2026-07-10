import { useEffect, useMemo, useState } from "react";
import {
  AbsoluteFill,
  continueRender,
  delayRender,
  Html5Audio,
  Sequence,
  staticFile,
  useCurrentFrame,
  useVideoConfig,
} from "remotion";
import { createHtmlFrameViewModel } from "./frame-view-model";
import { millisecondsToFrame } from "./timing";
import type { VideoCompositionInput } from "./types";
import { HtmlPlaybackStage } from "../stage/html-playback-stage";

export function KivaVideoComposition(input: VideoCompositionInput) {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const timeMs = (frame * 1000) / fps;
  const assets = resolveAssets(input.assets);
  const fontReady = useCompositionFont(assets.fontUrl);
  const viewModel = createHtmlFrameViewModel({
    gameTitle: input.gameTitle,
    items: input.items,
    timeMs,
    assets,
  });

  return (
    <AbsoluteFill style={{ backgroundColor: "#080a09" }}>
      {fontReady ? <HtmlPlaybackStage viewModel={viewModel} /> : null}
      {input.audioCues.map((cue) => {
        const from = millisecondsToFrame(cue.startsAtMs, fps);
        const durationInFrames =
          cue.durationMs === null
            ? undefined
            : Math.max(1, millisecondsToFrame(cue.durationMs, fps));

        return (
          <Sequence
            durationInFrames={durationInFrames}
            from={from}
            key={cue.id}
            name={cue.kind}
          >
            <Html5Audio
              src={resolveAssetUrl(cue.src)}
              trimBefore={millisecondsToFrame(cue.trimStartMs, fps)}
              volume={cue.volume}
            />
          </Sequence>
        );
      })}
    </AbsoluteFill>
  );
}

function resolveAssets(
  assets: VideoCompositionInput["assets"],
): VideoCompositionInput["assets"] {
  return {
    fontUrl: resolveAssetUrl(assets.fontUrl),
    dayBackgroundUrl: assets.dayBackgroundUrl
      ? resolveAssetUrl(assets.dayBackgroundUrl)
      : null,
    nightBackgroundUrl: assets.nightBackgroundUrl
      ? resolveAssetUrl(assets.nightBackgroundUrl)
      : null,
    avatarUrls: Object.fromEntries(
      Object.entries(assets.avatarUrls).map(([key, value]) => [
        key,
        resolveAssetUrl(value),
      ]),
    ),
  };
}

function resolveAssetUrl(source: string): string {
  const prefix = "remotion-static:";
  return source.startsWith(prefix)
    ? staticFile(source.slice(prefix.length))
    : source;
}

function useCompositionFont(fontUrl: string): boolean {
  const waitHandle = useMemo(
    () => delayRender("Loading Kiva preview font"),
    [],
  );
  const [ready, setReady] = useState(false);

  useEffect(() => {
    let active = true;
    const source = "url(" + JSON.stringify(fontUrl) + ")";
    const font = new FontFace("Kiva Noto Sans SC", source, {
      weight: "400 900",
    });

    font
      .load()
      .then((loaded) => {
        if (!active) {
          return;
        }
        document.fonts.add(loaded);
        setReady(true);
        continueRender(waitHandle);
      })
      .catch((error: unknown) => {
        continueRender(waitHandle);
        throw error;
      });

    return () => {
      active = false;
    };
  }, [fontUrl, waitHandle]);

  return ready;
}
