/**
 * Help-pagina — in-app documentatie in vier secties:
 * Gebruik, Formules, Afwijkingen (t.o.v. Vabi/norm) en Verificatie
 * (placeholder, wordt in een vervolg gevuld).
 *
 * Sub-tab-patroon volgt `pages/Library.tsx`. Content staat in
 * `content/help/` zodat tekst en presentatie gescheiden blijven.
 */
import { useState } from "react";

import { PageHeader } from "../components/layout/PageHeader";
import { HelpAfwijkingen } from "../content/help/HelpAfwijkingen";
import { HelpFormules } from "../content/help/HelpFormules";
import { HelpGebruik } from "../content/help/HelpGebruik";
import { HelpVerificatie } from "../content/help/HelpVerificatie";

export type HelpSection = "gebruik" | "formules" | "afwijkingen" | "verificatie";

const SECTION_ORDER: ReadonlyArray<HelpSection> = [
  "gebruik",
  "formules",
  "afwijkingen",
  "verificatie",
];

const SECTION_LABELS: Record<HelpSection, string> = {
  gebruik: "Gebruik",
  formules: "Formules",
  afwijkingen: "Afwijkingen",
  verificatie: "Verificatie",
};

export function Help({ initialSection = "gebruik" }: { initialSection?: HelpSection } = {}) {
  const [section, setSection] = useState<HelpSection>(initialSection);

  return (
    <div>
      <PageHeader title="Help" subtitle={SECTION_LABELS[section]} />

      <div className="mx-auto max-w-5xl p-4">
        {/* Sectie-tabs */}
        <div className="mb-4 flex gap-1 rounded-lg border border-[var(--oaec-border)] bg-surface-alt p-1">
          {SECTION_ORDER.map((key) => (
            <button
              key={key}
              type="button"
              onClick={() => setSection(key)}
              className={`rounded-md px-5 py-2 text-sm font-medium transition-colors ${
                section === key
                  ? "bg-[var(--oaec-bg-lighter)] text-on-surface shadow-sm"
                  : "text-on-surface-muted hover:text-on-surface-secondary"
              }`}
            >
              {SECTION_LABELS[key]}
            </button>
          ))}
        </div>

        {section === "gebruik" && <HelpGebruik />}
        {section === "formules" && <HelpFormules />}
        {section === "afwijkingen" && <HelpAfwijkingen />}
        {section === "verificatie" && <HelpVerificatie />}
      </div>
    </div>
  );
}
