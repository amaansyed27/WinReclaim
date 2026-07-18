import { getVersion } from "@tauri-apps/api/app";
import { relaunch } from "@tauri-apps/plugin-process";
import { check } from "@tauri-apps/plugin-updater";

type PendingUpdate = NonNullable<Awaited<ReturnType<typeof check>>>;

export interface AvailableUpdate {
  currentVersion: string;
  version: string;
  date?: string | null;
  notes?: string | null;
}

export interface UpdateProgress {
  downloadedBytes: number;
  totalBytes: number | null;
  percent: number | null;
}

let pendingUpdate: PendingUpdate | null = null;

export async function currentAppVersion(): Promise<string> {
  return getVersion();
}

export async function checkForAppUpdate(): Promise<AvailableUpdate | null> {
  const update = await check({ timeout: 15_000 });
  pendingUpdate = update;

  if (!update) return null;

  return {
    currentVersion: update.currentVersion,
    version: update.version,
    date: update.date,
    notes: update.body
  };
}

export async function installPendingUpdate(
  onProgress: (progress: UpdateProgress) => void
): Promise<void> {
  if (!pendingUpdate) {
    throw new Error("No downloaded update is ready. Check for updates again.");
  }

  let downloadedBytes = 0;
  let totalBytes: number | null = null;

  await pendingUpdate.downloadAndInstall((event) => {
    if (event.event === "Started") {
      totalBytes = event.data.contentLength ?? null;
    }

    if (event.event === "Progress") {
      downloadedBytes += event.data.chunkLength;
    }

    const percent = totalBytes
      ? Math.min(100, Math.round((downloadedBytes / totalBytes) * 100))
      : null;

    onProgress({ downloadedBytes, totalBytes, percent });
  });

  pendingUpdate = null;
  await relaunch();
}
