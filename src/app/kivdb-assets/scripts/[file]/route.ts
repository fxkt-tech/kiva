import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { notFound } from "next/navigation";

export async function GET(
  _request: Request,
  { params }: { readonly params: Promise<{ readonly file: string }> },
) {
  const { file } = await params;
  if (!/^script_[a-z0-9_-]+_v\d+\.png$/i.test(file)) notFound();
  const dataDir = process.env.KIVA_DATA_DIR ?? "kivdb";
  const content = await readFile(join(dataDir, "assets", "scripts", file));
  return new Response(content, {
    headers: {
      "cache-control": "public, max-age=31536000, immutable",
      "content-type": "image/png",
    },
  });
}
