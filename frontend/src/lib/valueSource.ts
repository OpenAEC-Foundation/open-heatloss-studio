/**
 * Bronregistratie-helpers (F4c) — keuzelijst + formattering voor de
 * NTA 8800-dossierplicht van prestatiewaarden.
 *
 * Puur presentatie/UI: de bron beïnvloedt de berekening niet (zie de Rust
 * `energy.rs`-module-doc). De `value`-strings spiegelen exact de serde-casing
 * van `ValueSourceKind` / `BengSubsystem`.
 */
import type { BengSubsystem, ValueSourceKind, ValueSourceReport } from "../types/beng";

/** Keuzelijst voor de bron-select (NL-label; serde-`value` is normatief). */
export const VALUE_SOURCE_KINDS: ReadonlyArray<{
  value: ValueSourceKind;
  label: string;
}> = [
  { value: "forfait", label: "Forfait (norm)" },
  { value: "kwaliteitsverklaring", label: "Kwaliteitsverklaring (BCRG)" },
  { value: "gelijkwaardigheidsverklaring", label: "Gelijkwaardigheidsverklaring" },
  { value: "meting", label: "Meting" },
  { value: "overig", label: "Overig" },
];

/** NL-label voor een bron-soort (fallback = de ruwe serde-waarde). */
export function valueSourceKindLabel(kind: ValueSourceKind): string {
  return VALUE_SOURCE_KINDS.find((k) => k.value === kind)?.label ?? kind;
}

/** NL-label per deelsysteem (spiegelt de Rust `source_note`-mapping). */
export const BENG_SUBSYSTEM_LABELS: Record<BengSubsystem, string> = {
  heating: "Verwarming",
  dhw: "Warm tapwater",
  dwtw: "Douchewater-WTW",
  ventilation: "Ventilatie",
  cooling: "Koeling",
  pv: "PV",
};

/** NL-label voor een deelsysteem (fallback = de ruwe serde-waarde). */
export function bengSubsystemLabel(system: BengSubsystem): string {
  return BENG_SUBSYSTEM_LABELS[system] ?? system;
}

/**
 * Compacte, menselijk-leesbare regel voor één bronregistratie in het resultaat,
 * bv. `"Verwarming (dak-zuid): Kwaliteitsverklaring (BCRG), ref. BCRG-123"`.
 */
export function formatValueSourceReport(r: ValueSourceReport): string {
  const system = bengSubsystemLabel(r.system);
  const label = r.label ? ` (${r.label})` : "";
  const kind = valueSourceKindLabel(r.kind);
  const reference =
    r.reference && r.reference.trim() !== "" ? `, ref. ${r.reference.trim()}` : "";
  return `${system}${label}: ${kind}${reference}`;
}
