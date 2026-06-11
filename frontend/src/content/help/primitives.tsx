/**
 * Gedeelde presentatie-bouwstenen voor de help-secties.
 *
 * Bewust puur presentationeel (geen store, geen router) zodat de
 * content-modules in `content/help/` server-side renderbaar blijven
 * voor de smoke-tests (`pages/Help.test.tsx`, renderToString in
 * node-environment).
 */
import type { ReactNode } from "react";

/** Hoofdformule-blok: monospace, geaccentueerd, met optionele norm-referentie. */
export function FormulaBlock({
  formula,
  reference,
}: {
  formula: string;
  reference?: string;
}) {
  return (
    <div className="my-3 rounded-md border border-[var(--oaec-border-subtle)] bg-surface-alt px-4 py-3">
      <code className="block font-mono text-sm text-on-surface">{formula}</code>
      {reference && (
        <p className="mt-1.5 text-xs text-on-surface-muted">{reference}</p>
      )}
    </div>
  );
}

/** Symboolverklaring als compacte definitielijst. */
export function SymbolList({
  symbols,
}: {
  symbols: ReadonlyArray<readonly [symbol: string, meaning: string]>;
}) {
  return (
    <dl className="my-2 grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-sm">
      {symbols.map(([sym, meaning]) => (
        <div key={sym} className="contents">
          <dt className="font-mono text-on-surface">{sym}</dt>
          <dd className="text-on-surface-secondary">{meaning}</dd>
        </div>
      ))}
    </dl>
  );
}

/** Genummerde werkflow-stap. */
export function Step({
  nr,
  title,
  children,
}: {
  nr: number;
  title: string;
  children: ReactNode;
}) {
  return (
    <li className="flex gap-3">
      <span className="mt-0.5 flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-primary text-xs font-semibold text-on-accent">
        {nr}
      </span>
      <div className="min-w-0">
        <h4 className="text-sm font-semibold text-on-surface">{title}</h4>
        <div className="mt-1 text-sm leading-relaxed text-on-surface-secondary">
          {children}
        </div>
      </div>
    </li>
  );
}

/** Inline keyboard/route badge. */
export function Kbd({ children }: { children: ReactNode }) {
  return (
    <kbd className="rounded border border-[var(--oaec-border)] bg-surface-alt px-1.5 py-0.5 font-mono text-xs text-on-surface">
      {children}
    </kbd>
  );
}
