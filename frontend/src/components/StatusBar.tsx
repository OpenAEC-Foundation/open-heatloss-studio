import { useTranslation } from "react-i18next";

import { useProjectStore } from "../store/projectStore";
import { useSaveStatusStore } from "../store/saveStatusStore";
import "./StatusBar.css";

/**
 * Persistente server-save-statusindicator (vervangt het stille falen van de
 * auto-save). Alleen zichtbaar wanneer een serverproject actief is. Toont
 * "bezig" / "opgeslagen HH:MM" / "niet opgeslagen (offline/fout)" met een
 * retry-knop / "conflict". Gevoed door `useSaveStatusStore` via de gedeelde
 * save-helpers in `lib/serverProjects.ts`.
 */
function SaveStatusIndicator() {
  const { t } = useTranslation();
  const activeProjectId = useProjectStore((s) => s.activeProjectId);
  const status = useSaveStatusStore((s) => s.status);
  const lastSavedAt = useSaveStatusStore((s) => s.lastSavedAt);
  const errorDetail = useSaveStatusStore((s) => s.errorDetail);
  const retry = useSaveStatusStore((s) => s.retry);
  const hasRetryHandler = useSaveStatusStore((s) => s.retryHandler !== null);

  if (!activeProjectId || status === "idle") return null;

  let label: string;
  let warning = false;
  switch (status) {
    case "saving":
      label = t("saveStatus.saving");
      break;
    case "saved": {
      const time = lastSavedAt
        ? new Date(lastSavedAt).toLocaleTimeString([], {
            hour: "2-digit",
            minute: "2-digit",
          })
        : "";
      label = t("saveStatus.savedAt", { time });
      break;
    }
    case "offline":
      label = t("saveStatus.notSavedOffline");
      warning = true;
      break;
    case "error":
      label = t("saveStatus.notSavedError");
      warning = true;
      break;
    case "conflict":
      label = t("saveStatus.conflictShort");
      warning = true;
      break;
  }

  // Retry-knop alleen tonen wanneer er daadwerkelijk een handler is
  // geregistreerd (door useAutoSave) — anders is de knop een stille no-op.
  const canRetry =
    (status === "offline" || status === "error") && hasRetryHandler;

  return (
    <>
      <div className="status-separator" />
      <div className="status-item" title={errorDetail ?? undefined}>
        <span
          className="status-item-label"
          style={warning ? { color: "var(--theme-warning, currentColor)" } : undefined}
        >
          {label}
        </span>
        {canRetry && (
          <button
            type="button"
            onClick={retry}
            className="status-item-value"
            style={{
              background: "none",
              border: "none",
              cursor: "pointer",
              padding: 0,
              textDecoration: "underline",
              color: "inherit",
              font: "inherit",
            }}
          >
            {t("saveStatus.retry")}
          </button>
        )}
      </div>
    </>
  );
}

export default function StatusBar() {
  const { t } = useTranslation();

  return (
    <div className="status-bar">
      <div className="status-bar-left">
        <div className="status-item">
          <span className="status-item-label">{t("ready")}</span>
        </div>
        <SaveStatusIndicator />
        <div className="status-separator" />
        <div className="status-item">
          <span className="status-item-label">{t("items")}:</span>
          <span className="status-item-value">0</span>
        </div>
      </div>

      <div className="status-bar-center">
        <span className="status-item-label" style={{ fontSize: "11px" }}>
          Warmteverlies v{__APP_VERSION__}
        </span>
      </div>

      <div className="status-bar-right">
        <div className="status-item">
          <span className="status-item-label">{t("zoom")}:</span>
          <span className="status-item-value">100%</span>
        </div>
      </div>
    </div>
  );
}
