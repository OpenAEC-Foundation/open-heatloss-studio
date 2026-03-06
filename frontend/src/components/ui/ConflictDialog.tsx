import { useCallback } from "react";

import { Button } from "./Button";
import { useProjectStore } from "../../store/projectStore";
import { fetchProject } from "../../lib/backend";
import { useToastStore } from "../../store/toastStore";
import type { Project, ProjectResult } from "../../types";

export function ConflictDialog() {
  const hasConflict = useProjectStore((s) => s.hasConflict);
  const activeProjectId = useProjectStore((s) => s.activeProjectId);
  const addToast = useToastStore((s) => s.addToast);

  const handleReload = useCallback(async () => {
    if (!activeProjectId) return;
    try {
      const response = await fetchProject(activeProjectId);
      useProjectStore.getState().loadServerProject(
        activeProjectId,
        response.project_data as Project,
        response.result_data as ProjectResult | null,
        response.updated_at,
      );
      addToast("Laatste versie geladen", "info", 2000);
    } catch {
      addToast("Kon project niet herladen", "error");
    }
  }, [activeProjectId, addToast]);

  const handleDismiss = useCallback(() => {
    useProjectStore.setState({ hasConflict: false });
  }, []);

  if (!hasConflict) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
      <div className="mx-4 w-full max-w-md rounded-lg bg-white p-6 shadow-lg">
        <h2 className="font-heading text-lg font-bold text-stone-900">
          Conflict gedetecteerd
        </h2>
        <p className="mt-2 text-sm text-stone-600">
          Dit project is ondertussen elders gewijzigd. Je kunt de laatste versie
          van de server laden (lokale wijzigingen gaan verloren) of de melding
          negeren en handmatig opslaan.
        </p>
        <div className="mt-5 flex justify-end gap-2">
          <Button variant="ghost" size="sm" onClick={handleDismiss}>
            Negeren
          </Button>
          <Button variant="primary" size="sm" onClick={handleReload}>
            Herladen
          </Button>
        </div>
      </div>
    </div>
  );
}
