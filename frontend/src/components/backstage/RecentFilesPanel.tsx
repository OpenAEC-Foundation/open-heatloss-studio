/**
 * Backstage Recent-files paneel.
 *
 * Toont de tien laatst-geopende projecten rechts in Backstage wanneer de
 * gebruiker links op de "Recent" menu-item klikt. Mirror van het patroon
 * van `AboutPanel` en `ExtensionManagerPanel`.
 *
 * Klikken op een entry roept de meegegeven `onOpen` callback aan; daar zit
 * de Tauri `readTextFile` + import-pipeline in (Backstage levert hem).
 */
import { useTranslation } from "react-i18next";

import { useRecentFilesStore, type RecentFile } from "../../store/recentFilesStore";

interface RecentFilesPanelProps {
  /** Roep aan om de geselecteerde entry te openen. */
  onOpen: (entry: RecentFile) => void | Promise<void>;
}

function formatRelative(iso: string): string {
  const t = new Date(iso).getTime();
  if (Number.isNaN(t)) return iso;
  const diff = Date.now() - t;
  const sec = Math.floor(diff / 1000);
  if (sec < 60) return "zojuist";
  const min = Math.floor(sec / 60);
  if (min < 60) return `${min} min geleden`;
  const hr = Math.floor(min / 60);
  if (hr < 24) return `${hr} uur geleden`;
  const days = Math.floor(hr / 24);
  if (days < 7) return `${days} dag${days === 1 ? "" : "en"} geleden`;
  return new Date(t).toLocaleDateString("nl-NL", {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

export default function RecentFilesPanel({ onOpen }: RecentFilesPanelProps) {
  const { t } = useTranslation("backstage");
  const recent = useRecentFilesStore((s) => s.recent);
  const remove = useRecentFilesStore((s) => s.remove);
  const clear = useRecentFilesStore((s) => s.clear);

  return (
    <div className="recent-panel" style={{ padding: "20px 24px" }}>
      <div
        style={{
          display: "flex",
          alignItems: "baseline",
          justifyContent: "space-between",
          marginBottom: 12,
        }}
      >
        <h2 style={{ fontSize: 20, fontWeight: 600, margin: 0 }}>
          {t("recent")}
        </h2>
        {recent.length > 0 && (
          <button
            type="button"
            onClick={() => clear()}
            style={{
              background: "none",
              border: "none",
              color: "var(--theme-text-muted)",
              fontSize: 12,
              cursor: "pointer",
              padding: "4px 8px",
            }}
            title="Lijst wissen"
          >
            {t("recentClear")}
          </button>
        )}
      </div>

      {recent.length === 0 ? (
        <div
          style={{
            padding: "32px 16px",
            textAlign: "center",
            border: "1px dashed var(--theme-border)",
            borderRadius: 8,
            color: "var(--theme-text-muted)",
            fontSize: 13,
          }}
        >
          <p style={{ margin: "0 0 8px" }}>{t("recentEmpty")}</p>
          <p style={{ margin: 0, fontSize: 11 }}>{t("recentEmptyHint")}</p>
        </div>
      ) : (
        <ul
          style={{
            listStyle: "none",
            margin: 0,
            padding: 0,
            display: "flex",
            flexDirection: "column",
            gap: 4,
          }}
        >
          {recent.map((entry) => (
            <li
              key={(entry.path ?? "") + entry.fileName + entry.openedAt}
              style={{
                display: "flex",
                alignItems: "center",
                gap: 12,
                padding: "10px 12px",
                borderRadius: 6,
                background: "var(--theme-bg-lighter)",
              }}
            >
              <button
                type="button"
                onClick={() => onOpen(entry)}
                style={{
                  flex: 1,
                  minWidth: 0,
                  background: "none",
                  border: "none",
                  padding: 0,
                  textAlign: "left",
                  cursor: "pointer",
                  color: "inherit",
                }}
                title={entry.path ?? entry.fileName}
              >
                <div
                  style={{
                    fontSize: 14,
                    fontWeight: 500,
                    color: "var(--theme-text)",
                    whiteSpace: "nowrap",
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                  }}
                >
                  {entry.name}
                </div>
                <div
                  style={{
                    fontSize: 11,
                    color: "var(--theme-text-muted)",
                    whiteSpace: "nowrap",
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                  }}
                >
                  {entry.path ?? entry.fileName}
                </div>
              </button>
              <span
                style={{
                  fontSize: 11,
                  color: "var(--theme-text-muted)",
                  whiteSpace: "nowrap",
                }}
              >
                {formatRelative(entry.openedAt)}
              </span>
              <button
                type="button"
                onClick={() => remove(entry)}
                style={{
                  background: "none",
                  border: "none",
                  color: "var(--theme-text-muted)",
                  cursor: "pointer",
                  padding: "4px 6px",
                  fontSize: 14,
                }}
                title={t("recentRemove")}
                aria-label={t("recentRemove")}
              >
                ✕
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
