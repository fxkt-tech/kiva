import { notFound } from "next/navigation";
import { PlaybackStage } from "@/components/preview/playback-stage";
import { compilePublicPlayback } from "@/core/playback";
import type { GameId } from "@/core/types";
import { createGameActions } from "@/server/game-actions";
import { createGameRepository } from "@/server/game-repository";

type PreviewPageProps = {
  readonly params: Promise<{
    readonly gameId: string;
  }>;
  readonly searchParams?: Promise<{
    readonly controls?: string;
  }>;
};

export default async function PreviewPage({
  params,
  searchParams,
}: PreviewPageProps) {
  const { gameId } = await params;
  const controlsParam = (await searchParams)?.controls;
  const dataDir = process.env.KIVA_DATA_DIR;
  const record = await createGameActions(createGameRepository(dataDir)).getGame(
    gameId as GameId,
  );

  if (!record) {
    notFound();
  }

  return (
    <PlaybackStage
      controls={
        controlsParam === "0" || controlsParam === "hidden"
          ? "hidden"
          : "visible"
      }
      items={compilePublicPlayback(record.events, record.game.players)}
    />
  );
}
