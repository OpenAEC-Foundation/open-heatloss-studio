/**
 * Gedeelde ventilatiebalans-state + handlers — één bron van waarheid voor het
 * Modeller-zijpaneel én de Ventilatiebalans-tab. Alles leest/schrijft direct
 * op `useProjectStore.ventilation`, dus een wijziging op de tab is meteen
 * zichtbaar in het zijpaneel en vice versa.
 */

import { useCallback, useMemo } from "react";

import { useProjectStore } from "../store/projectStore";
import {
  defaultBblFunction,
  deriveVentilationDemand,
} from "../lib/ventilationBalance";
import type { BblFunctionKey } from "../types/ventilation";

export function useVentilationBalance() {
  const project = useProjectStore((s) => s.project);
  const ventilation = useProjectStore((s) => s.ventilation);
  const updateVentilationRoom = useProjectStore(
    (s) => s.updateVentilationRoom,
  );
  const setVentilationSystem = useProjectStore((s) => s.setVentilationSystem);

  // Per-room BBL-eis (dm³/s), afgeleid uit project.rooms + bestaande sidecar.
  const ventilationRooms = useMemo(
    () => deriveVentilationDemand(project, ventilation.rooms),
    [project, ventilation.rooms],
  );

  /** Schrijf de gebruiksfunctie-override voor een ruimte. */
  const changeFunction = useCallback(
    (roomId: string, fn: BblFunctionKey) => {
      updateVentilationRoom(roomId, { ventilationFunction: fn });
    },
    [updateVentilationRoom],
  );

  /**
   * Schrijf de bezetting (personen-toeslag). Schrijft óók de huidige
   * effectieve functie mee zodat een verse sidecar-entry niet terugvalt op de
   * generieke base-default ("verblijfsruimte") en daarmee bv. een
   * keuken-classificatie zou overschrijven.
   */
  const changeOccupancy = useCallback(
    (roomId: string, occupancy: number | undefined) => {
      const room = project.rooms.find((r) => r.id === roomId);
      const fn =
        ventilationRooms[roomId]?.ventilationFunction ??
        defaultBblFunction(String(room?.function ?? "custom"));
      updateVentilationRoom(roomId, { ventilationFunction: fn, occupancy });
    },
    [updateVentilationRoom, ventilationRooms, project.rooms],
  );

  return {
    project,
    ventilation,
    ventilationRooms,
    changeFunction,
    changeOccupancy,
    setSystem: setVentilationSystem,
  };
}
