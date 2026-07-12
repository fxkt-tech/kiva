import { notFound } from "next/navigation";
import { rangedFileResponse } from "@/server/http/ranged-file-response";
import {
  loadPresenterVoiceManifest,
  presenterVoiceFilePath,
} from "@/server/presenter-voice-manifest";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

type RouteProps = {
  readonly params: Promise<{ readonly presenterId: string; readonly file: string }>;
};

export async function GET(request: Request, { params }: RouteProps) {
  const { presenterId, file } = await params;
  const dataDir = process.env.KIVA_DATA_DIR ?? "kivdb";
  try {
    const manifest = await loadPresenterVoiceManifest(dataDir, presenterId);
    if (!Object.values(manifest.clips).some((clip) => clip.file === file)) notFound();
    return await rangedFileResponse({
      request,
      path: presenterVoiceFilePath(dataDir, presenterId, file),
      headers: { "content-type": "audio/mpeg", "cache-control": "public, max-age=31536000, immutable" },
    });
  } catch {
    notFound();
  }
}
