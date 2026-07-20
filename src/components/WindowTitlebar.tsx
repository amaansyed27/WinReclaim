import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface WindowTitlebarProps {
  pageTitle: string;
}

const friendlyTitles: Record<string, string> = {
  "Storage scan": "Scan PC",
  "Storage Time Machine": "Storage history",
  "Reclaim Passports": "Choose items",
  "Reclaim Simulation": "Review cleanup",
  "Cleanup receipt": "Cleanup results",
  "Undo Vault": "Restore files"
};

export function WindowTitlebar({ pageTitle }: WindowTitlebarProps) {
  const [maximized, setMaximized] = useState(false);
  const displayTitle = friendlyTitles[pageTitle] ?? pageTitle;

  useEffect(() => {
    const appWindow = getCurrentWindow();
    let dispose: (() => void) | undefined;

    void appWindow.isMaximized().then(setMaximized).catch(() => undefined);
    void appWindow
      .onResized(() => {
        void appWindow.isMaximized().then(setMaximized).catch(() => undefined);
      })
      .then((unlisten) => {
        dispose = unlisten;
      })
      .catch(() => undefined);

    return () => dispose?.();
  }, []);

  async function minimize() {
    await getCurrentWindow().minimize();
  }

  async function toggleMaximize() {
    const appWindow = getCurrentWindow();
    await appWindow.toggleMaximize();
    setMaximized(await appWindow.isMaximized());
  }

  async function close() {
    await getCurrentWindow().close();
  }

  return (
    <header className="window-titlebar">
      <div className="window-drag-region" data-tauri-drag-region="deep">
        <div className="window-app-title" data-tauri-drag-region="deep">
          <span className="window-app-icon" aria-hidden="true">
            <i />
            <i />
            <i />
          </span>
          <strong data-tauri-drag-region="deep">WinReclaim</strong>
          <span className="window-title-separator" data-tauri-drag-region="deep" />
          <span data-tauri-drag-region="deep">{displayTitle}</span>
        </div>
      </div>

      <div className="window-controls" aria-label="Window controls">
        <button type="button" onClick={() => void minimize()} aria-label="Minimize">
          <svg viewBox="0 0 12 12" aria-hidden="true"><path d="M2 8.5h8" /></svg>
        </button>
        <button type="button" onClick={() => void toggleMaximize()} aria-label={maximized ? "Restore" : "Maximize"}>
          {maximized ? (
            <svg viewBox="0 0 12 12" aria-hidden="true"><path d="M3.5 4.5h5v5h-5z" /><path d="M5 4.5V3h4v4H8.5" /></svg>
          ) : (
            <svg viewBox="0 0 12 12" aria-hidden="true"><rect x="3" y="3" width="6" height="6" /></svg>
          )}
        </button>
        <button className="window-close" type="button" onClick={() => void close()} aria-label="Close">
          <svg viewBox="0 0 12 12" aria-hidden="true"><path d="m3 3 6 6M9 3 3 9" /></svg>
        </button>
      </div>
    </header>
  );
}
