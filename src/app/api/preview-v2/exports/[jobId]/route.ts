import { exportErrorResponse } from "@/server/video-export/http";
import {
  ExportServiceError,
  getVideoExportService,
} from "@/server/video-export/export-service";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

type RouteProps = {
  readonly params: Promise<{ readonly jobId: string }>;
};

export async function GET(_request: Request, { params }: RouteProps) {
  try {
    const { jobId } = await params;
    const job = await getVideoExportService().get(jobId);
    if (!job) {
      throw new ExportServiceError("job_not_found", "Export job not found.", 404);
    }
    return Response.json({ job });
  } catch (error) {
    return exportErrorResponse(error);
  }
}
