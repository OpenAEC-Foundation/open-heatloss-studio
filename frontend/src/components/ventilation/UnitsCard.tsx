/**
 * WTW/MV-units-kaart (`/ventilation`-tab): catalogus-keuze + aantal,
 * custom-unit-formulier en de capaciteitstoets (capaciteit vs. gecombineerde
 * eis, zie `lib/ventilationUnits.ts`).
 *
 * Port van de "Units"-tab uit de pyRevit-plugin
 * (`VentilatieBalans.pushbutton/script.py`, `_setup_units_tab` r.766+),
 * vereenvoudigd naar gebouwniveau (het webmodel kent geen zone-concept; het
 * datamodel is wel zone-ready via `VentilationUnitAssignment.zoneId`).
 *
 * Systeem-afhankelijk: bij systeem A (volledig natuurlijk) rendert de parent
 * deze kaart niet. De catalogus-filter volgt het voorkeurs-type van het
 * systeem (D → WTW, B/C → MV) maar is door de gebruiker omschakelbaar.
 *
 * **Eenheden:** capaciteit in m³/h (fabrikant-conventie); de toets toont
 * dm³/s primair, conform de rest van de module.
 */

import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import {
  ventilationSystemOf,
  type VentilationState,
  type VentilationUnit,
  type VentilationUnitType,
} from "../../types/ventilation";
import {
  getCatalogUnits,
  preferredUnitType,
  resolveUnitAssignments,
  type UnitCapacityCheck,
} from "../../lib/ventilationUnits";
import { formatDecimals } from "../../lib/formatNumber";
import { Button } from "../ui/Button";
import { UnitCapacitySummary, flowLabel } from "./shared";

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface UnitsCardProps {
  /** Ventilatie-sidecar (units + toewijzingen + systeem). */
  ventilation: VentilationState;
  /** Uitkomst van de capaciteitstoets (uit `useVentilationBalance`). */
  unitCapacity: UnitCapacityCheck;
  /** Wijs een catalogus-unit toe (kopieert snapshot + zet aantal). */
  onAssignCatalogUnit: (catalogId: string, aantal: number) => void;
  /** Voeg een custom unit toe (en wijs direct toe). */
  onAddCustomUnit: (unit: Omit<VentilationUnit, "id" | "source">) => void;
  /** Werk een bestaande (custom) unit bij. */
  onUpdateUnit: (
    id: string,
    partial: Partial<Omit<VentilationUnit, "id">>,
  ) => void;
  /** Verwijder een unit + zijn toewijzing. */
  onRemoveUnit: (id: string) => void;
  /** Zet het toegewezen aantal (absoluut; ≤ 0 verwijdert de toewijzing). */
  onSetAssignment: (unitId: string, aantal: number) => void;
}

// ---------------------------------------------------------------------------
// Custom-unit formulierstate
// ---------------------------------------------------------------------------

interface UnitFormState {
  type: VentilationUnitType;
  fabrikant: string;
  model: string;
  capaciteitM3h: string;
  rendementPct: string;
  geluidDb: string;
}

const EMPTY_FORM: UnitFormState = {
  type: "wtw",
  fabrikant: "",
  model: "",
  capaciteitM3h: "",
  rendementPct: "",
  geluidDb: "",
};

function formToUnit(
  form: UnitFormState,
): Omit<VentilationUnit, "id" | "source"> | null {
  // Verdediging in de diepte (naast de input-attributen): ongeldige invoer
  // levert `null` — de submit-knop blijft dan disabled en er wordt géén unit
  // aangemaakt. Capaciteit wordt op hele m³/h afgerond bij opslag.
  const capaciteitM3h = Math.round(Number(form.capaciteitM3h));
  if (!form.model.trim() || !Number.isFinite(capaciteitM3h) || capaciteitM3h <= 0) {
    return null;
  }

  // Rendement (alleen WTW, optioneel): indien ingevuld begrensd tot (0, 100]
  // — negatief, 0 of > 100 wordt geweigerd i.p.v. stilzwijgend weggelaten.
  let rendement: number | undefined;
  if (form.type === "wtw" && form.rendementPct !== "") {
    const pct = Number(form.rendementPct);
    if (!Number.isFinite(pct) || pct <= 0 || pct > 100) return null;
    rendement = pct / 100;
  }

  // Geluid (optioneel): indien ingevuld een eindig getal > 0.
  let geluidDb: number | undefined;
  if (form.geluidDb !== "") {
    const db = Number(form.geluidDb);
    if (!Number.isFinite(db) || db <= 0) return null;
    geluidDb = db;
  }

  return {
    type: form.type,
    fabrikant: form.fabrikant.trim(),
    model: form.model.trim(),
    capaciteitM3h,
    ...(rendement !== undefined ? { rendement } : {}),
    ...(geluidDb !== undefined ? { geluidDb } : {}),
  };
}

function unitToForm(unit: VentilationUnit): UnitFormState {
  return {
    type: unit.type,
    fabrikant: unit.fabrikant,
    model: unit.model,
    capaciteitM3h: String(unit.capaciteitM3h),
    rendementPct:
      unit.rendement !== undefined ? String(Math.round(unit.rendement * 100)) : "",
    geluidDb: unit.geluidDb !== undefined ? String(unit.geluidDb) : "",
  };
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function UnitsCard({
  ventilation,
  unitCapacity,
  onAssignCatalogUnit,
  onAddCustomUnit,
  onUpdateUnit,
  onRemoveUnit,
  onSetAssignment,
}: UnitsCardProps) {
  const { t } = useTranslation();
  const sys = ventilationSystemOf(ventilation);

  // Catalogus-keuze: type-filter default op het systeem-voorkeurstype.
  const [typeFilter, setTypeFilter] = useState<VentilationUnitType>(
    () => preferredUnitType(ventilation.system) ?? "wtw",
  );
  const [catalogId, setCatalogId] = useState("");
  const [catalogAantal, setCatalogAantal] = useState(1);

  // Custom-unit formulier (leeg = toevoegen; editingId = bewerken).
  const [form, setForm] = useState<UnitFormState>(EMPTY_FORM);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [showForm, setShowForm] = useState(false);

  const catalogUnits = useMemo(
    () => getCatalogUnits().filter((u) => u.type === typeFilter),
    [typeFilter],
  );
  const assigned = useMemo(
    () =>
      resolveUnitAssignments(ventilation.units, ventilation.unitAssignments),
    [ventilation.units, ventilation.unitAssignments],
  );

  const typeLabel = (type: VentilationUnitType) =>
    type === "wtw" ? t("ventilation.units.typeWtw") : t("ventilation.units.typeMv");

  const handleSubmitForm = () => {
    const unit = formToUnit(form);
    if (!unit) return;
    if (editingId) {
      onUpdateUnit(editingId, unit);
    } else {
      onAddCustomUnit(unit);
    }
    setForm(EMPTY_FORM);
    setEditingId(null);
    setShowForm(false);
  };

  return (
    <div className="space-y-3">
      {/* Disclaimer */}
      <p className="text-xs text-on-surface-muted">
        {t("ventilation.units.intro", { system: sys.key })}{" "}
        <span className="font-medium text-amber-700">
          {t("ventilation.units.disclaimer")}
        </span>
      </p>

      {/* Catalogus-keuze + aantal */}
      <div className="flex flex-wrap items-end gap-2">
        <label className="flex flex-col gap-1 text-xs text-scaffold-gray">
          {t("ventilation.units.type")}
          <select
            value={typeFilter}
            onChange={(e) => {
              setTypeFilter(e.target.value as VentilationUnitType);
              setCatalogId("");
            }}
            className="rounded border border-primary/20 bg-surface px-1.5 py-1 text-xs text-on-surface"
          >
            <option value="wtw">{t("ventilation.units.typeWtw")}</option>
            <option value="mv">{t("ventilation.units.typeMv")}</option>
          </select>
        </label>
        <label className="flex min-w-[16rem] flex-1 flex-col gap-1 text-xs text-scaffold-gray">
          {t("ventilation.units.catalogUnit")}
          <select
            value={catalogId}
            onChange={(e) => setCatalogId(e.target.value)}
            className="rounded border border-primary/20 bg-surface px-1.5 py-1 text-xs text-on-surface"
          >
            <option value="">{t("ventilation.units.choose")}</option>
            {catalogUnits.map((u) => (
              <option key={u.id} value={u.id}>
                {u.fabrikant} {u.model} — {u.capaciteitM3h} m³/h
                {u.rendement !== undefined
                  ? ` · η ${Math.round(u.rendement * 100)}%`
                  : ""}
              </option>
            ))}
          </select>
        </label>
        <label className="flex flex-col gap-1 text-xs text-scaffold-gray">
          {t("ventilation.units.count")}
          <input
            type="number"
            min={1}
            step={1}
            value={catalogAantal}
            onChange={(e) =>
              setCatalogAantal(Math.max(1, Math.floor(Number(e.target.value) || 1)))
            }
            className="w-16 rounded border border-primary/20 bg-surface px-1.5 py-1 text-right text-xs tabular-nums text-on-surface"
          />
        </label>
        <Button
          variant="secondary"
          size="sm"
          disabled={catalogId === ""}
          onClick={() => {
            if (catalogId === "") return;
            onAssignCatalogUnit(catalogId, catalogAantal);
            setCatalogId("");
            setCatalogAantal(1);
          }}
        >
          {t("ventilation.units.assign")}
        </Button>
        <Button
          variant="secondary"
          size="sm"
          onClick={() => {
            setShowForm((v) => !v);
            setEditingId(null);
            setForm(EMPTY_FORM);
          }}
        >
          {showForm
            ? t("ventilation.units.customCancel")
            : t("ventilation.units.customAdd")}
        </Button>
      </div>

      {/* Custom-unit formulier */}
      {showForm && (
        <div className="rounded-md border border-primary/15 bg-surface p-3">
          <div className="mb-2 text-xs font-semibold text-on-surface">
            {editingId
              ? t("ventilation.units.customEditTitle")
              : t("ventilation.units.customTitle")}
          </div>
          <div className="flex flex-wrap items-end gap-2">
            <label className="flex flex-col gap-1 text-xs text-scaffold-gray">
              {t("ventilation.units.type")}
              <select
                value={form.type}
                onChange={(e) =>
                  setForm({ ...form, type: e.target.value as VentilationUnitType })
                }
                className="rounded border border-primary/20 bg-surface px-1.5 py-1 text-xs text-on-surface"
              >
                <option value="wtw">{t("ventilation.units.typeWtw")}</option>
                <option value="mv">{t("ventilation.units.typeMv")}</option>
              </select>
            </label>
            <label className="flex flex-col gap-1 text-xs text-scaffold-gray">
              {t("ventilation.units.manufacturer")}
              <input
                value={form.fabrikant}
                onChange={(e) => setForm({ ...form, fabrikant: e.target.value })}
                className="w-32 rounded border border-primary/20 bg-surface px-1.5 py-1 text-xs text-on-surface"
              />
            </label>
            <label className="flex flex-col gap-1 text-xs text-scaffold-gray">
              {t("ventilation.units.model")} *
              <input
                value={form.model}
                onChange={(e) => setForm({ ...form, model: e.target.value })}
                className="w-36 rounded border border-primary/20 bg-surface px-1.5 py-1 text-xs text-on-surface"
              />
            </label>
            <label className="flex flex-col gap-1 text-xs text-scaffold-gray">
              {t("ventilation.units.capacity")} *
              <input
                type="number"
                min={1}
                step={1}
                value={form.capaciteitM3h}
                onChange={(e) =>
                  setForm({ ...form, capaciteitM3h: e.target.value })
                }
                className="w-24 rounded border border-primary/20 bg-surface px-1.5 py-1 text-right text-xs tabular-nums text-on-surface"
              />
            </label>
            {form.type === "wtw" && (
              <label className="flex flex-col gap-1 text-xs text-scaffold-gray">
                {t("ventilation.units.efficiency")}
                <input
                  type="number"
                  min={1}
                  max={100}
                  step={1}
                  value={form.rendementPct}
                  onChange={(e) =>
                    setForm({ ...form, rendementPct: e.target.value })
                  }
                  className="w-20 rounded border border-primary/20 bg-surface px-1.5 py-1 text-right text-xs tabular-nums text-on-surface"
                />
              </label>
            )}
            <label className="flex flex-col gap-1 text-xs text-scaffold-gray">
              {t("ventilation.units.sound")}
              <input
                type="number"
                min={1}
                step={1}
                value={form.geluidDb}
                onChange={(e) => setForm({ ...form, geluidDb: e.target.value })}
                className="w-20 rounded border border-primary/20 bg-surface px-1.5 py-1 text-right text-xs tabular-nums text-on-surface"
              />
            </label>
            <Button
              variant="secondary"
              size="sm"
              disabled={formToUnit(form) === null}
              onClick={handleSubmitForm}
            >
              {editingId
                ? t("ventilation.units.customSave")
                : t("ventilation.units.customSubmit")}
            </Button>
          </div>
        </div>
      )}

      {/* Toegewezen units */}
      {assigned.length === 0 ? (
        <p className="text-xs text-on-surface-muted">
          {t("ventilation.units.none")}
        </p>
      ) : (
        <table className="w-full border-collapse text-sm">
          <thead>
            <tr className="border-b border-[var(--oaec-border)] text-left text-xs font-semibold text-scaffold-gray">
              <th className="px-2 py-1.5">{t("ventilation.units.colUnit")}</th>
              <th className="px-2 py-1.5">{t("ventilation.units.type")}</th>
              <th className="px-2 py-1.5 text-right">
                {t("ventilation.units.colCapacity")}
              </th>
              <th className="px-2 py-1.5 text-right">
                {t("ventilation.units.count")}
              </th>
              <th className="px-2 py-1.5 text-right">
                {t("ventilation.units.colTotal")}
              </th>
              <th className="px-2 py-1.5" />
            </tr>
          </thead>
          <tbody>
            {assigned.map(({ unit, aantal }) => (
              <tr
                key={unit.id}
                className="border-b border-[var(--oaec-border-subtle)]"
              >
                <td className="px-2 py-1.5 text-on-surface">
                  <span className="font-medium">
                    {unit.fabrikant} {unit.model}
                  </span>
                  {unit.rendement !== undefined && (
                    <span className="ml-1 text-xs text-scaffold-gray">
                      η {Math.round(unit.rendement * 100)}%
                    </span>
                  )}
                  {unit.geluidDb !== undefined && (
                    <span className="ml-1 text-xs text-scaffold-gray">
                      · {formatDecimals(unit.geluidDb, 0)} dB
                    </span>
                  )}
                  {unit.source === "custom" && (
                    <span className="ml-1.5 rounded-full bg-primary/10 px-1.5 py-0.5 text-[9px] font-semibold text-scaffold-gray">
                      {t("ventilation.units.customBadge")}
                    </span>
                  )}
                </td>
                <td className="px-2 py-1.5 text-xs text-on-surface">
                  {typeLabel(unit.type)}
                </td>
                <td className="px-2 py-1.5 text-right text-xs tabular-nums text-on-surface">
                  {formatDecimals(unit.capaciteitM3h, 0)} m³/h
                </td>
                <td className="px-2 py-1.5 text-right">
                  <input
                    type="number"
                    min={1}
                    step={1}
                    value={aantal}
                    onChange={(e) => {
                      const n = Math.floor(Number(e.target.value));
                      if (Number.isFinite(n) && n >= 1) {
                        onSetAssignment(unit.id, n);
                      }
                    }}
                    className="w-14 rounded border border-primary/20 bg-surface px-1.5 py-0.5 text-right text-xs tabular-nums text-on-surface"
                  />
                </td>
                <td className="px-2 py-1.5 text-right text-xs tabular-nums text-on-surface">
                  {formatDecimals(unit.capaciteitM3h * aantal, 0)} m³/h
                </td>
                <td className="px-2 py-1.5 text-right">
                  {unit.source === "custom" && (
                    <button
                      className="mr-2 text-xs text-primary hover:underline"
                      onClick={() => {
                        setForm(unitToForm(unit));
                        setEditingId(unit.id);
                        setShowForm(true);
                      }}
                    >
                      {t("ventilation.units.edit")}
                    </button>
                  )}
                  <button
                    className="text-xs text-red-600 hover:underline"
                    onClick={() => onRemoveUnit(unit.id)}
                  >
                    {t("ventilation.units.remove")}
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {/* Capaciteitstoets */}
      {assigned.length > 0 ? (
        <UnitCapacitySummary check={unitCapacity} />
      ) : (
        unitCapacity.requiredDm3s > 0 && (
          <p className="text-xs text-on-surface-muted">
            {t("ventilation.units.requirementHint", {
              flow: flowLabel(unitCapacity.requiredDm3s),
            })}
          </p>
        )
      )}
    </div>
  );
}
