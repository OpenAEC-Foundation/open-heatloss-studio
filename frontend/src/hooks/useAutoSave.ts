/**
 * Auto-save hook: debounced save to server when project is dirty.
 *
 * Only active when authenticated and a server-side project is loaded.
 * Saves the FULL project envelope (geometry + sidecars) via the shared
 * `saveExistingServerProject` helper — same payload as the file export.
 * Save state is surfaced persistently through `useSaveStatusStore`
 * (StatusBar indicator); toasts only cover actionable failures.
 * Retries automatically when the browser comes back online after a network
 * error, and exposes a manual retry handler for the StatusBar.
 */
import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";

import { useAuth } from "./useAuth";
import { useProjectStore } from "../store/projectStore";
import { useSaveStatusStore } from "../store/saveStatusStore";
import { useToastStore } from "../store/toastStore";
import { saveExistingServerProject } from "../lib/serverProjects";
import { ConflictError, SessionExpiredError } from "../lib/backend";

const AUTO_SAVE_DELAY_MS = 5_000;

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

  // Save function extracted so it can be called from the timer, the online
  // handler and the StatusBar retry button.
  const saveRef = useRef<(() => Promise<void>) | null>(null);
  saveRef.current = async () => {
    const state = useProjectStore.getState();
    const id = state.activeProjectId;
    if (!id || !state.isDirty) return;

    try {
      // Stuurt de volledige envelope (modeller-geometrie, sharedExtra,
      // ISSO 53 + ventilatie-sidecars) — pariteit met de file-save. De
      // helper werkt isDirty/serverUpdatedAt en de statusindicator bij.
      await saveExistingServerProject(id);
      pendingRetryRef.current = false;
      // Geen succes-toast meer: de persistente StatusBar-indicator toont
      // "opgeslagen HH:MM" — een toast elke 5 s was ruis.
    } catch (err) {
      // Statusindicator (offline/error/conflict) is al gezet door de helper;
      // hier alleen de aanvullende, actiegerichte UX.
      if (err instanceof SessionExpiredError) {
        // Authentik session expired — saving to the server is impossible
        // until the user logs in again. A reload triggers the login flow.
        pendingRetryRef.current = false;
        addToast(t("saveStatus.sessionExpired"), "error", 8000, {
          label: t("saveStatus.loginAgain"),
          onClick: () => window.location.reload(),
        });
      } else if (err instanceof ConflictError) {
        // hasConflict is door de helper gezet → ConflictDialog opent.
        pendingRetryRef.current = false;
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

  // Maak de save handmatig triggerbaar vanuit de StatusBar (retry-knop).
  useEffect(() => {
    const store = useSaveStatusStore.getState();
    store.registerRetryHandler(() => {
      void saveRef.current?.();
    });
    return () => {
      useSaveStatusStore.getState().registerRetryHandler(null);
    };
  }, []);

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
