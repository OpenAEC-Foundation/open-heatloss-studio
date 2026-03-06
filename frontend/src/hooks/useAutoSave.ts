/**
 * Auto-save hook: debounced save to server when project is dirty.
 *
 * Only active when authenticated and a server-side project is loaded.
 */
import { useEffect, useRef } from "react";

import { useAuth } from "./useAuth";
import { useProjectStore } from "../store/projectStore";
import { updateProject } from "../lib/backend";

const AUTO_SAVE_DELAY_MS = 5_000;

export function useAutoSave(): void {
  const auth = useAuth();
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const isDirty = useProjectStore((s) => s.isDirty);
  const activeProjectId = useProjectStore((s) => s.activeProjectId);
  const project = useProjectStore((s) => s.project);

  useEffect(() => {
    if (timerRef.current) {
      clearTimeout(timerRef.current);
      timerRef.current = null;
    }

    if (!auth.isLoggedIn || !activeProjectId || !isDirty) {
      return;
    }

    timerRef.current = setTimeout(async () => {
      try {
        await updateProject(activeProjectId, {
          name: project.info.name || undefined,
          project_data: project,
        });
        // Only clear dirty flag if the project hasn't changed during the save.
        useProjectStore.setState((state) => {
          if (state.activeProjectId === activeProjectId) {
            return { isDirty: false };
          }
          return {};
        });
      } catch {
        // Silent fail — user can still manually save.
      }
    }, AUTO_SAVE_DELAY_MS);

    return () => {
      if (timerRef.current) {
        clearTimeout(timerRef.current);
      }
    };
  }, [auth.isLoggedIn, activeProjectId, isDirty, project]);
}
