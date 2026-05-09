/**
 * IFC-page — toont de gegenereerde IFC4X3 (STEP) en IFCX (.ifcenergy) van
 * het huidige project side-by-side. Pattern direct gespiegeld op Open Calc
 * Studio's `IfcPreview` component (zie components/report/IfcPreview.tsx
 * in OCS).
 */
import { IfcPreview } from "../components/ifc/IfcPreview";

export function Ifc() {
  return (
    <div className="flex h-full w-full flex-col">
      <div className="border-b border-border px-6 py-3">
        <h1 className="text-lg font-semibold text-on-surface">IFC</h1>
        <p className="text-xs text-scaffold-gray">
          Gegenereerde IFC-representaties van het project: STEP (IFC4X3)
          links, IFCX (.ifcenergy) rechts. Ververst automatisch bij elke
          wijziging in Vertrekken/Constructies/Modeller.
        </p>
      </div>
      <div className="flex-1 overflow-hidden">
        <IfcPreview />
      </div>
    </div>
  );
}
