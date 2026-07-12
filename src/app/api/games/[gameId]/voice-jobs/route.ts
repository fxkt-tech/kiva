import { getVoiceJobService, VoiceJobError } from "@/server/voice-jobs/service";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

type RouteProps = { readonly params: Promise<{ readonly gameId: string }> };

export async function GET(_request: Request, { params }: RouteProps) {
  try {
    const { gameId } = await params;
    return Response.json({ jobs: await getVoiceJobService().list(gameId) });
  } catch (error) {
    return errorResponse(error);
  }
}

export async function POST(_request: Request, { params }: RouteProps) {
  try {
    const { gameId } = await params;
    return Response.json(
      { job: await getVoiceJobService().create(gameId) },
      { status: 202 },
    );
  } catch (error) {
    return errorResponse(error);
  }
}

function errorResponse(error: unknown): Response {
  const known = error instanceof VoiceJobError;
  return Response.json(
    {
      error: {
        code: known ? error.code : "voice_job_failed",
        message: known ? error.message : "Voice job request failed",
      },
    },
    { status: known ? error.httpStatus : 500 },
  );
}
