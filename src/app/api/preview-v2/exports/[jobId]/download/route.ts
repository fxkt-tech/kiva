import { createReadStream } from "node:fs";
import { Readable } from "node:stream";
import { exportErrorResponse } from "@/server/video-export/http";
import { getVideoExportService } from "@/server/video-export/export-service";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

type RouteProps = {
  readonly params: Promise<{ readonly jobId: string }>;
};

export async function GET(_request: Request, { params }: RouteProps) {
  try {
    const { jobId } = await params;
    const output = await getVideoExportService().outputPath(jobId);
    const stream = Readable.toWeb(createReadStream(output.path));
    return new Response(stream as ReadableStream<Uint8Array>, {
      headers: {
        "cache-control": "private, no-store",
        "content-length": String(output.job.output?.sizeBytes ?? 0),
        "content-disposition":
          'attachment; filename="' + (output.job.output?.fileName ?? "kiva.mp4") + '"',
        "content-type": "video/mp4",
      },
    });
  } catch (error) {
    return exportErrorResponse(error);
  }
}
