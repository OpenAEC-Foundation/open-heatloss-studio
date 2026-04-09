/**
 * Thermal Import Wizard — 5-step import wizard for Revit thermal export.
 *
 * Steps:
 * 1. File upload + parse + backend POST
 * 2. Room review (type editing)
 * 3. Construction review (layer inspection)
 * 4. Opening review (U-value editing)
 * 5. Summary + final import
 */
import { useCallback, useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import {
  Upload,
  Home,
  Layers,
  DoorOpen,
  ClipboardCheck,
  ChevronLeft,
  ChevronRight,
  Loader2,
} from "lucide-react";

import type {
  ThermalImportFile,
  ThermalImportResult,
  ThermalRoom,
  ThermalOpening,
} from "../../lib/thermalImport";
import {
  importThermal,
  parseThermalImportFile,
  toImportedBoundaries,
  applyEditsToProject,
} from "../../lib/thermalImport";
import { useProjectStore } from "../../store/projectStore";
import { useModellerStore } from "../modeller/modellerStore";

import { FileUploadStep } from "./FileUploadStep";
import { RoomImportStep } from "./RoomImportStep";
import { ConstructionImportStep } from "./ConstructionImportStep";
import { OpeningImportStep } from "./OpeningImportStep";
import { ImportSummary } from "./ImportSummary";

// ---------------------------------------------------------------------------
// Step definitions
// ---------------------------------------------------------------------------

const STEPS = [
  { label: "Bestand", icon: Upload },
  { label: "Ruimtes", icon: Home },
  { label: "Constructies", icon: Layers },
  { label: "Openingen", icon: DoorOpen },
  { label: "Samenvatting", icon: ClipboardCheck },
] as const;

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function ThermalImportWizard() {
  const navigate = useNavigate();
  const setProject = useProjectStore((s) => s.setProject);
  const setImportedBoundaries = useModellerStore((s) => s.setImportedBoundaries);

  // Wizard state
  const [currentStep, setCurrentStep] = useState(0);
  const [importFile, setImportFile] = useState<ThermalImportFile | null>(null);
  const [importResult, setImportResult] = useState<ThermalImportResult | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Editable copies of rooms and openings
  const [editedRooms, setEditedRooms] = useState<ThermalRoom[]>([]);
  const [editedOpenings, setEditedOpenings] = useState<ThermalOpening[]>([]);
  // U-values calculated via LayerEditor in step 3, keyed by CatalogEntry.id.
  // applyEditsToProject propagates these to every ConstructionElement whose
  // catalog_ref points at that catalog entry, so a single edit fans out to
  // all rooms that share that construction.
  const [catalogUValues, setCatalogUValues] = useState<Map<string, number>>(new Map());

  // Step 1 complete handler: parse file, call backend
  const handleFileAccepted = useCallback(
    async (file: ThermalImportFile) => {
      setImportFile(file);
      setEditedRooms(file.rooms.map((r) => ({ ...r })));
      setEditedOpenings((file.openings ?? []).map((o) => ({ ...o })));
      setError(null);
      setIsLoading(true);

      try {
        const result = await importThermal(file);
        setImportResult(result);
        setCurrentStep(1);
      } catch (err) {
        setError(err instanceof Error ? err.message : "Import mislukt");
      } finally {
        setIsLoading(false);
      }
    },
    [],
  );

  // Auto-load pre-supplied thermal JSON from sessionStorage (set by regular import auto-detect)
  useEffect(() => {
    const preloaded = sessionStorage.getItem("thermalImportJson");
    if (!preloaded) return;

    // Clean up immediately so it doesn't fire again on re-mount
    sessionStorage.removeItem("thermalImportJson");

    try {
      const parsed = parseThermalImportFile(preloaded);
      // Trigger the same flow as a manual file upload
      handleFileAccepted(parsed);
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Automatische thermal import mislukt",
      );
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Final import action — applies user edits before loading into stores
  const handleFinalImport = useCallback(() => {
    if (!importResult || !importFile) return;

    // 1. Merge user edits (room types, U-values, LayerEditor results) into the backend-mapped project
    const mergedProject = applyEditsToProject(
      importResult.project,
      editedRooms,
      editedOpenings,
      catalogUValues,
    );
    setProject(mergedProject);

    // 2. Load imported boundaries into modellerStore for 3D viewer
    //    Use editedRooms so boundary conditions reflect user's type changes
    const boundaries = toImportedBoundaries(importFile.constructions, editedRooms);
    setImportedBoundaries(boundaries);

    // 3. Navigate to modeller
    navigate("/modeller");
  }, [importResult, importFile, editedRooms, editedOpenings, catalogUValues, setProject, setImportedBoundaries, navigate]);

  // LayerEditor U-value callback — keyed by CatalogEntry.id
  const handleConstructionUValue = useCallback((catalogId: string, uValue: number) => {
    setCatalogUValues((prev) => {
      const next = new Map(prev);
      next.set(catalogId, uValue);
      return next;
    });
  }, []);

  // Navigation
  const canGoNext = currentStep < STEPS.length - 1;
  const canGoPrev = currentStep > 0;

  const handleNext = useCallback(() => {
    if (canGoNext) setCurrentStep((s) => s + 1);
  }, [canGoNext]);

  const handlePrev = useCallback(() => {
    if (canGoPrev) setCurrentStep((s) => s - 1);
  }, [canGoPrev]);

  // Only allow navigating past step 0 when we have data
  const hasData = importFile !== null && importResult !== null;

  return (
    <div className="mx-auto max-w-5xl px-4 py-6">
      {/* Step indicator */}
      <nav className="mb-8">
        <ol className="flex items-center justify-between">
          {STEPS.map((step, index) => {
            const Icon = step.icon;
            const isActive = index === currentStep;
            const isCompleted = index < currentStep;
            const isAccessible = index === 0 || hasData;

            return (
              <li key={step.label} className="flex flex-1 items-center">
                <button
                  onClick={() => isAccessible && setCurrentStep(index)}
                  disabled={!isAccessible}
                  className={`flex flex-col items-center gap-1.5 transition-colors ${
                    isActive
                      ? "text-[#45B6A8]"
                      : isCompleted
                        ? "text-[#45B6A8]/70"
                        : "text-gray-500"
                  } ${isAccessible ? "cursor-pointer" : "cursor-not-allowed opacity-50"}`}
                >
                  <div
                    className={`flex h-10 w-10 items-center justify-center rounded-full border-2 transition-colors ${
                      isActive
                        ? "border-[#45B6A8] bg-[#45B6A8]/10"
                        : isCompleted
                          ? "border-[#45B6A8]/60 bg-[#45B6A8]/5"
                          : "border-gray-600 bg-gray-800"
                    }`}
                  >
                    <Icon className="h-5 w-5" />
                  </div>
                  <span className="text-xs font-medium">{step.label}</span>
                </button>
                {index < STEPS.length - 1 && (
                  <div
                    className={`mx-2 h-0.5 flex-1 ${
                      index < currentStep ? "bg-[#45B6A8]/40" : "bg-gray-700"
                    }`}
                  />
                )}
              </li>
            );
          })}
        </ol>
      </nav>

      {/* Loading overlay */}
      {isLoading && (
        <div className="mb-6 flex items-center gap-3 rounded-lg border border-[#45B6A8]/30 bg-[#45B6A8]/5 px-4 py-3 text-sm text-gray-300">
          <Loader2 className="h-5 w-5 animate-spin text-[#45B6A8]" />
          <span>Bezig met verwerken van het bestand...</span>
        </div>
      )}

      {/* Error message */}
      {error && (
        <div className="mb-6 rounded-lg border border-red-500/30 bg-red-500/5 px-4 py-3 text-sm text-red-300">
          {error}
        </div>
      )}

      {/* Step content */}
      <div className="min-h-[400px]">
        {currentStep === 0 && (
          <FileUploadStep
            onFileAccepted={handleFileAccepted}
            isLoading={isLoading}
            importFile={importFile}
          />
        )}
        {currentStep === 1 && importFile && (
          <RoomImportStep
            rooms={editedRooms}
            onRoomsChange={setEditedRooms}
          />
        )}
        {currentStep === 2 && importFile && importResult && (
          <ConstructionImportStep
            rooms={editedRooms}
            project={importResult.project}
            catalog={importResult.construction_catalog}
            onCatalogUValue={handleConstructionUValue}
          />
        )}
        {currentStep === 3 && importFile && (
          <OpeningImportStep
            openings={editedOpenings}
            constructions={importFile.constructions}
            rooms={editedRooms}
            onOpeningsChange={setEditedOpenings}
          />
        )}
        {currentStep === 4 && importFile && importResult && (
          <ImportSummary
            importFile={importFile}
            importResult={importResult}
            editedRooms={editedRooms}
            editedOpenings={editedOpenings}
            onImport={handleFinalImport}
          />
        )}
      </div>

      {/* Footer navigation */}
      {hasData && currentStep > 0 && (
        <div className="mt-8 flex items-center justify-between border-t border-gray-700 pt-4">
          <button
            onClick={handlePrev}
            disabled={!canGoPrev}
            className="flex items-center gap-2 rounded-lg border border-gray-600 px-4 py-2 text-sm text-gray-300 transition-colors hover:bg-gray-800 disabled:opacity-40"
          >
            <ChevronLeft className="h-4 w-4" />
            Vorige
          </button>

          {currentStep < STEPS.length - 1 && (
            <button
              onClick={handleNext}
              disabled={!canGoNext}
              className="flex items-center gap-2 rounded-lg bg-[#45B6A8] px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-[#3da396] disabled:opacity-40"
            >
              Volgende
              <ChevronRight className="h-4 w-4" />
            </button>
          )}
        </div>
      )}
    </div>
  );
}
