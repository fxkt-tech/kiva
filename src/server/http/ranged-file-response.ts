import { readFile } from "node:fs/promises";

export async function rangedFileResponse(input: {
  readonly request: Request;
  readonly path: string;
  readonly headers: Readonly<Record<string, string>>;
}): Promise<Response> {
  const content = await readFile(input.path);
  const headers = new Headers(input.headers);
  headers.set("accept-ranges", "bytes");
  const range = input.request.headers.get("range");
  if (!range) {
    headers.set("content-length", String(content.byteLength));
    return new Response(content, { headers });
  }
  const parsed = parseRange(range, content.byteLength);
  if (!parsed) {
    headers.set("content-range", `bytes */${content.byteLength}`);
    headers.set("content-length", "0");
    return new Response(null, { status: 416, headers });
  }
  const body = content.subarray(parsed.start, parsed.end + 1);
  headers.set("content-length", String(body.byteLength));
  headers.set(
    "content-range",
    `bytes ${parsed.start}-${parsed.end}/${content.byteLength}`,
  );
  return new Response(body, { status: 206, headers });
}

function parseRange(
  value: string,
  size: number,
): { readonly start: number; readonly end: number } | null {
  const match = /^bytes=(\d*)-(\d*)$/.exec(value.trim());
  if (!match || size <= 0) return null;
  const [, startText, endText] = match;
  if (!startText && !endText) return null;
  if (!startText) {
    const suffixLength = Number(endText);
    if (!Number.isInteger(suffixLength) || suffixLength <= 0) return null;
    return { start: Math.max(0, size - suffixLength), end: size - 1 };
  }
  const start = Number(startText);
  const requestedEnd = endText ? Number(endText) : size - 1;
  if (
    !Number.isInteger(start) ||
    !Number.isInteger(requestedEnd) ||
    start < 0 ||
    start >= size ||
    requestedEnd < start
  ) return null;
  return { start, end: Math.min(requestedEnd, size - 1) };
}
