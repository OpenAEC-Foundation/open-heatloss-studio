/**
 * F6 fase 3 — gevel-georiënteerde BENG-geometrie-invoer (Uniec-isomorf).
 *
 * Bewerkt het additieve `beng_geometry`-blok op `projectStore` (zie
 * `types/bengGeometry.ts`, gespiegeld aan
 * `crates/openaec-project-shared/src/beng_geometry.rs`). Volgt Uniecs
 * boomstructuur:
 *
 *   bibliotheken (opake constructies + kozijnmerken)
 *     → rekenzones (A_g + bouwwijze)
 *       → gevels (vlak-type, grenst-aan, bruto buiten-opp, constructie-ref)
 *         → ramen (kozijn-ref + aantal + belemmering/zonwering)
 *
 * De client doet lichte plausibiliteits-feedback (refs bestaan, raamopp ≤
 * gevelopp, omtrek verplicht bij vloer-op-grond); de Rust-`validate()` blijft de
 * echte poortwachter en levert een 422 die de BENG-pagina toont zoals F4b.
 *
 * Update-conventie: elke lijst-mutatie schrijft via `updateBengGeometry`
 * (merge-semantiek als het energy-blok) een nieuwe onveranderlijke array terug.
 */
import { useCallback } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "../ui/Button";
import { Card } from "../ui/Card";
import { useProjectStore } from "../../store/projectStore";
import { NumberField, SelectField, TextField } from "./fields";
import {
  adjacencyHasOrientation,
  adjacencyKind,
  adjacencyOrientation,
  adjacencyRequiresOmtrek,
  makeAdjacency,
  type BengAdjacency,
  type BengAdjacencyKind,
  type BengBoundary,
  type BengGeometry,
  type BengWindowPlacement,
  type BengZone,
  type KozijnType,
  type Obstruction,
  type OpaqueConstructionDef,
  type Orientation,
  type RcOrU,
  type ShadingControl,
  type VlakType,
  type WindowDef,
} from "../../types/bengGeometry";

// ---------------------------------------------------------------------------
// Keuzelijsten (label = NL, waarde = normatieve serde-string). Enum-labels
// staan inline zoals de installatie-tab dat doet (HEAT_GENERATORS etc.); de
// structurele teksten lopen via i18n.
// ---------------------------------------------------------------------------

const VLAK_TYPES: Array<{ value: VlakType; label: string }> = [
  { value: "vloer", label: "Vloer" },
  { value: "vloer_boven_buitenlucht", label: "Vloer boven buitenlucht" },
  { value: "gevel", label: "Gevel" },
  { value: "dak", label: "Dak" },
  { value: "kelderwand", label: "Kelderwand" },
  { value: "bodem", label: "Bodem" },
];

const KOZIJN_TYPES: Array<{ value: KozijnType; label: string }> = [
  { value: "raam", label: "Raam" },
  { value: "deur", label: "Deur" },
  { value: "paneel_in_kozijn", label: "Paneel in kozijn" },
];

const ORIENTATIONS: Array<{ value: Orientation; label: string }> = [
  { value: "noord", label: "Noord" },
  { value: "noord_oost", label: "Noordoost" },
  { value: "oost", label: "Oost" },
  { value: "zuid_oost", label: "Zuidoost" },
  { value: "zuid", label: "Zuid" },
  { value: "zuid_west", label: "Zuidwest" },
  { value: "west", label: "West" },
  { value: "noord_west", label: "Noordwest" },
  { value: "horizontaal", label: "Horizontaal (plat dak)" },
];

const ADJACENCY_KINDS: Array<{ value: BengAdjacencyKind; label: string }> = [
  { value: "buitenlucht", label: "Buitenlucht" },
  {
    value: "vloer_op_maaiveld_boven_kruipruimte",
    label: "Op/boven mv; boven kruipruimte",
  },
  {
    value: "vloer_op_maaiveld_boven_grond",
    label: "Op/boven mv; boven grond/spouw (z ≤ 0,3)",
  },
  {
    value: "vloer_op_maaiveld_boven_onverwarmde_kelder",
    label: "Op/boven mv; boven onverwarmde kelder",
  },
  {
    value: "vloer_onder_maaiveld_boven_kruipruimte",
    label: "Onder mv; boven kruipruimte",
  },
  {
    value: "vloer_onder_maaiveld_boven_grond",
    label: "Onder mv; boven grond/spouw (z ≤ 0,3)",
  },
  {
    value: "vloer_onder_maaiveld_boven_onverwarmde_kelder",
    label: "Onder mv; boven onverwarmde kelder",
  },
  { value: "sterk_geventileerd", label: "Sterk geventileerd" },
  { value: "water", label: "Water" },
  {
    value: "aangrenzende_verwarmde_ruimte",
    label: "Aangrenzende verwarmde ruimte (AVR)",
  },
  { value: "aos_forfaitair", label: "AOS forfaitair (onverwarmde serre)" },
  { value: "aor_forfaitair", label: "AOR forfaitair (onverwarmde ruimte)" },
];

const OBSTRUCTIONS: Array<{ value: Obstruction; label: string }> = [
  { value: "none", label: "Geen belemmering" },
  { value: "minimal", label: "Minimale belemmering" },
];

const SHADING_CONTROLS: Array<{ value: ShadingControl; label: string }> = [
  { value: "manual_residential", label: "Handbediend (woningbouw)" },
  { value: "automatic", label: "Automatisch geregeld" },
];

type ThermalKind = "rc" | "u";

const THERMAL_KINDS: Array<{ value: ThermalKind; label: string }> = [
  { value: "rc", label: "Rc [m²·K/W]" },
  { value: "u", label: "U [W/(m²·K)]" },
];

// ---------------------------------------------------------------------------
// Kleine gedeelde bouwstenen
// ---------------------------------------------------------------------------

const uid = (prefix: string) => `${prefix}-${crypto.randomUUID()}`;

/** Compacte inline-waarschuwing (plausibiliteit; server blijft de validator). */
function Warn({ children }: { children: React.ReactNode }) {
  return (
    <p className="text-xs text-amber-500 dark:text-amber-400">⚠ {children}</p>
  );
}

/** Sectiekop met een omschrijving eronder. */
function SectionCard({
  title,
  hint,
  children,
}: {
  title: string;
  hint?: string;
  children: React.ReactNode;
}) {
  return (
    <Card title={title}>
      {hint && (
        <p className="mb-4 text-xs text-on-surface-muted">{hint}</p>
      )}
      {children}
    </Card>
  );
}

// ---------------------------------------------------------------------------
// Editor
// ---------------------------------------------------------------------------

export function BengGeometryEditor() {
  const { t } = useTranslation();
  const bengGeometry = useProjectStore((s) => s.bengGeometry);
  const updateBengGeometry = useProjectStore((s) => s.updateBengGeometry);

  const geo: BengGeometry = bengGeometry ?? {};
  const opaqueDefs = geo.opaque_defs ?? [];
  const windowDefs = geo.window_defs ?? [];
  const zones = geo.zones ?? [];

  const setOpaqueDefs = useCallback(
    (next: OpaqueConstructionDef[]) => updateBengGeometry({ opaque_defs: next }),
    [updateBengGeometry],
  );
  const setWindowDefs = useCallback(
    (next: WindowDef[]) => updateBengGeometry({ window_defs: next }),
    [updateBengGeometry],
  );
  const setZones = useCallback(
    (next: BengZone[]) => updateBengGeometry({ zones: next }),
    [updateBengGeometry],
  );

  return (
    <div className="space-y-4">
      <div className="rounded-md border border-[var(--oaec-border-subtle)] bg-[var(--oaec-bg-subtle)] px-4 py-3 text-xs text-on-surface-muted">
        {t(
          "beng.geometry.intro",
          "Gevel-georiënteerde invoer conform Uniec 3 / NTA 8800: buiten-oppervlakten per gevel op rekenzone-niveau. Vul eerst de bibliotheken, koppel ze daarna aan de gevels. Dit blok overschrijft de room-geometrie in de BENG-berekening.",
        )}
      </div>

      {/* -- Bibliotheek: opake constructies -- */}
      <SectionCard
        title={t("beng.geometry.opaqueTitle", "Bouwkundige bibliotheek — constructies")}
        hint={t(
          "beng.geometry.opaqueHint",
          "Herbruikbare opake constructies (Rc of U). Gevels verwijzen hiernaar.",
        )}
      >
        <div className="space-y-3">
          {opaqueDefs.map((def, idx) => (
            <OpaqueDefRow
              key={def.id}
              def={def}
              onChange={(next) =>
                setOpaqueDefs(opaqueDefs.map((d, i) => (i === idx ? next : d)))
              }
              onRemove={() =>
                setOpaqueDefs(opaqueDefs.filter((_, i) => i !== idx))
              }
              t={t}
            />
          ))}
          {opaqueDefs.length === 0 && (
            <p className="text-sm text-on-surface-muted">
              {t("beng.geometry.opaqueEmpty", "Nog geen constructies.")}
            </p>
          )}
        </div>
        <div className="mt-3">
          <Button
            variant="secondary"
            size="sm"
            onClick={() =>
              setOpaqueDefs([
                ...opaqueDefs,
                {
                  id: uid("def"),
                  omschrijving: "",
                  kind: "gevel",
                  thermal: { rc: 4.5 },
                },
              ])
            }
          >
            {t("beng.geometry.addOpaque", "+ Constructie toevoegen")}
          </Button>
        </div>
      </SectionCard>

      {/* -- Bibliotheek: kozijnmerken -- */}
      <SectionCard
        title={t("beng.geometry.windowTitle", "Kozijn-bibliotheek — merken")}
        hint={t(
          "beng.geometry.windowHint",
          "Kozijnmerken met U, g-waarde en oppervlakte per exemplaar. Ramen verwijzen hiernaar.",
        )}
      >
        <div className="space-y-3">
          {windowDefs.map((def, idx) => (
            <WindowDefRow
              key={def.id}
              def={def}
              onChange={(next) =>
                setWindowDefs(windowDefs.map((d, i) => (i === idx ? next : d)))
              }
              onRemove={() =>
                setWindowDefs(windowDefs.filter((_, i) => i !== idx))
              }
              t={t}
            />
          ))}
          {windowDefs.length === 0 && (
            <p className="text-sm text-on-surface-muted">
              {t("beng.geometry.windowEmpty", "Nog geen kozijnmerken.")}
            </p>
          )}
        </div>
        <div className="mt-3">
          <Button
            variant="secondary"
            size="sm"
            onClick={() =>
              setWindowDefs([
                ...windowDefs,
                {
                  id: uid("merk"),
                  omschrijving: "",
                  kind: "raam",
                  u_w_per_m2k: 1.3,
                  ggl: 0.4,
                  area_m2: 1.0,
                },
              ])
            }
          >
            {t("beng.geometry.addWindow", "+ Kozijnmerk toevoegen")}
          </Button>
        </div>
      </SectionCard>

      {/* -- Rekenzones -- */}
      {zones.map((zone, idx) => (
        <ZoneCard
          key={zone.id}
          zone={zone}
          opaqueDefs={opaqueDefs}
          windowDefs={windowDefs}
          onChange={(next) =>
            setZones(zones.map((z, i) => (i === idx ? next : z)))
          }
          onRemove={() => setZones(zones.filter((_, i) => i !== idx))}
          t={t}
        />
      ))}

      <div>
        <Button
          variant="secondary"
          size="sm"
          onClick={() =>
            setZones([
              ...zones,
              { id: uid("rz"), naam: "", a_g_m2: 0, gevels: [] },
            ])
          }
        >
          {t("beng.geometry.addZone", "+ Rekenzone toevoegen")}
        </Button>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Bibliotheek-rijen
// ---------------------------------------------------------------------------

type Tfn = (key: string, fallback: string) => string;

function OpaqueDefRow({
  def,
  onChange,
  onRemove,
  t,
}: {
  def: OpaqueConstructionDef;
  onChange: (next: OpaqueConstructionDef) => void;
  onRemove: () => void;
  t: Tfn;
}) {
  const thermalKind: ThermalKind = "rc" in def.thermal ? "rc" : "u";
  const thermalValue = "rc" in def.thermal ? def.thermal.rc : def.thermal.u;
  return (
    <div className="grid grid-cols-1 items-end gap-3 rounded-md border border-[var(--oaec-border-subtle)] p-3 sm:grid-cols-5">
      <TextField
        label={t("beng.geometry.omschrijving", "Omschrijving")}
        value={def.omschrijving}
        placeholder="bv. Wand"
        onChange={(v) => onChange({ ...def, omschrijving: v ?? "" })}
      />
      <SelectField
        label={t("beng.geometry.vlakType", "Vlak-type")}
        value={def.kind}
        options={VLAK_TYPES}
        onChange={(v) => onChange({ ...def, kind: v })}
      />
      <SelectField
        label={t("beng.geometry.thermalKind", "Invoer")}
        value={thermalKind}
        options={THERMAL_KINDS}
        onChange={(v) =>
          onChange({
            ...def,
            thermal: (v === "rc"
              ? { rc: thermalValue }
              : { u: thermalValue }) as RcOrU,
          })
        }
      />
      <NumberField
        label={thermalKind === "rc" ? "Rc" : "U"}
        step={0.05}
        value={thermalValue}
        onChange={(v) =>
          onChange({
            ...def,
            thermal: (thermalKind === "rc"
              ? { rc: v ?? 0 }
              : { u: v ?? 0 }) as RcOrU,
          })
        }
      />
      <Button variant="danger" size="sm" onClick={onRemove}>
        {t("beng.geometry.remove", "Verwijder")}
      </Button>
    </div>
  );
}

function WindowDefRow({
  def,
  onChange,
  onRemove,
  t,
}: {
  def: WindowDef;
  onChange: (next: WindowDef) => void;
  onRemove: () => void;
  t: Tfn;
}) {
  return (
    <div className="grid grid-cols-1 items-end gap-3 rounded-md border border-[var(--oaec-border-subtle)] p-3 sm:grid-cols-6">
      <TextField
        label={t("beng.geometry.merk", "Merk")}
        value={def.omschrijving}
        placeholder="bv. A"
        onChange={(v) => onChange({ ...def, omschrijving: v ?? "" })}
      />
      <SelectField
        label={t("beng.geometry.kozijnType", "Type")}
        value={def.kind}
        options={KOZIJN_TYPES}
        onChange={(v) => onChange({ ...def, kind: v })}
      />
      <NumberField
        label="U"
        unit="W/m²K"
        step={0.05}
        value={def.u_w_per_m2k}
        onChange={(v) => onChange({ ...def, u_w_per_m2k: v ?? 0 })}
      />
      <NumberField
        label="ggl"
        step={0.05}
        value={def.ggl}
        placeholder="—"
        onChange={(v) => onChange({ ...def, ggl: v })}
        hint={t("beng.geometry.gglHint", "0 voor opake deur")}
      />
      <NumberField
        label={t("beng.geometry.areaPerUnit", "Opp/stuk")}
        unit="m²"
        step={0.01}
        value={def.area_m2}
        onChange={(v) => onChange({ ...def, area_m2: v ?? 0 })}
      />
      <Button variant="danger" size="sm" onClick={onRemove}>
        {t("beng.geometry.remove", "Verwijder")}
      </Button>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Rekenzone
// ---------------------------------------------------------------------------

function ZoneCard({
  zone,
  opaqueDefs,
  windowDefs,
  onChange,
  onRemove,
  t,
}: {
  zone: BengZone;
  opaqueDefs: OpaqueConstructionDef[];
  windowDefs: WindowDef[];
  onChange: (next: BengZone) => void;
  onRemove: () => void;
  t: Tfn;
}) {
  const gevels = zone.gevels ?? [];
  const setGevels = (next: BengBoundary[]) => onChange({ ...zone, gevels: next });

  return (
    <Card
      title={`${t("beng.geometry.zone", "Rekenzone")}${zone.naam ? ` — ${zone.naam}` : ""}`}
    >
      <div className="space-y-4">
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-4">
          <TextField
            label={t("beng.geometry.zoneNaam", "Naam")}
            value={zone.naam}
            placeholder="bv. woning"
            onChange={(v) => onChange({ ...zone, naam: v ?? "" })}
          />
          <NumberField
            label="A_g"
            unit="m²"
            step={0.1}
            value={zone.a_g_m2}
            onChange={(v) => onChange({ ...zone, a_g_m2: v ?? 0 })}
            hint={t("beng.geometry.agHint", "Gebruiksoppervlak (noemer BENG)")}
          />
          <TextField
            label={t("beng.geometry.bouwwijzeVloer", "Bouwwijze vloer")}
            value={zone.bouwwijze_vloer}
            placeholder="Uniec-code"
            onChange={(v) => onChange({ ...zone, bouwwijze_vloer: v })}
          />
          <TextField
            label={t("beng.geometry.woningtype", "Woningtype")}
            value={zone.woningtype}
            placeholder="Uniec-code"
            onChange={(v) => onChange({ ...zone, woningtype: v })}
          />
        </div>

        {zone.a_g_m2 <= 0 && (
          <Warn>
            {t("beng.geometry.warnAg", "A_g moet groter dan 0 zijn.")}
          </Warn>
        )}

        {/* Gevels */}
        <div className="space-y-3">
          <h4 className="text-sm font-semibold text-on-surface">
            {t("beng.geometry.gevels", "Gevels (thermische schil)")}
          </h4>
          {gevels.map((gevel, idx) => (
            <BoundaryCard
              key={gevel.id}
              gevel={gevel}
              opaqueDefs={opaqueDefs}
              windowDefs={windowDefs}
              onChange={(next) =>
                setGevels(gevels.map((g, i) => (i === idx ? next : g)))
              }
              onRemove={() => setGevels(gevels.filter((_, i) => i !== idx))}
              t={t}
            />
          ))}
          {gevels.length === 0 && (
            <p className="text-sm text-on-surface-muted">
              {t("beng.geometry.gevelsEmpty", "Nog geen gevels in deze zone.")}
            </p>
          )}
          <Button
            variant="secondary"
            size="sm"
            onClick={() =>
              setGevels([
                ...gevels,
                {
                  id: uid("gevel"),
                  omschrijving: "",
                  vlak_type: "gevel",
                  grenst_aan: makeAdjacency("buitenlucht", "zuid"),
                  bruto_buiten_opp_m2: 0,
                  helling_deg: 90,
                  constructie_ref: opaqueDefs[0]?.id ?? "",
                  ramen: [],
                },
              ])
            }
          >
            {t("beng.geometry.addGevel", "+ Gevel toevoegen")}
          </Button>
        </div>

        <div className="border-t border-[var(--oaec-border-subtle)] pt-3">
          <Button variant="danger" size="sm" onClick={onRemove}>
            {t("beng.geometry.removeZone", "Rekenzone verwijderen")}
          </Button>
        </div>
      </div>
    </Card>
  );
}

// ---------------------------------------------------------------------------
// Gevel (begrenzing)
// ---------------------------------------------------------------------------

function BoundaryCard({
  gevel,
  opaqueDefs,
  windowDefs,
  onChange,
  onRemove,
  t,
}: {
  gevel: BengBoundary;
  opaqueDefs: OpaqueConstructionDef[];
  windowDefs: WindowDef[];
  onChange: (next: BengBoundary) => void;
  onRemove: () => void;
  t: Tfn;
}) {
  const ramen = gevel.ramen ?? [];
  const setRamen = (next: BengWindowPlacement[]) =>
    onChange({ ...gevel, ramen: next });

  // -- Plausibiliteits-feedback (server blijft de validator) --
  const constructieOk =
    gevel.constructie_ref !== "" &&
    opaqueDefs.some((d) => d.id === gevel.constructie_ref);
  const ramenOpp = ramen.reduce((sum, r) => {
    const def = windowDefs.find((d) => d.id === r.kozijn_ref);
    return sum + (def ? (r.aantal ?? 1) * def.area_m2 : 0);
  }, 0);
  const ramenExceed = ramenOpp - gevel.bruto_buiten_opp_m2 > 1e-9;
  const omtrekMissing =
    adjacencyRequiresOmtrek(gevel.grenst_aan) &&
    (gevel.omtrek_p_m == null || gevel.omtrek_p_m <= 0);

  const constructieOptions = [
    { value: "", label: t("beng.geometry.pickConstruction", "— kies —") },
    ...opaqueDefs.map((d) => ({
      value: d.id,
      label: d.omschrijving || d.id,
    })),
  ];

  return (
    <div className="space-y-3 rounded-md border border-[var(--oaec-border)] p-3">
      <div className="grid grid-cols-1 gap-3 sm:grid-cols-3">
        <TextField
          label={t("beng.geometry.omschrijving", "Omschrijving")}
          value={gevel.omschrijving}
          placeholder="bv. Wand"
          onChange={(v) => onChange({ ...gevel, omschrijving: v ?? "" })}
        />
        <SelectField
          label={t("beng.geometry.vlakType", "Vlak-type")}
          value={gevel.vlak_type}
          options={VLAK_TYPES}
          onChange={(v) => onChange({ ...gevel, vlak_type: v })}
        />
        <SelectField
          label={t("beng.geometry.constructie", "Constructie")}
          value={
            constructieOptions.some((o) => o.value === gevel.constructie_ref)
              ? gevel.constructie_ref
              : ""
          }
          options={constructieOptions}
          onChange={(v) => onChange({ ...gevel, constructie_ref: v })}
        />
      </div>

      <AdjacencyEditor
        value={gevel.grenst_aan}
        onChange={(next) => onChange({ ...gevel, grenst_aan: next })}
        t={t}
      />

      <div className="grid grid-cols-1 gap-3 sm:grid-cols-3">
        <NumberField
          label={t("beng.geometry.brutoOpp", "Bruto buiten-opp")}
          unit="m²"
          step={0.01}
          value={gevel.bruto_buiten_opp_m2}
          onChange={(v) => onChange({ ...gevel, bruto_buiten_opp_m2: v ?? 0 })}
        />
        <NumberField
          label={t("beng.geometry.helling", "Helling")}
          unit="°"
          step={1}
          value={gevel.helling_deg}
          placeholder="—"
          onChange={(v) => onChange({ ...gevel, helling_deg: v })}
          hint={t("beng.geometry.hellingHint", "90 = gevel, 15 = hellend dak")}
        />
        {adjacencyRequiresOmtrek(gevel.grenst_aan) && (
          <NumberField
            label={t("beng.geometry.omtrek", "Omtrek P")}
            unit="m"
            step={0.01}
            value={gevel.omtrek_p_m}
            onChange={(v) => onChange({ ...gevel, omtrek_p_m: v })}
            hint={t("beng.geometry.omtrekHint", "P/A-methode vloer-op-grond")}
          />
        )}
      </div>

      {!constructieOk && (
        <Warn>
          {t(
            "beng.geometry.warnConstruction",
            "Kies een geldige constructie uit de bibliotheek.",
          )}
        </Warn>
      )}
      {omtrekMissing && (
        <Warn>
          {t(
            "beng.geometry.warnOmtrek",
            "Omtrek P is verplicht bij een vloer-op-grond.",
          )}
        </Warn>
      )}
      {gevel.bruto_buiten_opp_m2 <= 0 && (
        <Warn>
          {t("beng.geometry.warnOpp", "Bruto oppervlak moet groter dan 0 zijn.")}
        </Warn>
      )}
      {ramenExceed && (
        <Warn>
          {t(
            "beng.geometry.warnRamen",
            "Totaal raamoppervlak overschrijdt het bruto gevelvlak",
          )}{" "}
          ({ramenOpp.toFixed(2)} &gt; {gevel.bruto_buiten_opp_m2.toFixed(2)} m²).
        </Warn>
      )}

      {/* Ramen */}
      <div className="space-y-2 border-t border-[var(--oaec-border-subtle)] pt-3">
        <div className="flex items-center justify-between">
          <h5 className="text-xs font-semibold uppercase tracking-wider text-on-surface-muted">
            {t("beng.geometry.ramen", "Ramen / deuren")}
          </h5>
          <span className="text-xs text-on-surface-muted">
            Σ {ramenOpp.toFixed(2)} m²
          </span>
        </div>
        {ramen.map((raam, idx) => (
          <WindowPlacementRow
            key={idx}
            raam={raam}
            windowDefs={windowDefs}
            onChange={(next) =>
              setRamen(ramen.map((r, i) => (i === idx ? next : r)))
            }
            onRemove={() => setRamen(ramen.filter((_, i) => i !== idx))}
            t={t}
          />
        ))}
        <div className="flex gap-2">
          <Button
            variant="secondary"
            size="sm"
            disabled={windowDefs.length === 0}
            onClick={() =>
              setRamen([
                ...ramen,
                {
                  kozijn_ref: windowDefs[0]?.id ?? "",
                  aantal: 1,
                  belemmering: "minimal",
                  zomernachtventilatie: false,
                },
              ])
            }
          >
            {t("beng.geometry.addRaam", "+ Raam toevoegen")}
          </Button>
          {windowDefs.length === 0 && (
            <span className="self-center text-xs text-on-surface-muted">
              {t(
                "beng.geometry.needWindowDef",
                "Voeg eerst een kozijnmerk toe.",
              )}
            </span>
          )}
        </div>
      </div>

      <div className="border-t border-[var(--oaec-border-subtle)] pt-3">
        <Button variant="danger" size="sm" onClick={onRemove}>
          {t("beng.geometry.removeGevel", "Gevel verwijderen")}
        </Button>
      </div>
    </div>
  );
}

/** Grenst-aan-editor: keuze-sleutel + (bij buitenlucht/AOS) oriëntatie. */
function AdjacencyEditor({
  value,
  onChange,
  t,
}: {
  value: BengAdjacency;
  onChange: (next: BengAdjacency) => void;
  t: Tfn;
}) {
  const kind = adjacencyKind(value);
  const orientatie = adjacencyOrientation(value);
  return (
    <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
      <SelectField
        label={t("beng.geometry.grenstAan", "Grenst aan")}
        value={kind}
        options={ADJACENCY_KINDS}
        onChange={(v) => onChange(makeAdjacency(v, orientatie))}
      />
      {adjacencyHasOrientation(kind) && (
        <SelectField
          label={t("beng.geometry.orientatie", "Oriëntatie")}
          value={orientatie ?? "zuid"}
          options={ORIENTATIONS}
          onChange={(v) => onChange(makeAdjacency(kind, v))}
        />
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Kozijn-plaatsing (raam)
// ---------------------------------------------------------------------------

function WindowPlacementRow({
  raam,
  windowDefs,
  onChange,
  onRemove,
  t,
}: {
  raam: BengWindowPlacement;
  windowDefs: WindowDef[];
  onChange: (next: BengWindowPlacement) => void;
  onRemove: () => void;
  t: Tfn;
}) {
  const refOk = windowDefs.some((d) => d.id === raam.kozijn_ref);
  const zonwering = raam.zonwering ?? null;
  const kozijnOptions = [
    { value: "", label: t("beng.geometry.pickWindow", "— kies —") },
    ...windowDefs.map((d) => ({ value: d.id, label: d.omschrijving || d.id })),
  ];
  return (
    <div className="space-y-2 rounded-md border border-[var(--oaec-border-subtle)] bg-[var(--oaec-bg-subtle)] p-3">
      <div className="grid grid-cols-1 items-end gap-3 sm:grid-cols-4">
        <SelectField
          label={t("beng.geometry.kozijnmerk", "Kozijnmerk")}
          value={refOk ? raam.kozijn_ref : ""}
          options={kozijnOptions}
          onChange={(v) => onChange({ ...raam, kozijn_ref: v })}
        />
        <NumberField
          label={t("beng.geometry.aantal", "Aantal")}
          step={1}
          value={raam.aantal ?? 1}
          onChange={(v) =>
            onChange({ ...raam, aantal: v == null ? 1 : Math.max(1, Math.round(v)) })
          }
        />
        <SelectField
          label={t("beng.geometry.belemmering", "Belemmering")}
          value={raam.belemmering ?? "none"}
          options={OBSTRUCTIONS}
          onChange={(v) => onChange({ ...raam, belemmering: v })}
        />
        <Button variant="danger" size="sm" onClick={onRemove}>
          {t("beng.geometry.remove", "Verwijder")}
        </Button>
      </div>

      <div className="flex flex-wrap items-center gap-4">
        <label className="flex items-center gap-2 text-sm text-on-surface">
          <input
            type="checkbox"
            checked={zonwering != null}
            onChange={(e) =>
              onChange({
                ...raam,
                zonwering: e.target.checked
                  ? { f_c: 0.3, control: "manual_residential" }
                  : null,
              })
            }
            className="h-4 w-4 accent-[var(--oaec-primary,#6d28d9)]"
          />
          <span>{t("beng.geometry.zonwering", "Beweegbare zonwering")}</span>
        </label>
        <label className="flex items-center gap-2 text-sm text-on-surface">
          <input
            type="checkbox"
            checked={raam.zomernachtventilatie ?? false}
            onChange={(e) =>
              onChange({ ...raam, zomernachtventilatie: e.target.checked })
            }
            className="h-4 w-4 accent-[var(--oaec-primary,#6d28d9)]"
          />
          <span>{t("beng.geometry.znvent", "Zomernachtventilatie")}</span>
        </label>
      </div>

      {zonwering != null && (
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
          <NumberField
            label={t("beng.geometry.fc", "Reductiefactor F_c")}
            step={0.05}
            value={zonwering.f_c}
            onChange={(v) =>
              onChange({ ...raam, zonwering: { ...zonwering, f_c: v ?? 0 } })
            }
            hint={t("beng.geometry.fcHint", "Tabel 7.5/7.6, 0..1")}
          />
          <SelectField
            label={t("beng.geometry.zonweringControl", "Bediening")}
            value={zonwering.control}
            options={SHADING_CONTROLS}
            onChange={(v) =>
              onChange({ ...raam, zonwering: { ...zonwering, control: v } })
            }
          />
        </div>
      )}

      {!refOk && (
        <Warn>
          {t(
            "beng.geometry.warnWindowRef",
            "Kies een geldig kozijnmerk uit de bibliotheek.",
          )}
        </Warn>
      )}
    </div>
  );
}
