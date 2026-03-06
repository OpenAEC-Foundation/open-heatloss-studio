/**
 * Auto-save hook: debounced save to server when project is dirty.
 *
 * Only active when authenticated and a server-side project is loaded.
 * Shows toast notifications for save success/failure and conflict detection.
 * Retries automatically when the browser comes back online after a network error.
 */
import { useEffect, useRef } from "react";

import { useAuth } from "./useAuth";
import { useProjectStore } from "../store/projectStore";
import { useToastStore } from "../store/toastStore";
import { updateProject, ConflictError } from "../lib/backend";

const AUTO_SAVE_DELAY_MS = 5_000;
const SUCCESS_TOAST_DURATION_MS = 2_000;

export function useAutoSave(): void {
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
      addToast("Project opgeslagen", "success", SUCCESS_TOAST_DURATION_MS);
    } catch (err) {
      if (err instanceof ConflictError) {
        pendingRetryRef.current = false;
        useProjectStore.setState({ hasConflict: true });
      } else if (!navigator.onLine) {
        // Network error — mark for retry when online.
        pendingRetryRef.current = true;
        addToast("Geen verbinding — wordt opgeslagen zodra je weer online bent", "info");
      } else {
        addToast("Auto-save mislukt", "error");
      }
    }
  };

  // Retry on reconnect.
  useEffect(() => {
    const handleOnline = () => {
      if (pendingRetryRef.current) {
        addToast("Verbinding hersteld — opslaan...", "info", 2000);
        saveRef.current?.();
      }
    };
    window.addEventListener("online", handleOnline);
    return () => window.removeEventListener("online", handleOnline);
  }, [addToast]);

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
