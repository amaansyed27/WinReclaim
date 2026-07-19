import type { AppStep } from "../App";
import { UpdateControl } from "../features/update/UpdateControl";

interface SidebarProps {
  current: AppStep;
  available: Set<AppStep>;
  scanning: boolean;
  onNavigate: (step: AppStep) => void;
}

const items: { id: AppStep; label: string; icon: JSX.Element }[] = [
  {
    id: "scan",
    label: "Scan",
    icon: <path d="M9.5 3.5a6 6 0 1 0 0 12 6 6 0 0 0 0-12Zm4.3 10.3L18 18" />
  },
  {
    id: "findings",
    label: "Findings",
    icon: <><path d="M4 5h12M4 10h12M4 15h8" /><circle cx="17" cy="15" r="1" /></>
  },
  {
    id: "plan",
    label: "Review",
    icon: <><path d="M5 3h10v15H5z" /><path d="m8 8 1.5 1.5L13 6M8 13h5" /></>
  },
  {
    id: "receipt",
    label: "Receipt",
    icon: <><path d="M5 3h10v15l-2-1.5-2 1.5-2-1.5L7 18 5 16.5V3Z" /><path d="M8 7h4M8 11h4" /></>
  }
];

export function Sidebar({ current, available, scanning, onNavigate }: SidebarProps) {
  return (
    <aside className="sidebar">
      <div className="sidebar-brand" aria-label="WinReclaim">
        <div className="sidebar-brand-mark" aria-hidden="true">
          <span />
          <span />
          <span />
        </div>
        <div>
          <strong>WinReclaim</strong>
          <span>Storage control</span>
        </div>
      </div>

      <nav className="sidebar-nav" aria-label="Application pages">
        <span className="sidebar-section-label">Workspace</span>
        {items.map((item) => {
          const enabled = available.has(item.id);
          return (
            <button
              key={item.id}
              type="button"
              className={`sidebar-nav-item ${current === item.id ? "is-active" : ""}`}
              onClick={() => onNavigate(item.id)}
              disabled={!enabled}
              aria-current={current === item.id ? "page" : undefined}
            >
              <svg viewBox="0 0 21 21" aria-hidden="true">{item.icon}</svg>
              <span>{item.label}</span>
              {item.id === "scan" && scanning && <i className="sidebar-activity" aria-label="Scanning" />}
            </button>
          );
        })}
      </nav>

      <div className="sidebar-footer">
        <div className="local-status">
          <span className="local-status-dot" />
          <div>
            <strong>Local mode</strong>
            <span>Paths stay on device</span>
          </div>
        </div>
        <UpdateControl />
      </div>
    </aside>
  );
}
