import { extname } from "node:path";
import { exportErrorResponse } from "@/server/video-export/http";
import { getVideoExportService } from "@/server/video-export/export-service";
import { rangedFileResponse } from "@/server/http/ranged-file-response";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

type RouteProps = {
  readonly params: Promise<{
    readonly jobId: string;
    readonly path: readonly string[];
  }>;
};

export async function GET(request: Request, { params }: RouteProps) {
  try {
    const { jobId, path } = await params;
    const file = await getVideoExportService().assetPath(jobId, path);
    return await rangedFileResponse({
      request,
      path: file,
      headers: {
        "access-control-allow-origin": "*",
        "cache-control": "private, max-age=31536000, immutable",
        "content-type": contentType(file),
      },
    });
  } catch (error) {
    return exportErrorResponse(error);
  }
}

function contentType(path: string): string {
  switch (extname(path).toLowerCase()) {
    case ".png":
      return "image/png";
    case ".mp3":
      return "audio/mpeg";
    case ".ttf":
      return "font/ttf";
    default:
      return "application/octet-stream";
  }
}
