import { useCallback } from "react";
import { useNavigate } from "react-router-dom";

import { PageHeader } from "../components/layout/PageHeader";
import { RoomTable } from "../components/rooms/RoomTable";
import { Button } from "../components/ui/Button";
import { useBackend } from "../hooks/useBackend";
import { useProjectStore } from "../store/projectStore";

export function RoomEditor() {
  const navigate = useNavigate();
  const backend = useBackend();
  const { project, isCalculating, setCalculating, setResult, setError } =
    useProjectStore();

  const handleCalculate = useCallback(async () => {
    setCalculating(true);
    try {
      const result = await backend.calculate(project);
      setResult(result);
      navigate("/results");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Berekening mislukt");
    }
  }, [backend, project, setCalculating, setResult, setError, navigate]);

  return (
    <div>
      <PageHeader
        title="Vertrekken"
        subtitle={`${project.rooms.length} vertrek${project.rooms.length !== 1 ? "ken" : ""}`}
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
