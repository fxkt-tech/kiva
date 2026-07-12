import { compilePublicPlayback } from "@/core/playback";
import { projectDirectorDuration } from "@/core/duration-projection";
import { getActiveEvents } from "@/core/event-log";
import type { GameId } from "@/core/types";
import { createGameRepository } from "@/server/game-repository";
import { loadPresenterVoiceManifest } from "@/server/presenter-voice-manifest";

void main();

async function main(): Promise<void> {
  const gameId = (process.argv[2] ??
    "ae9b3a8a-fb8b-43cf-901b-252ad49b0341") as GameId;
  const dataDir = process.env.KIVA_DATA_DIR ?? "kivdb";
  const repository = createGameRepository(dataDir);
  const record = await repository.get(gameId);
  if (!record) throw new Error(`Game not found: ${gameId}`);

  const presenterVoiceManifest = await loadPresenterVoiceManifest(
    dataDir,
    record.game.presenter.presenterSourceId,
  ).catch(() => undefined);
  const events = getActiveEvents(record.events);
  const playback = compilePublicPlayback(events, record.game.players, {
    presenter: record.game.presenter,
    audience: "director",
    voiceArtifactsByEventId: record.voiceArtifactsByEventId,
    presenterVoiceManifest,
  });
  const projection = projectDirectorDuration({ events, playback });

  console.log(
    JSON.stringify(
      {
        gameId,
        directorScenes: playback.length,
        actualDurationMinutes: minutes(projection.committedDurationMs),
        speechCount: projection.speechCount,
        speechCharacters: projection.speechCharacters,
        actualSpeechMinutes: minutes(projection.actualSpeechDurationMs),
        projectedDirectorMinutes: [
          minutes(projection.projectedMinDurationMs),
          minutes(projection.projectedMaxDurationMs),
        ],
        missingSpeechVoiceCount: projection.missingSpeechVoiceCount,
      },
      null,
      2,
    ),
  );
}

function minutes(durationMs: number): number {
  return Math.round((durationMs / 60_000) * 100) / 100;
}
