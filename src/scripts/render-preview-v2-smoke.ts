import { mkdir, rm } from "node:fs/promises";
import { resolve } from "node:path";
import { renderVideo } from "@/server/video-export/remotion-renderer";
import { defaultCompositionInput } from "@/video/root";

async function main() {
  const outputDir = resolve(process.cwd(), ".tmp", "preview-v2-smoke");
  const outputPath = resolve(outputDir, "smoke.mp4");
  await mkdir(outputDir, { recursive: true });
  await rm(outputPath, { force: true });

  const handle = renderVideo({
    composition: defaultCompositionInput,
    outputPath,
    publicDir: resolve(process.cwd(), "kivdb", "assets"),
    onProgress: ({ progress, stage }) => {
      process.stdout.write(
        "\r" + stage.padEnd(10) + " " + String(Math.round(progress * 100)).padStart(3) + "%",
      );
    },
  });
  await handle.promise;
  process.stdout.write("\n" + outputPath + "\n");
}

main().catch((error: unknown) => {
  console.error(error);
  process.exitCode = 1;
});
