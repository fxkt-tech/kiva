import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { notFound } from "next/navigation";
import { declaredPresenterVoiceFiles } from "@/core/presenter";
import { createLibraryRepository } from "@/server/library-repository";

type SystemVoiceAssetRouteProps = {
  readonly params: Promise<{
    readonly file: string;
  }>;
};

export async function GET(
  _request: Request,
  { params }: SystemVoiceAssetRouteProps,
) {
  const { file } = await params;
  const dataDir = process.env.KIVA_DATA_DIR ?? "kivdb";
  const definitions = await createLibraryRepository(dataDir).getPresenters();
  if (!declaredPresenterVoiceFiles(definitions).has(file)) {
    notFound();
  }

  const content = await readFile(
    join(dataDir, "assets", "voice", "system", file),
  );

  return new Response(content, {
    headers: {
      "cache-control": "public, max-age=31536000, immutable",
      "content-type": "audio/mpeg",
    },
  });
}
