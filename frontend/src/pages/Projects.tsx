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
import type { ProjectSummary, Project } from "../types";

export function Projects() {
  const navigate = useNavigate();
  const { project, setProject, setActiveProjectId, setResult } = useProjectStore();

  const [projects, setProjects] = useState<ProjectSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

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
        setProject(response.project_data as Project);
        setActiveProjectId(id);
        if (response.result_data) {
          setResult(response.result_data as Parameters<typeof setResult>[0]);
        }
        navigate("/project");
      } catch (err) {
        setError(err instanceof Error ? err.message : "Kon project niet openen");
      }
    },
    [navigate, setProject, setActiveProjectId, setResult],
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
        actions={
          <Button onClick={handleSaveNew}>Huidig project opslaan</Button>
        }
      />

      <div className="space-y-4 p-6">
        {error && (
          <div className="rounded-md bg-red-50 px-4 py-3 text-sm text-red-700">
            {error}
          </div>
        )}

        {loading ? (
          <p className="text-sm text-stone-400">Laden...</p>
        ) : projects.length === 0 ? (
          <Card>
            <div className="py-4 text-center text-sm text-stone-500">
              Nog geen opgeslagen projecten. Sla je huidige project op met de knop
              hierboven.
            </div>
          </Card>
        ) : (
          <Card>
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-stone-200 text-left text-xs font-medium uppercase text-stone-400">
                  <th className="px-3 py-2">Naam</th>
                  <th className="px-3 py-2">Laatst gewijzigd</th>
                  <th className="px-3 py-2">Resultaat</th>
                  <th className="px-3 py-2 text-right">Acties</th>
                </tr>
              </thead>
              <tbody>
                {projects.map((p) => (
                  <tr
                    key={p.id}
                    className="border-b border-stone-100 transition-colors last:border-0 hover:bg-stone-50"
                  >
                    <td className="px-3 py-2.5 font-medium">{p.name}</td>
                    <td className="px-3 py-2.5 text-stone-500">
                      {formatDate(p.updated_at)}
                    </td>
                    <td className="px-3 py-2.5">
                      {p.has_result ? (
                        <span className="inline-block rounded bg-green-100 px-1.5 py-0.5 text-xs text-green-700">
                          Berekend
                        </span>
                      ) : (
                        <span className="inline-block rounded bg-stone-100 px-1.5 py-0.5 text-xs text-stone-500">
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
