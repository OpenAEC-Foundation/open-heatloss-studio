/**
 * Gedeelde ventilatiebalans-state + handlers — één bron van waarheid voor het
 * Modeller-zijpaneel én de Ventilatiebalans-tab. Alles leest/schrijft direct
 * op `useProjectStore.ventilation`, dus een wijziging op de tab is meteen
 * zichtbaar in het zijpaneel en vice versa.
 */

import { useCallback, useMemo } from "react";

import { useProjectStore } from "../store/projectStore";
import {
  computeOverflowDistribution,
  defaultBblFunction,
  deriveVentilationDemand,
} from "../lib/ventilationBalance";
import { checkUnitCapacity, findCatalogUnit } from "../lib/ventilationUnits";
import type { BblFunctionKey, VentilationUnit } from "../types/ventilation";

export function useVentilationBalance() {
  const project = useProjectStore((s) => s.project);
  const ventilation = useProjectStore((s) => s.ventilation);
  const updateVentilationRoom = useProjectStore(
    (s) => s.updateVentilationRoom,
  );
  const setVentilationSystem = useProjectStore((s) => s.setVentilationSystem);
  const addVentilationUnit = useProjectStore((s) => s.addVentilationUnit);
  const updateVentilationUnit = useProjectStore(
    (s) => s.updateVentilationUnit,
  );
  const removeVentilationUnit = useProjectStore(
    (s) => s.removeVentilationUnit,
  );
  const setVentilationUnitAssignment = useProjectStore(
    (s) => s.setVentilationUnitAssignment,
  );

  // Per-room BBL-eis (dm³/s), afgeleid uit project.rooms + bestaande sidecar.
  const ventilationRooms = useMemo(
    () => deriveVentilationDemand(project, ventilation.rooms),
    [project, ventilation.rooms],
  );

  // Gebouwbrede overdruk-verdeling: toevoer-overschot naar afvoerruimtes,
  // naar rato van oppervlak (plugin `_bereken_overdruk_verdeling`). Voedt de
  // overstroom-relaties/spleet-berekening in de Modeller.
  const overflowDistribution = useMemo(() => {
    const areasM2: Record<string, number> = {};
    for (const room of project.rooms) areasM2[room.id] = room.floor_area;
    return computeOverflowDistribution(ventilationRooms, areasM2);
  }, [project.rooms, ventilationRooms]);

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

  // Capaciteitstoets: toegewezen unit-capaciteit vs. de gecombineerde eis
  // (systeem-bewust, zie `combinedRequirementDm3s` in lib/ventilationUnits.ts).
  const unitCapacity = useMemo(() => {
    let supply = 0;
    let exhaust = 0;
    for (const vr of Object.values(ventilationRooms)) {
      supply += vr.requiredSupplyDm3s;
      exhaust += vr.requiredExhaustDm3s;
    }
    return checkUnitCapacity(
      ventilation.units,
      ventilation.unitAssignments,
      supply,
      exhaust,
      ventilation.system,
    );
  }, [
    ventilationRooms,
    ventilation.units,
    ventilation.unitAssignments,
    ventilation.system,
  ]);

  /**
   * Wijs een catalogus-unit toe: kopieer het catalogus-snapshot eenmalig naar
   * de project-unitbibliotheek (source "catalog") en zet het aantal.
   */
  const assignCatalogUnit = useCallback(
    (catalogId: string, aantal: number) => {
      const unit = findCatalogUnit(catalogId);
      if (!unit) return;
      addVentilationUnit(unit); // no-op wanneer het snapshot al bestaat
      setVentilationUnitAssignment(unit.id, aantal);
    },
    [addVentilationUnit, setVentilationUnitAssignment],
  );

  /** Voeg een custom unit toe en wijs hem direct toe (aantal 1). */
  const addCustomUnit = useCallback(
    (unit: Omit<VentilationUnit, "id" | "source">) => {
      const id = addVentilationUnit({ ...unit, source: "custom" });
      setVentilationUnitAssignment(id, 1);
      return id;
    },
    [addVentilationUnit, setVentilationUnitAssignment],
  );

  return {
    project,
    ventilation,
    ventilationRooms,
    overflowDistribution,
    changeFunction,
    changeOccupancy,
    setSystem: setVentilationSystem,
    unitCapacity,
    assignCatalogUnit,
    addCustomUnit,
    updateUnit: updateVentilationUnit,
    removeUnit: removeVentilationUnit,
    setUnitAssignment: setVentilationUnitAssignment,
  };
}
