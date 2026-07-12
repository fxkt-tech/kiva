import { VideoExportService } from "@/server/video-export/export-service";
import { runPersistedWorker } from "@/server/workers/runtime";
import { setPriority } from "node:os";

const dataDir = process.env.KIVA_DATA_DIR ?? "kivdb";
const service = new VideoExportService(dataDir, { executeInline: false });
try {
  setPriority(0, 10);
} catch {
  // Process isolation remains effective when the OS does not allow reprioritization.
}

void runPersistedWorker({
  dataDir,
  name: "video",
  processOnce: () => service.processQueuedOnce(),
});
