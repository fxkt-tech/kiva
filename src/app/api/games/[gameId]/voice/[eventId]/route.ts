import { notFound } from "next/navigation";
import type { GameId } from "@/core/types";
import { createGameRepository } from "@/server/game-repository";
import { rangedFileResponse } from "@/server/http/ranged-file-response";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

type RouteProps = { readonly params: Promise<{ readonly gameId: string; readonly eventId: string }> };

export async function GET(request: Request, { params }: RouteProps) {
  const { gameId, eventId } = await params;
  const repository = createGameRepository(process.env.KIVA_DATA_DIR);
  const record = await repository.get(gameId as GameId);
  if (!record || !(eventId in record.voiceArtifactsByEventId)) notFound();
  const artifact = record.voiceArtifactsByEventId[eventId]!;
  if (artifact.audio.file !== `${eventId}.mp3`) notFound();
  try {
    return await rangedFileResponse({
      request,
      path: repository.voicePath(gameId as GameId, eventId),
      headers: { "content-type": "audio/mpeg", "cache-control": "private, max-age=31536000, immutable" },
    });
  } catch {
    notFound();
  }
}
