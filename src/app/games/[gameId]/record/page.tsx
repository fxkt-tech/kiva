import { notFound } from "next/navigation";
import { RecordingWorkspace } from "@/components/preview/recording-workspace";
import type { GameId } from "@/core/types";
import { createGameActions } from "@/server/game-actions";
import { createGameRepository } from "@/server/game-repository";

type RecordPageProps = {
  readonly params: Promise<{
    readonly gameId: string;
  }>;
};

export default async function RecordPage({ params }: RecordPageProps) {
  const { gameId } = await params;
  const dataDir = process.env.KIVA_DATA_DIR;
  const record = await createGameActions(createGameRepository(dataDir)).getGame(
    gameId as GameId,
  );

  if (!record) {
    notFound();
  }

  return (
    <RecordingWorkspace
      cleanPreviewHref={`/games/${record.game.id}/preview?controls=0`}
      previewHref={`/games/${record.game.id}/preview`}
      title={record.game.title}
    />
  );
}
