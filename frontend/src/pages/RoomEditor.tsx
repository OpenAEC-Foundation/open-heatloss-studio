import { useCallback } from "react";
import { useNavigate } from "react-router-dom";

import { PageHeader } from "../components/layout/PageHeader";
import { RoomTable } from "../components/rooms/RoomTable";
import { Button } from "../components/ui/Button";
import { useProjectStore } from "../store/projectStore";
import { useRunCalculation } from "../hooks/useRunCalculation";

export function RoomEditor() {
  const navigate = useNavigate();
  const { project, isCalculating } = useProjectStore();
  const runCalculation = useRunCalculation();

  const handleCalculate = useCallback(async () => {
    const ok = await runCalculation();
    if (ok) {
      navigate("/results");
    }
  }, [runCalculation, navigate]);

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
