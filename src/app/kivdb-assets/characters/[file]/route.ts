import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { notFound } from "next/navigation";

type ActorAssetRouteProps = {
  readonly params: Promise<{
    readonly file: string;
  }>;
};

export async function GET(
  _request: Request,
  { params }: ActorAssetRouteProps,
) {
  const { file } = await params;
  if (!/^[a-z0-9_-]+\.png$/i.test(file)) {
    notFound();
  }

  const dataDir = process.env.KIVA_DATA_DIR ?? "kivdb";
  const content = await readFile(join(dataDir, "assets", "characters", file));

  return new Response(content, {
    headers: {
      "cache-control": "public, max-age=31536000, immutable",
      "content-type": "image/png",
    },
  });
}
