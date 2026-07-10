import { exportErrorResponse } from "@/server/video-export/http";
import { getVideoExportService } from "@/server/video-export/export-service";

export const runtime = "nodejs";

type RouteProps = {
  readonly params: Promise<{ readonly jobId: string }>;
};

export async function POST(_request: Request, { params }: RouteProps) {
  try {
    const { jobId } = await params;
    return Response.json({
      job: await getVideoExportService().cancel(jobId),
    });
  } catch (error) {
    return exportErrorResponse(error);
  }
}
