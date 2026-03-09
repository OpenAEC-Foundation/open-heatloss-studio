import { useEffect, useRef, useState } from "react";
import { isTauri } from "../../lib/backend";
import { useThemeStore, THEME_LABELS, type Theme } from "../../store/themeStore";

/**
 * Custom Windows-style titlebar for Tauri.
 *
 * Uses `data-tauri-drag-region` for native drag behavior including:
 * - Drag to move window
 * - Double-click to maximize/restore
 * - Drag to screen edges for snap layouts
 * - Right-click for system context menu
 *
 * Only renders in Tauri desktop builds.
 */
export function TitleBar() {
  if (!isTauri()) return null;
  return <TitleBarInner />;
}

/** Theme icon per theme type. */
function ThemeIcon({ theme }: { theme: string }) {
  switch (theme) {
    case "light":
      return (
        /* Sun */
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
          <circle cx="12" cy="12" r="5" />
          <line x1="12" y1="1" x2="12" y2="3" />
          <line x1="12" y1="21" x2="12" y2="23" />
          <line x1="4.22" y1="4.22" x2="5.64" y2="5.64" />
          <line x1="18.36" y1="18.36" x2="19.78" y2="19.78" />
          <line x1="1" y1="12" x2="3" y2="12" />
          <line x1="21" y1="12" x2="23" y2="12" />
          <line x1="4.22" y1="19.78" x2="5.64" y2="18.36" />
          <line x1="18.36" y1="5.64" x2="19.78" y2="4.22" />
        </svg>
      );
    case "blue":
      return (
        /* Droplet */
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
          <path d="M12 2.69l5.66 5.66a8 8 0 11-11.31 0z" />
        </svg>
      );
    case "highContrast":
      return (
        /* Eye */
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
          <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
          <circle cx="12" cy="12" r="3" />
        </svg>
      );
    default:
      return (
        /* Moon (dark) */
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
          <path d="M21 12.79A9 9 0 1111.21 3 7 7 0 0021 12.79z" />
        </svg>
      );
  }
}

const ALL_THEMES: Theme[] = ["dark", "light", "blue", "highContrast"];

function ThemeDropdown() {
  const { theme, setTheme } = useThemeStore();
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  // Close on click outside
  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [open]);

  return (
    <div ref={ref} className="relative h-full">
      <button
        onClick={() => setOpen(!open)}
        className="inline-flex h-full w-[40px] items-center justify-center text-app-titlebar-text hover:bg-app-hover hover:text-app-text active:opacity-80"
        aria-label={`Theme: ${THEME_LABELS[theme]}`}
        title={`Theme: ${THEME_LABELS[theme]}`}
      >
        {/* Palette icon — does not change per theme */}
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
          <circle cx="13.5" cy="6.5" r="1.5" fill="currentColor" stroke="none" />
          <circle cx="17.5" cy="10.5" r="1.5" fill="currentColor" stroke="none" />
          <circle cx="8.5" cy="7.5" r="1.5" fill="currentColor" stroke="none" />
          <circle cx="6.5" cy="12.5" r="1.5" fill="currentColor" stroke="none" />
          <path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.9 0 1.5-.7 1.5-1.5 0-.4-.1-.7-.4-1-.3-.3-.4-.7-.4-1.1 0-.8.7-1.5 1.5-1.5H16c3.3 0 6-2.7 6-6 0-5.5-4.5-9-10-9z" />
        </svg>
      </button>

      {open && (
        <div className="absolute right-0 top-[32px] z-[60] min-w-[160px] rounded-md border border-app-border bg-app-dropdown-bg py-1 shadow-lg">
          {ALL_THEMES.map((t) => (
            <button
              key={t}
              onClick={() => { setTheme(t); setOpen(false); }}
              className={`flex w-full items-center gap-2.5 px-3 py-1.5 text-xs transition-colors ${
                t === theme
                  ? "bg-app-active text-white font-medium"
                  : "text-app-text hover:bg-app-hover"
              }`}
            >
              <ThemeIcon theme={t} />
              {THEME_LABELS[t]}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

function TitleBarInner() {
  const [isMaximized, setIsMaximized] = useState(false);
  const { theme } = useThemeStore();

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    (async () => {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      const win = getCurrentWindow();

      setIsMaximized(await win.isMaximized());

      const { listen } = await import("@tauri-apps/api/event");
      const unlistenFn = await listen("tauri://resize", async () => {
        setIsMaximized(await win.isMaximized());
      });
      unlisten = unlistenFn;
    })();

    return () => { unlisten?.(); };
  }, []);

  const windowAction = async (action: "minimize" | "toggleMaximize" | "close") => {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    const win = getCurrentWindow();
    await win[action]();
  };

  return (
    <div
      className="fixed top-0 left-0 right-0 z-50 flex h-[32px] select-none items-center bg-app-titlebar-bg"
      data-tauri-drag-region
    >
      {/* Centered title — absolutely positioned so it stays centered */}
      <div className="absolute inset-0 flex items-center justify-center pointer-events-none" data-tauri-drag-region>
        <div className="flex items-center gap-2">
          <div
            className="h-4 w-4 rounded-sm"
            style={{ background: "var(--gradient-amber, #D97706)" }}
          />
          <span className="text-xs text-app-titlebar-text font-sans">
            ISSO 51 Warmteverliesberekening
          </span>
          <span className="text-[10px] text-app-text-muted font-sans">v0.1.0</span>
        </div>
      </div>

      {/* Spacer pushes buttons to the right */}
      <div className="flex-1 h-full" data-tauri-drag-region />

      {/* Window controls — Windows 11 style: 46px wide, 32px tall */}
      <div className="flex h-full">
        {/* Theme selector dropdown */}
        <ThemeDropdown />

        <button
          onClick={() => windowAction("minimize")}
          className="inline-flex h-full w-[46px] items-center justify-center text-app-titlebar-text hover:bg-app-hover hover:text-app-text active:opacity-80"
          aria-label="Minimize"
        >
          <svg width="10" height="1" viewBox="0 0 10 1">
            <path d="M0 0h10v1H0z" fill="currentColor" />
          </svg>
        </button>

        <button
          onClick={() => windowAction("toggleMaximize")}
          className="inline-flex h-full w-[46px] items-center justify-center text-app-titlebar-text hover:bg-app-hover hover:text-app-text active:opacity-80"
          aria-label={isMaximized ? "Restore Down" : "Maximize"}
        >
          {isMaximized ? (
            <svg width="10" height="10" viewBox="0 0 10 10">
              <path d="M3 1h6v6H8V2H3V1z" fill="currentColor" />
              <rect x="1" y="3" width="6" height="6" fill="none" stroke="currentColor" strokeWidth="1" />
            </svg>
          ) : (
            <svg width="10" height="10" viewBox="0 0 10 10">
              <rect x="1" y="1" width="8" height="8" fill="none" stroke="currentColor" strokeWidth="1" />
            </svg>
          )}
        </button>

        <button
          onClick={() => windowAction("close")}
          className="inline-flex h-full w-[46px] items-center justify-center text-app-titlebar-text hover:bg-[#c42b1c] hover:text-white active:bg-[#b22a1c]"
          aria-label="Close"
        >
          <svg width="10" height="10" viewBox="0 0 10 10">
            <path d="M1 1l8 8M9 1l-8 8" stroke="currentColor" strokeWidth="1" fill="none" />
          </svg>
        </button>
      </div>
    </div>
  );
}

/** Height of the custom titlebar in pixels. */
export const TITLEBAR_HEIGHT = 32;
