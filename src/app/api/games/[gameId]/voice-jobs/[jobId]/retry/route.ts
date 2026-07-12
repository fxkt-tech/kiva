import { getVoiceJobService, VoiceJobError } from "@/server/voice-jobs/service";

export const runtime = "nodejs";
type RouteProps = { readonly params: Promise<{ readonly gameId: string; readonly jobId: string }> };

export async function POST(_request: Request, { params }: RouteProps) {
  try {
    const { gameId, jobId } = await params;
    return Response.json({ job: await getVoiceJobService().retry(gameId, jobId) });
  } catch (error) {
    const known = error instanceof VoiceJobError;
    return Response.json(
      { error: { message: known ? error.message : "Voice job retry failed" } },
      { status: known ? error.httpStatus : 500 },
    );
  }
}
