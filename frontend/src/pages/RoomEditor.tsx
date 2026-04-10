import { useCallback } from "react";
import { useNavigate } from "react-router-dom";

import { PageHeader } from "../components/layout/PageHeader";
import { RoomTable } from "../components/rooms/RoomTable";
import { Button } from "../components/ui/Button";
import { useBackend } from "../hooks/useBackend";
import { useProjectStore } from "../store/projectStore";
import { useModellerStore } from "../components/modeller/modellerStore";
import { prepareProjectForCalculation } from "../lib/frameOverride";

export function RoomEditor() {
  const navigate = useNavigate();
  const backend = useBackend();
  const { project, isCalculating, setCalculating, setResult, setError } =
    useProjectStore();
  const projectConstructions = useModellerStore((s) => s.projectConstructions);

  const handleCalculate = useCallback(async () => {
    setCalculating(true);
    try {
      const payload = prepareProjectForCalculation(project, projectConstructions);
      const result = await backend.calculate(payload);
      setResult(result);
      navigate("/results");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Berekening mislukt");
    }
  }, [
    backend,
    project,
    projectConstructions,
    setCalculating,
    setResult,
    setError,
    navigate,
  ]);

  return (
    <div>
      <PageHeader
        title="Vertrekken"
        subtitle={`${project.rooms.length} vertrek${project.rooms.length !== 1 ? "ken" : ""}`}
        breadcrumbs={[{ label: "Vertrekken" }]}
        actions={
          <Button
            onClick={handleCalculate}
            disabled={isCalculating || project.rooms.length === 0}
          >
            {isCalculating ? "Berekenen..." : "Berekenen"}
          </Button>
        }
      />
      <div className="p-4">
        <RoomTable />
      </div>
    </div>
  );
}
