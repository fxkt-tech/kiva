import { exportErrorResponse } from "@/server/video-export/http";
import { getVideoExportService } from "@/server/video-export/export-service";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

type RouteProps = {
  readonly params: Promise<{ readonly gameId: string }>;
};

export async function GET(_request: Request, { params }: RouteProps) {
  try {
    const { gameId } = await params;
    return Response.json({
      jobs: await getVideoExportService().list(gameId),
    });
  } catch (error) {
    return exportErrorResponse(error);
  }
}

export async function POST(_request: Request, { params }: RouteProps) {
  try {
    const { gameId } = await params;
    return Response.json(
      { job: await getVideoExportService().create(gameId) },
      { status: 202 },
    );
  } catch (error) {
    return exportErrorResponse(error);
  }
}
