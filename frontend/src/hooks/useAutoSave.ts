/**
 * Auto-save hook: debounced save to server when project is dirty.
 *
 * Only active when authenticated and a server-side project is loaded.
 * Shows toast notifications for save success/failure and conflict detection.
 * Retries automatically when the browser comes back online after a network error.
 */
import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";

import { useAuth } from "./useAuth";
import { useProjectStore } from "../store/projectStore";
import { useToastStore } from "../store/toastStore";
import { updateProject, ConflictError, SessionExpiredError } from "../lib/backend";

const AUTO_SAVE_DELAY_MS = 5_000;
const SUCCESS_TOAST_DURATION_MS = 2_000;

export function useAutoSave(): void {
  const { t } = useTranslation("common");
  const auth = useAuth();
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const pendingRetryRef = useRef(false);
  const addToast = useToastStore((s) => s.addToast);

  const isDirty = useProjectStore((s) => s.isDirty);
  const activeProjectId = useProjectStore((s) => s.activeProjectId);
  const project = useProjectStore((s) => s.project);
  const serverUpdatedAt = useProjectStore((s) => s.serverUpdatedAt);

  // Save function extracted so it can be called from both the timer and the online handler.
  const saveRef = useRef<(() => Promise<void>) | null>(null);
  saveRef.current = async () => {
    const state = useProjectStore.getState();
    const id = state.activeProjectId;
    if (!id || !state.isDirty) return;

    try {
      const response = await updateProject(id, {
        name: state.project.info.name || undefined,
        project_data: state.project,
        expected_updated_at: state.serverUpdatedAt ?? undefined,
      });
      useProjectStore.setState((prev) => {
        if (prev.activeProjectId === id) {
          return { isDirty: false, serverUpdatedAt: response.updated_at };
        }
        return {};
      });
      pendingRetryRef.current = false;
      addToast(t("saveStatus.saved"), "success", SUCCESS_TOAST_DURATION_MS);
    } catch (err) {
      if (err instanceof SessionExpiredError) {
        // Authentik session expired — saving to the server is impossible
        // until the user logs in again. A reload triggers the login flow.
        pendingRetryRef.current = false;
        addToast(t("saveStatus.sessionExpired"), "error", 8000, {
          label: t("saveStatus.loginAgain"),
          onClick: () => window.location.reload(),
        });
      } else if (err instanceof ConflictError) {
        pendingRetryRef.current = false;
        useProjectStore.setState({ hasConflict: true });
      } else if (!navigator.onLine) {
        // Network error — mark for retry when online.
        pendingRetryRef.current = true;
        addToast(t("saveStatus.offline"), "info");
      } else {
        const detail = err instanceof Error ? err.message : String(err);
        addToast(t("saveStatus.failed", { detail }), "error");
      }
    }
  };

  // Retry on reconnect.
  useEffect(() => {
    const handleOnline = () => {
      if (pendingRetryRef.current) {
        addToast(t("saveStatus.reconnected"), "info", 2000);
        saveRef.current?.();
      }
    };
    window.addEventListener("online", handleOnline);
    return () => window.removeEventListener("online", handleOnline);
  }, [addToast, t]);

  // Debounced auto-save.
  useEffect(() => {
    if (timerRef.current) {
      clearTimeout(timerRef.current);
      timerRef.current = null;
    }

    if (!auth.isLoggedIn || !activeProjectId || !isDirty) {
      return;
    }

    timerRef.current = setTimeout(() => {
      saveRef.current?.();
    }, AUTO_SAVE_DELAY_MS);

    return () => {
      if (timerRef.current) {
        clearTimeout(timerRef.current);
      }
    };
  }, [auth.isLoggedIn, activeProjectId, isDirty, project, serverUpdatedAt]);
}
