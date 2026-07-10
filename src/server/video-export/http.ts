import { ExportServiceError } from "./export-service";

export function exportErrorResponse(error: unknown): Response {
  if (error instanceof ExportServiceError) {
    return Response.json(
      { error: { code: error.code, message: error.message } },
      { status: error.status },
    );
  }
  const message = error instanceof Error ? error.message : String(error);
  console.error("Video export request failed:", error);
  return Response.json(
    {
      error: {
        code: "internal_error",
        message: message.includes("ENOENT")
          ? "Export file was not found."
          : "Video export request failed.",
      },
    },
    { status: message.includes("ENOENT") ? 404 : 500 },
  );
}
