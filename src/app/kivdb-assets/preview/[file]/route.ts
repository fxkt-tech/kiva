import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { notFound } from "next/navigation";

const allowedFiles = new Set(["day-background.png", "night-background.png"]);

type PreviewAssetRouteProps = {
  readonly params: Promise<{
    readonly file: string;
  }>;
};

export async function GET(_request: Request, { params }: PreviewAssetRouteProps) {
  const { file } = await params;
  if (!allowedFiles.has(file)) {
    notFound();
  }

  const dataDir = process.env.KIVA_DATA_DIR ?? "kivdb";
  const content = await readFile(join(dataDir, "assets", "preview", file));

  return new Response(content, {
    headers: {
      "cache-control": "public, max-age=31536000, immutable",
      "content-type": "image/png",
    },
  });
}
