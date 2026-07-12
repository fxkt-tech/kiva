import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { afterEach, describe, expect, it } from "vitest";
import { rangedFileResponse } from "./ranged-file-response";

const roots: string[] = [];

afterEach(async () => {
  await Promise.all(roots.splice(0).map((root) => rm(root, { recursive: true })));
});

describe("ranged file response", () => {
  it("serves a seekable byte range", async () => {
    const root = await mkdtemp(join(tmpdir(), "kiva-range-"));
    roots.push(root);
    const path = join(root, "audio.mp3");
    await writeFile(path, Buffer.from("0123456789"));
    const response = await rangedFileResponse({
      request: new Request("http://localhost/audio", {
        headers: { range: "bytes=2-5" },
      }),
      path,
      headers: { "content-type": "audio/mpeg" },
    });

    expect(response.status).toBe(206);
    expect(response.headers.get("accept-ranges")).toBe("bytes");
    expect(response.headers.get("content-range")).toBe("bytes 2-5/10");
    expect(response.headers.get("content-length")).toBe("4");
    expect(await response.text()).toBe("2345");
  });

  it("rejects an unsatisfiable range", async () => {
    const root = await mkdtemp(join(tmpdir(), "kiva-range-"));
    roots.push(root);
    const path = join(root, "audio.mp3");
    await writeFile(path, Buffer.from("0123"));
    const response = await rangedFileResponse({
      request: new Request("http://localhost/audio", {
        headers: { range: "bytes=9-10" },
      }),
      path,
      headers: { "content-type": "audio/mpeg" },
    });

    expect(response.status).toBe(416);
    expect(response.headers.get("content-range")).toBe("bytes */4");
  });
});
