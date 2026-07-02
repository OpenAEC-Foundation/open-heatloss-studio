/**
 * Pure naam-helpers voor zones (`Building.zones`). Losgekoppeld van `ZonesCard`
 * zodat de dedup-regel unit-testbaar is en gedeeld wordt tussen het toevoeg- en
 * het hernoem-pad.
 *
 * Waarom dedup: zones worden bij de Revit thermal-import op naam
 * ge-find-or-create'd (`lib/thermalImport.ts`). Twee zones met dezelfde naam
 * maken die mapping niet-deterministisch (de naam→id `Map` laat er willekeurig
 * één winnen). Daarom weren we duplicaten — trim + case-insensitief — al bij het
 * aanmaken/hernoemen in de UI.
 */
import type { Zone } from "../../types";

/** Trim whitespace; de canonieke opslagvorm van een zonenaam. */
export function normalizeZoneName(name: string): string {
  return name.trim();
}

/**
 * Bestaat er al een zone met deze naam (trim + case-insensitief)? `exceptId`
 * sluit één zone uit — nodig op het hernoem-pad zodat een zone zichzelf niet
 * als duplicaat ziet. Een lege (na trim) naam telt nooit als duplicaat.
 */
export function zoneNameExists(
  zones: readonly Zone[],
  name: string,
  exceptId?: string,
): boolean {
  const norm = name.trim().toLowerCase();
  if (norm === "") return false;
  return zones.some(
    (z) => z.id !== exceptId && z.name.trim().toLowerCase() === norm,
  );
}
