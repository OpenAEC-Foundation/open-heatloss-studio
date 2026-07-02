/**
 * Zones-card — beheer van gebouw-zones (`Building.zones`) op de
 * Project-pagina, naast de overige gebouw-brede instellingen
 * (zelfde Card-patroon als `AlgemeenTab`).
 *
 * Zones zijn een datalaag-concept (zie `types/project.ts::Zone`): ze komen
 * uit de Revit thermal-import (`rooms[].zone`) of worden hier handmatig
 * aangemaakt. Toekenning per vertrek gebeurt in de Vertrekken-tabel
 * (`components/rooms/RoomTable.tsx`); de ventilatiebalans groepeert erop.
 *
 * Alle mutaties lopen via de bestaande undo-aware store-actions
 * (`addZone` / `renameZone` / `removeZone`) — geen eigen data-logica.
 */
import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { Card } from "../ui/Card";
import { useProjectStore } from "../../store/projectStore";
import type { Zone } from "../../types";
import { normalizeZoneName, zoneNameExists } from "./zoneNames";

export function ZonesCard() {
  const { t } = useTranslation();
  const zones = useProjectStore((s) => s.project.building.zones);
  const rooms = useProjectStore((s) => s.project.rooms);
  const addZone = useProjectStore((s) => s.addZone);
  const renameZone = useProjectStore((s) => s.renameZone);
  const removeZone = useProjectStore((s) => s.removeZone);

  const [newName, setNewName] = useState("");
  const [addError, setAddError] = useState<string | null>(null);

  const zoneList = zones ?? [];

  /** Aantal vertrekken dat aan een zone hangt (voor badge + delete-confirm). */
  const roomCountFor = useCallback(
    (zoneId: string) => rooms.filter((r) => r.zoneId === zoneId).length,
    [rooms],
  );

  const duplicateMsg = t(
    "projectSetup.fields.zoneDuplicate",
    "Er bestaat al een zone met deze naam.",
  );

  const handleAdd = useCallback(() => {
    const name = normalizeZoneName(newName);
    if (!name) return;
    // Case-insensitieve dedup: geen tweede zone met dezelfde naam (zie
    // `zoneNames.ts` — voorkomt niet-deterministische Revit-import-mapping).
    if (zoneNameExists(zoneList, name)) {
      setAddError(duplicateMsg);
      return;
    }
    addZone(name);
    setNewName("");
    setAddError(null);
  }, [newName, addZone, zoneList, duplicateMsg]);

  const handleRemove = useCallback(
    (zone: Zone) => {
      const count = roomCountFor(zone.id);
      if (count > 0) {
        const ok = window.confirm(
          `Zone "${zone.name}" verwijderen? ${count} vertrek${count === 1 ? "" : "ken"} ` +
            `${count === 1 ? "is" : "zijn"} aan deze zone gekoppeld en ` +
            `${count === 1 ? "wordt" : "worden"} teruggezet naar "geen zone".`,
        );
        if (!ok) return;
      }
      removeZone(zone.id);
    },
    [roomCountFor, removeZone],
  );

  return (
    <Card title={t("projectSetup.sections.zones", "Zones")}>
      <p className="text-[10px] text-on-surface-muted">
        {t(
          "projectSetup.fields.zonesHint",
          "Deel vertrekken in zones in (handmatig of via Revit-import). Toekenning per vertrek gebeurt in de Vertrekken-tabel; de ventilatiebalans groepeert per zone.",
        )}
      </p>

      {zoneList.length > 0 && (
        <ul className="mt-3 divide-y divide-[var(--oaec-border-subtle)] rounded border border-[var(--oaec-border)]">
          {zoneList.map((zone) => (
            <ZoneRow
              key={zone.id}
              zone={zone}
              roomCount={roomCountFor(zone.id)}
              onRename={(name) => renameZone(zone.id, name)}
              onRemove={() => handleRemove(zone)}
              isDuplicateName={(name) => zoneNameExists(zoneList, name, zone.id)}
            />
          ))}
        </ul>
      )}

      <div className="mt-3 flex items-center gap-2">
        <input
          type="text"
          value={newName}
          onChange={(e) => {
            setNewName(e.target.value);
            if (addError) setAddError(null);
          }}
          onKeyDown={(e) => {
            if (e.key === "Enter") handleAdd();
          }}
          placeholder={t(
            "projectSetup.fields.newZonePlaceholder",
            "Nieuwe zone (bv. Begane grond)",
          )}
          className="w-64 rounded border border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] px-2 py-1.5 text-sm text-on-surface placeholder:text-on-surface-muted focus:border-primary focus:outline-none"
        />
        <button
          type="button"
          onClick={handleAdd}
          disabled={
            normalizeZoneName(newName) === "" ||
            zoneNameExists(zoneList, newName)
          }
          className="rounded border border-[var(--oaec-border)] px-3 py-1.5 text-xs font-medium text-on-surface-secondary hover:bg-[var(--oaec-hover)] disabled:cursor-not-allowed disabled:opacity-40"
        >
          {t("projectSetup.fields.addZone", "+ Zone toevoegen")}
        </button>
      </div>

      {addError && (
        <p className="mt-1.5 text-[11px] text-red-400">{addError}</p>
      )}
    </Card>
  );
}

/**
 * Eén zone-regel: inline hernoemen (commit op blur/Enter, Escape annuleert)
 * + vertrek-teller + verwijder-knop. Draft-state lokaal zodat elke toetsaanslag
 * niet meteen een undo-stap in de store wordt.
 */
function ZoneRow({
  zone,
  roomCount,
  onRename,
  onRemove,
  isDuplicateName,
}: {
  zone: Zone;
  roomCount: number;
  onRename: (name: string) => void;
  onRemove: () => void;
  /** True als `name` (trim, case-insensitief) al een ándere zone is. */
  isDuplicateName: (name: string) => boolean;
}) {
  const [draft, setDraft] = useState(zone.name);

  // Externe wijziging (undo/redo, import) → draft mee laten lopen.
  useEffect(() => {
    setDraft(zone.name);
  }, [zone.name]);

  const commit = () => {
    const name = draft.trim();
    // Leeg, ongewijzigd, óf duplicaat van een andere zone → revert (geen
    // rename). Zelfde dedup-regel als het toevoeg-pad.
    if (name === "" || name === zone.name || isDuplicateName(name)) {
      setDraft(zone.name);
      return;
    }
    onRename(name);
  };

  return (
    <li className="flex items-center gap-2 px-2 py-1.5">
      <input
        type="text"
        value={draft}
        onChange={(e) => setDraft(e.target.value)}
        onBlur={commit}
        onKeyDown={(e) => {
          if (e.key === "Enter") (e.currentTarget as HTMLInputElement).blur();
          if (e.key === "Escape") setDraft(zone.name);
        }}
        className="flex-1 rounded border border-transparent bg-transparent px-1.5 py-0.5 text-sm text-on-surface hover:border-[var(--oaec-border)] focus:border-primary focus:bg-[var(--oaec-bg-input)] focus:outline-none"
        title="Zone hernoemen"
      />
      <span className="shrink-0 rounded-full bg-primary/10 px-2 py-0.5 text-[10px] font-semibold text-scaffold-gray">
        {roomCount} vertrek{roomCount === 1 ? "" : "ken"}
      </span>
      <button
        type="button"
        onClick={onRemove}
        className="shrink-0 rounded p-1 text-on-surface-muted hover:bg-red-600/15 hover:text-red-400"
        title="Zone verwijderen"
      >
        <svg className="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor">
          <path
            fillRule="evenodd"
            d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
            clipRule="evenodd"
          />
        </svg>
      </button>
    </li>
  );
}
