import { Composition } from "remotion";
import type { PlaybackItem } from "@/core/playback";
import { KivaVideoComposition } from "@/components/preview-v2/composition/kiva-video-composition";
import { compositionDurationInFrames } from "@/components/preview-v2/composition/timing";
import {
  VIDEO_COMPOSITION_SCHEMA_VERSION,
  type VideoCompositionInput,
} from "@/components/preview-v2/composition/types";
import { VIDEO_SPEC } from "@/components/preview-v2/composition/video-spec";

const defaultItem: PlaybackItem = {
  index: 1,
  phase: "night",
  kind: "announcement",
  title: "第 1 夜开始",
  text: "夜幕已经落下，所有玩家请确认自己的身份。",
  details: ["案卷已开启", "系统等待下一项行动"],
  durationMs: 1200,
  startsAtMs: 0,
  players: [],
  presenterName: "守夜人",
  presenterAvatar: null,
  transcriptSpeaker: "presenter",
  presenterCue: {
    copyKey: "phase.night",
    text: "夜幕已经落下，所有玩家请确认自己的身份。",
  },
};

export const defaultCompositionInput: VideoCompositionInput = {
  schemaVersion: VIDEO_COMPOSITION_SCHEMA_VERSION,
  gameId: "fixture",
  gameTitle: "Kiva Render Fixture",
  items: [defaultItem],
  assets: {
    fontUrl: "remotion-static:preview/noto-sans-sc-900.ttf",
    dayBackgroundUrl: "remotion-static:preview/day-background.png",
    nightBackgroundUrl: "remotion-static:preview/night-background.png",
    avatarUrls: {},
  },
  audioCues: [],
};

export function RemotionRoot() {
  return (
    <Composition
      calculateMetadata={({ props }) => ({
        durationInFrames: compositionDurationInFrames(props.items, VIDEO_SPEC.fps),
      })}
      component={KivaVideoComposition}
      defaultProps={defaultCompositionInput}
      durationInFrames={compositionDurationInFrames(
        defaultCompositionInput.items,
        VIDEO_SPEC.fps,
      )}
      fps={VIDEO_SPEC.fps}
      height={VIDEO_SPEC.height}
      id={VIDEO_SPEC.compositionId}
      width={VIDEO_SPEC.width}
    />
  );
}
