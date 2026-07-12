import { mkdir, open, readFile, rm } from "node:fs/promises";
import { dirname, join } from "node:path";

export async function runPersistedWorker(input: {
  readonly dataDir: string;
  readonly name: "voice" | "video";
  readonly processOnce: () => Promise<boolean>;
  readonly idlePollMs?: number;
}): Promise<never> {
  const lockPath = join(input.dataDir, "workers", `${input.name}.lock`);
  await acquireProcessLock(lockPath);
  let stopping = false;
  const stop = () => {
    stopping = true;
  };
  process.once("SIGINT", stop);
  process.once("SIGTERM", stop);
  process.stdout.write(`${input.name} worker started (pid ${process.pid})\n`);
  try {
    while (!stopping) {
      const processed = await input.processOnce();
      if (!processed) await delay(input.idlePollMs ?? 750);
    }
  } finally {
    await rm(lockPath, { force: true });
  }
  process.exit(0);
}

async function acquireProcessLock(path: string): Promise<void> {
  await mkdir(dirname(path), { recursive: true });
  try {
    const handle = await open(path, "wx");
    await handle.writeFile(String(process.pid));
    await handle.close();
    return;
  } catch (error) {
    if (!isAlreadyExists(error)) throw error;
  }
  const owner = Number.parseInt(await readFile(path, "utf8").catch(() => ""), 10);
  if (Number.isInteger(owner) && processIsAlive(owner)) {
    throw new Error(`Another worker already owns ${path} (pid ${owner})`);
  }
  await rm(path, { force: true });
  const handle = await open(path, "wx");
  await handle.writeFile(String(process.pid));
  await handle.close();
}

function processIsAlive(pid: number): boolean {
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
}

function isAlreadyExists(error: unknown): boolean {
  return typeof error === "object" && error !== null && "code" in error && error.code === "EEXIST";
}

function delay(milliseconds: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}
