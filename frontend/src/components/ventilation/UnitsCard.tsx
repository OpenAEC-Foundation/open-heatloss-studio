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
 * **Eenheden:** capaciteit wordt **opgeslagen** in m³/h (fabrikant-conventie,
 * afgerond op hele m³/h); de weergave en het invoerveld volgen de
 * UI-eenheidstoggle (`unit`-prop, zie `ventilationUiStore`).
 */

import { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

import {
  FLOW_UNIT_LABELS,
  dm3sToM3h,
  m3hToDm3s,
  ventilationSystemOf,
  type FlowDisplayUnit,
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
import { UnitCapacitySummary, flowDisplayLabel } from "./shared";

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface UnitsCardProps {
  /** Ventilatie-sidecar (units + toewijzingen + systeem). */
  ventilation: VentilationState;
  /** Uitkomst van de capaciteitstoets (uit `useVentilationBalance`). */
  unitCapacity: UnitCapacityCheck;
  /** Weergave-eenheid voor capaciteiten/debieten (opslag blijft m³/h resp. dm³/s). */
  unit: FlowDisplayUnit;
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

export interface UnitFormState {
  type: VentilationUnitType;
  fabrikant: string;
  model: string;
  /** Capaciteit in de actieve weergave-eenheid (string, vrij invoerveld). */
  capaciteit: string;
  rendementPct: string;
  geluidDb: string;
}

const EMPTY_FORM: UnitFormState = {
  type: "wtw",
  fabrikant: "",
  model: "",
  capaciteit: "",
  rendementPct: "",
  geluidDb: "",
};

/**
 * Opgeslagen capaciteit (m³/h) → invoerveld-string in de weergave-eenheid.
 * dm³/s op max 2 decimalen: ruim genoeg dat ×3,6 + afronding op hele m³/h
 * weer exact op de opgeslagen waarde uitkomt (max fout 0,005 × 3,6 < 0,5).
 */
export function capacityToFormValue(
  m3h: number,
  unit: FlowDisplayUnit,
): string {
  return unit === "m3h"
    ? String(Math.round(m3h))
    : String(Number(m3hToDm3s(m3h).toFixed(2)));
}

export function formToUnit(
  form: UnitFormState,
  unit: FlowDisplayUnit,
): Omit<VentilationUnit, "id" | "source"> | null {
  // Verdediging in de diepte (naast de input-attributen): ongeldige invoer
  // levert `null` — de submit-knop blijft dan disabled en er wordt géén unit
  // aangemaakt. Invoer is in de weergave-eenheid; opslag blijft m³/h
  // (fabrikant-conventie), afgerond op hele m³/h.
  const entered = Number(form.capaciteit);
  const capaciteitM3h = Math.round(
    unit === "m3h" ? entered : dm3sToM3h(entered),
  );
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

function unitToForm(
  unit: VentilationUnit,
  displayUnit: FlowDisplayUnit,
): UnitFormState {
  return {
    type: unit.type,
    fabrikant: unit.fabrikant,
    model: unit.model,
    capaciteit: capacityToFormValue(unit.capaciteitM3h, displayUnit),
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
  unit,
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

  // Het capaciteit-invoerveld is in de weergave-eenheid; wisselt de toggle
  // terwijl het formulier open staat, reken de getypte waarde dan mee om
  // zodat de betekenis niet stilzwijgend verandert.
  const prevUnitRef = useRef(unit);
  useEffect(() => {
    const from = prevUnitRef.current;
    if (from === unit) return;
    prevUnitRef.current = unit;
    setForm((f) => {
      const n = Number(f.capaciteit);
      if (f.capaciteit === "" || !Number.isFinite(n)) return f;
      const m3h = from === "m3h" ? n : dm3sToM3h(n);
      return { ...f, capaciteit: capacityToFormValue(m3h, unit) };
    });
  }, [unit]);

  /** Opgeslagen capaciteit (m³/h) → weergave-label in de actieve eenheid. */
  const capacityLabel = (m3h: number) => flowDisplayLabel(m3hToDm3s(m3h), unit);

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
    const parsed = formToUnit(form, unit);
    if (!parsed) return;
    if (editingId) {
      onUpdateUnit(editingId, parsed);
    } else {
      onAddCustomUnit(parsed);
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
        <span className="font-medium oa-warning-text">
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
                {u.fabrikant} {u.model} — {capacityLabel(u.capaciteitM3h)}
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
              {t("ventilation.units.colCapacity")} ({FLOW_UNIT_LABELS[unit]}) *
              <input
                type="number"
                min={unit === "m3h" ? 1 : 0.1}
                step={unit === "m3h" ? 1 : 0.1}
                value={form.capaciteit}
                onChange={(e) =>
                  setForm({ ...form, capaciteit: e.target.value })
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
              disabled={formToUnit(form, unit) === null}
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
            {assigned.map(({ unit: u, aantal }) => (
              <tr
                key={u.id}
                className="border-b border-[var(--oaec-border-subtle)]"
              >
                <td className="px-2 py-1.5 text-on-surface">
                  <span className="font-medium">
                    {u.fabrikant} {u.model}
                  </span>
                  {u.rendement !== undefined && (
                    <span className="ml-1 text-xs text-scaffold-gray">
                      η {Math.round(u.rendement * 100)}%
                    </span>
                  )}
                  {u.geluidDb !== undefined && (
                    <span className="ml-1 text-xs text-scaffold-gray">
                      · {formatDecimals(u.geluidDb, 0)} dB
                    </span>
                  )}
                  {u.source === "custom" && (
                    <span className="ml-1.5 rounded-full bg-primary/10 px-1.5 py-0.5 text-[9px] font-semibold text-scaffold-gray">
                      {t("ventilation.units.customBadge")}
                    </span>
                  )}
                </td>
                <td className="px-2 py-1.5 text-xs text-on-surface">
                  {typeLabel(u.type)}
                </td>
                <td className="px-2 py-1.5 text-right text-xs tabular-nums text-on-surface">
                  {capacityLabel(u.capaciteitM3h)}
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
                        onSetAssignment(u.id, n);
                      }
                    }}
                    className="w-14 rounded border border-primary/20 bg-surface px-1.5 py-0.5 text-right text-xs tabular-nums text-on-surface"
                  />
                </td>
                <td className="px-2 py-1.5 text-right text-xs tabular-nums text-on-surface">
                  {capacityLabel(u.capaciteitM3h * aantal)}
                </td>
                <td className="px-2 py-1.5 text-right">
                  {u.source === "custom" && (
                    <button
                      className="mr-2 text-xs text-primary hover:underline"
                      onClick={() => {
                        setForm(unitToForm(u, unit));
                        setEditingId(u.id);
                        setShowForm(true);
                      }}
                    >
                      {t("ventilation.units.edit")}
                    </button>
                  )}
                  <button
                    className="text-xs text-red-600 hover:underline"
                    onClick={() => onRemoveUnit(u.id)}
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
        <UnitCapacitySummary check={unitCapacity} unit={unit} />
      ) : (
        unitCapacity.requiredDm3s > 0 && (
          <p className="text-xs text-on-surface-muted">
            {t("ventilation.units.requirementHint", {
              flow: flowDisplayLabel(unitCapacity.requiredDm3s, unit),
            })}
          </p>
        )
      )}
    </div>
  );
}
