import { randomUUID } from "node:crypto";
import { mkdir, readFile, rm, stat, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { setTimeout } from "node:timers/promises";

const OWNER_FILE = "owner.json";
const OWNERLESS_STALE_AFTER_MS = 5_000;

export async function acquireDirectoryLock(
  path: string,
): Promise<() => Promise<void>> {
  await mkdir(dirname(path), { recursive: true });
  const token = randomUUID();
  const ownerPath = join(path, OWNER_FILE);

  while (true) {
    try {
      await mkdir(path);
      await writeFile(
        ownerPath,
        JSON.stringify({ pid: process.pid, token, createdAt: Date.now() }),
        "utf8",
      );
      return async () => {
        const owner = await readLockOwner(ownerPath);
        if (owner?.token === token) {
          await rm(path, { recursive: true, force: true });
        }
      };
    } catch (error) {
      if (isNodeError(error) && error.code === "EEXIST") {
        if (await lockIsStale(path, ownerPath)) {
          await rm(path, { recursive: true, force: true });
          continue;
        }
        await setTimeout(10);
        continue;
      }
      throw error;
    }
  }
}

type LockOwner = {
  readonly pid: number;
  readonly token: string;
};

async function readLockOwner(path: string): Promise<LockOwner | null> {
  try {
    const value = JSON.parse(await readFile(path, "utf8")) as Partial<LockOwner>;
    return Number.isInteger(value.pid) && typeof value.token === "string"
      ? (value as LockOwner)
      : null;
  } catch {
    return null;
  }
}

async function lockIsStale(path: string, ownerPath: string): Promise<boolean> {
  const owner = await readLockOwner(ownerPath);
  if (owner) return !processIsAlive(owner.pid);
  try {
    return Date.now() - (await stat(path)).mtimeMs > OWNERLESS_STALE_AFTER_MS;
  } catch (error) {
    return isNodeError(error) && error.code === "ENOENT";
  }
}

function processIsAlive(pid: number): boolean {
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
}

function isNodeError(error: unknown): error is NodeJS.ErrnoException {
  return error instanceof Error && "code" in error;
}
