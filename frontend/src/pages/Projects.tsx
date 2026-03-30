import { useCallback, useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";

import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { PageHeader } from "../components/layout/PageHeader";
import { useProjectStore } from "../store/projectStore";
import {
  fetchProjects,
  fetchProject,
  createProject,
  deleteProject,
} from "../lib/backend";
import { validateProject, validateProjectResult } from "../lib/importExport";
import type { ProjectSummary } from "../types";

export function Projects() {
  const navigate = useNavigate();
  const { project, loadServerProject, setActiveProjectId } = useProjectStore();

  const [projects, setProjects] = useState<ProjectSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [search, setSearch] = useState("");

  const loadProjects = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const list = await fetchProjects();
      setProjects(list);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Kon projecten niet laden");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadProjects();
  }, [loadProjects]);

  const handleOpen = useCallback(
    async (id: string) => {
      try {
        const response = await fetchProject(id);
        const projectData = validateProject(response.project_data);
        loadServerProject(
          id,
          projectData,
          validateProjectResult(response.result_data),
          response.updated_at,
        );
        navigate("/project");
      } catch (err) {
        setError(err instanceof Error ? err.message : "Kon project niet openen");
      }
    },
    [navigate, loadServerProject],
  );

  const handleSaveNew = useCallback(async () => {
    try {
      const name = project.info.name || "Naamloos project";
      const result = await createProject(name, project);
      setActiveProjectId(result.id);
      await loadProjects();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Kon project niet opslaan");
    }
  }, [project, setActiveProjectId, loadProjects]);

  const handleDuplicate = useCallback(
    async (id: string) => {
      try {
        const response = await fetchProject(id);
        const sourceData = validateProject(response.project_data);
        const name = `Kopie van ${response.name}`;
        await createProject(name, sourceData);
        await loadProjects();
      } catch (err) {
        setError(err instanceof Error ? err.message : "Kon project niet dupliceren");
      }
    },
    [loadProjects],
  );

  const handleDelete = useCallback(
    async (id: string, projectName: string) => {
      if (!window.confirm(`Weet je zeker dat je "${projectName}" wilt verwijderen?`)) {
        return;
      }
      try {
        await deleteProject(id);
        await loadProjects();
      } catch (err) {
        setError(err instanceof Error ? err.message : "Kon project niet verwijderen");
      }
    },
    [loadProjects],
  );

  const filtered = projects.filter((p) =>
    p.name.toLowerCase().includes(search.toLowerCase()),
  );

  const formatDate = (dateStr: string) => {
    try {
      return new Date(dateStr + "Z").toLocaleDateString("nl-NL", {
        day: "numeric",
        month: "short",
        year: "numeric",
        hour: "2-digit",
        minute: "2-digit",
      });
    } catch {
      return dateStr;
    }
  };

  return (
    <div>
      <PageHeader
        title="Projecten"
        subtitle="Opgeslagen projecten op de server"
        breadcrumbs={[{ label: "Projecten" }]}
        actions={
          <Button onClick={handleSaveNew}>Huidig project opslaan</Button>
        }
      />

      <div className="space-y-4 p-6">
        {error && (
          <div className="rounded-md bg-red-600/15 border border-red-600/30 px-4 py-3 text-sm text-red-400">
            {error}
          </div>
        )}

        {projects.length > 0 && (
          <input
            type="text"
            placeholder="Zoek op projectnaam..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full max-w-sm rounded-md border border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] text-on-surface px-3 py-1.5 text-sm placeholder:text-on-surface-muted focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
          />
        )}

        {loading ? (
          <p className="text-sm text-on-surface-muted">Laden...</p>
        ) : filtered.length === 0 && projects.length === 0 ? (
          <Card>
            <div className="py-4 text-center text-sm text-on-surface-muted">
              Nog geen opgeslagen projecten. Sla je huidige project op met de knop
              hierboven.
            </div>
          </Card>
        ) : filtered.length === 0 ? (
          <Card>
            <div className="py-4 text-center text-sm text-on-surface-muted">
              Geen projecten gevonden voor &ldquo;{search}&rdquo;.
            </div>
          </Card>
        ) : (
          <Card>
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-[var(--oaec-border)] text-left text-xs font-medium uppercase text-on-surface-muted">
                  <th className="px-3 py-2">Naam</th>
                  <th className="px-3 py-2">Laatst gewijzigd</th>
                  <th className="px-3 py-2">Resultaat</th>
                  <th className="px-3 py-2 text-right">Acties</th>
                </tr>
              </thead>
              <tbody>
                {filtered.map((p) => (
                  <tr
                    key={p.id}
                    className="border-b border-[var(--oaec-border-subtle)] transition-colors last:border-0 hover:bg-[var(--oaec-hover)]"
                  >
                    <td className="px-3 py-2.5 font-medium">{p.name}</td>
                    <td className="px-3 py-2.5 text-on-surface-muted">
                      {formatDate(p.updated_at)}
                    </td>
                    <td className="px-3 py-2.5">
                      {p.has_result ? (
                        <span className="inline-block rounded bg-green-600/15 px-1.5 py-0.5 text-xs text-green-400">
                          Berekend
                        </span>
                      ) : (
                        <span className="inline-block rounded bg-[var(--oaec-hover)] px-1.5 py-0.5 text-xs text-on-surface-muted">
                          Concept
                        </span>
                      )}
                    </td>
                    <td className="px-3 py-2.5 text-right">
                      <div className="flex justify-end gap-2">
                        <Button
                          variant="secondary"
                          size="sm"
                          onClick={() => handleOpen(p.id)}
                        >
                          Openen
                        </Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => handleDuplicate(p.id)}
                        >
                          Dupliceren
                        </Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => handleDelete(p.id, p.name)}
                        >
                          Verwijderen
                        </Button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </Card>
        )}
      </div>
    </div>
  );
}
