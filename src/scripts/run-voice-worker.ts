import { VoiceJobService } from "@/server/voice-jobs/service";
import { runPersistedWorker } from "@/server/workers/runtime";

const dataDir = process.env.KIVA_DATA_DIR ?? "kivdb";
const service = new VoiceJobService(dataDir, undefined, { executeInline: false });

void runPersistedWorker({
  dataDir,
  name: "voice",
  processOnce: () => service.processQueuedOnce(),
});
