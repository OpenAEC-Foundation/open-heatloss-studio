import { Pencil } from "lucide-react";
import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type KeyboardEvent as ReactKeyboardEvent,
  type MouseEvent as ReactMouseEvent,
} from "react";
import { useNavigate } from "react-router-dom";

import { PageHeader } from "../components/layout/PageHeader";
import { Button } from "../components/ui/Button";
import {
  CATALOGUE_CATEGORY_LABELS,
  type CatalogueCategory,
  type CatalogueEntry,
} from "../lib/constructionCatalogue";
import { VERTICAL_POSITION_LABELS } from "../lib/constants";
import {
  MATERIAL_CATEGORY_LABELS,
  MATERIAL_CATEGORY_ORDER,
  type Material,
  type MaterialCategory,
} from "../lib/materialsDatabase";
import { useCatalogueStore } from "../store/catalogueStore";
import { useMaterialsStore } from "../store/materialsStore";
import type { MaterialType, VerticalPosition } from "../types";

type LibrarySection = "constructies" | "materialen";

// ────────────────────────────────────────────
// Constructies — constanten
// ────────────────────────────────────────────

const CONSTR_CATEGORY_ORDER: CatalogueCategory[] = [
  "wanden",
  "vloeren_plafonds",
  "daken",
  "kozijnen_vullingen",
];

const CONSTR_CATEGORY_ICONS: Record<CatalogueCategory, string> = {
  wanden: "\u2B1C",
  vloeren_plafonds: "\u2B1B",
  daken: "\u25B3",
  kozijnen_vullingen: "\u25A3",
};

const MATERIAL_TYPE_LABELS: Record<MaterialType, string> = {
  masonry: "Steenachtig",
  non_masonry: "Niet-steenachtig",
};

const NAME_EDIT_INPUT_CLASS =
  "min-w-0 flex-1 rounded border border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] px-1.5 py-0 text-sm font-medium text-on-surface outline-none focus:border-primary";

const NAME_EDIT_ICON_SIZE_CLASS = "h-3.5 w-3.5";

const EMPTY_ENTRY: Omit<CatalogueEntry, "id"> = {
  name: "",
  category: "wanden",
  uValue: 0,
  materialType: "masonry",
  verticalPosition: "wall",
};

// ────────────────────────────────────────────
// Materialen — constanten
// ────────────────────────────────────────────

interface MaterialDraft {
  name: string;
  category: MaterialCategory;
  brand: string;
  lambda: string;
  lambdaWet: string;
  mu: string;
  rho: string;
  keywords: string;
}

const EMPTY_MAT_DRAFT: MaterialDraft = {
  name: "",
  category: "metselwerk",
  brand: "",
  lambda: "",
  lambdaWet: "",
  mu: "",
  rho: "",
  keywords: "",
};

// ============================================================
// Library (main)
// ============================================================

export function Library({ initialSection = "constructies" }: { initialSection?: LibrarySection } = {}) {
  const [section, setSection] = useState<LibrarySection>(initialSection);

  return (
    <div>
      <PageHeader
        title="Bibliotheek"
        subtitle={section === "constructies" ? "Constructies" : "Materialen"}
        breadcrumbs={[{ label: "Bibliotheek" }]}
      />

      <div className="p-4">
        {/* Top-level toggle */}
        <div className="mb-4 flex gap-1 rounded-lg border border-[var(--oaec-border)] bg-surface-alt p-1">
          <button
            type="button"
            onClick={() => setSection("constructies")}
            className={`rounded-md px-5 py-2 text-sm font-medium transition-colors ${
              section === "constructies"
                ? "bg-[var(--oaec-bg-lighter)] text-on-surface shadow-sm"
                : "text-on-surface-muted hover:text-on-surface-secondary"
            }`}
          >
            Constructies
          </button>
          <button
            type="button"
            onClick={() => setSection("materialen")}
            className={`rounded-md px-5 py-2 text-sm font-medium transition-colors ${
              section === "materialen"
                ? "bg-[var(--oaec-bg-lighter)] text-on-surface shadow-sm"
                : "text-on-surface-muted hover:text-on-surface-secondary"
            }`}
          >
            Materialen
          </button>
        </div>

        {section === "constructies" ? <ConstructionsView /> : <MaterialsView />}
      </div>
    </div>
  );
}

// ============================================================
// Constructies view
// ============================================================

function ConstructionsView() {
  const entries = useCatalogueStore((s) => s.entries);
  const addEntry = useCatalogueStore((s) => s.addEntry);
  const updateEntry = useCatalogueStore((s) => s.updateEntry);
  const removeEntry = useCatalogueStore((s) => s.removeEntry);
  const duplicateEntry = useCatalogueStore((s) => s.duplicateEntry);
  const resetEntry = useCatalogueStore((s) => s.resetEntry);
  const resetAll = useCatalogueStore((s) => s.resetAll);
  const isModified = useCatalogueStore((s) => s.isModified);

  const [activeTab, setActiveTab] = useState<CatalogueCategory>("wanden");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [showAddForm, setShowAddForm] = useState(false);
  const [draft, setDraft] = useState<Omit<CatalogueEntry, "id">>({ ...EMPTY_ENTRY });

  const filtered = useMemo(
    () => entries.filter((e) => e.category === activeTab),
    [entries, activeTab],
  );

  const categoryCounts = useMemo(() => {
    const counts: Record<string, number> = {};
    for (const e of entries) {
      counts[e.category] = (counts[e.category] ?? 0) + 1;
    }
    return counts;
  }, [entries]);

  const handleAdd = useCallback(() => {
    if (!draft.name.trim()) return;
    addEntry({ ...draft, category: activeTab });
    setDraft({ ...EMPTY_ENTRY, category: activeTab });
    setShowAddForm(false);
  }, [addEntry, draft, activeTab]);

  const handleStartAdd = useCallback(() => {
    setDraft({ ...EMPTY_ENTRY, category: activeTab });
    setShowAddForm(true);
    setEditingId(null);
  }, [activeTab]);

  const handleCancelAdd = useCallback(() => {
    setShowAddForm(false);
    setDraft({ ...EMPTY_ENTRY });
  }, []);

  return (
    <>
      {/* Actions */}
      <div className="mb-4 flex items-center justify-between">
        <div />
        <div className="flex gap-2">
          <button
            type="button"
            onClick={() => {
              if (window.confirm("Alle aanpassingen ongedaan maken en standaardwaarden herstellen?")) {
                resetAll();
                setEditingId(null);
                setShowAddForm(false);
              }
            }}
            className="rounded-md border border-[var(--oaec-border)] px-3 py-1.5 text-sm text-on-surface-secondary hover:bg-surface-alt"
          >
            Standaardwaarden herstellen
          </button>
          <Button onClick={handleStartAdd}>+ Constructie toevoegen</Button>
        </div>
      </div>

      {/* Category tabs */}
      <div className="mb-4 flex gap-1 rounded-lg border border-[var(--oaec-border)] bg-surface-alt p-1">
        {CONSTR_CATEGORY_ORDER.map((cat) => (
          <button
            key={cat}
            type="button"
            onClick={() => {
              setActiveTab(cat);
              setEditingId(null);
              setShowAddForm(false);
            }}
            className={`flex items-center gap-2 rounded-md px-4 py-2 text-sm font-medium transition-colors
              ${
                activeTab === cat
                  ? "bg-[var(--oaec-bg-lighter)] text-on-surface shadow-sm"
                  : "text-on-surface-muted hover:text-on-surface-secondary"
              }`}
          >
            <span>{CONSTR_CATEGORY_ICONS[cat]}</span>
            {CATALOGUE_CATEGORY_LABELS[cat]}
            <span className="ml-1 rounded-full bg-[var(--oaec-hover)] px-1.5 py-0.5 text-xs tabular-nums text-on-surface-muted">
              {categoryCounts[cat] ?? 0}
            </span>
          </button>
        ))}
      </div>

      {/* Add form */}
      {showAddForm && (
        <div className="mb-4 rounded-lg border-2 border-dashed border-blue-300 bg-blue-600/15/50 p-4">
          <h3 className="mb-3 text-sm font-semibold text-on-surface-secondary">
            Nieuwe constructie toevoegen aan {CATALOGUE_CATEGORY_LABELS[activeTab]}
          </h3>
          <ConstructionForm
            draft={draft}
            onChange={(partial) => setDraft((prev) => ({ ...prev, ...partial }))}
            onSubmit={handleAdd}
            onCancel={handleCancelAdd}
            submitLabel="Toevoegen"
          />
        </div>
      )}

      {/* Entry table */}
      <div className="overflow-hidden rounded-lg border border-[var(--oaec-border)]">
        <table className="w-full border-collapse text-sm">
          <thead>
            <tr className="border-b-2 border-[var(--oaec-border)] bg-surface-alt text-left text-xs font-semibold uppercase tracking-wider text-on-surface-secondary">
              <th className="px-3 py-2.5">Beschrijving</th>
              <th className="w-[120px] px-3 py-2.5 text-right">U-waarde</th>
              <th className="w-[140px] px-3 py-2.5">Materiaal</th>
              <th className="w-[100px] px-3 py-2.5">Positie</th>
              <th className="w-[100px] px-3 py-2.5" />
            </tr>
          </thead>
          <tbody>
            {filtered.map((entry) => (
              <ConstructionRow
                key={entry.id}
                entry={entry}
                isEditing={editingId === entry.id}
                modified={isModified(entry.id)}
                onEdit={() => {
                  setEditingId(entry.id);
                  setShowAddForm(false);
                }}
                onCancelEdit={() => setEditingId(null)}
                onUpdate={(partial) => {
                  updateEntry(entry.id, partial);
                  setEditingId(null);
                }}
                onDuplicate={() => duplicateEntry(entry.id)}
                onRemove={() => {
                  removeEntry(entry.id);
                  if (editingId === entry.id) setEditingId(null);
                }}
                onReset={entry.isBuiltIn ? () => resetEntry(entry.id) : undefined}
              />
            ))}
            {filtered.length === 0 && (
              <tr>
                <td colSpan={5} className="px-3 py-8 text-center text-sm text-on-surface-muted">
                  Geen constructies in deze categorie.
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </>
  );
}

// ============================================================
// Materialen view
// ============================================================

function MaterialsView() {
  const materials = useMaterialsStore((s) => s.materials);
  const addMaterial = useMaterialsStore((s) => s.addMaterial);
  const updateMaterial = useMaterialsStore((s) => s.updateMaterial);
  const removeMaterial = useMaterialsStore((s) => s.removeMaterial);
  const resetMaterial = useMaterialsStore((s) => s.resetMaterial);
  const resetAll = useMaterialsStore((s) => s.resetAll);
  const isModified = useMaterialsStore((s) => s.isModified);

  const [search, setSearch] = useState("");
  const [showAddForm, setShowAddForm] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [draft, setDraft] = useState<MaterialDraft>({ ...EMPTY_MAT_DRAFT });

  // Filter materials by search
  const filtered = useMemo(() => {
    if (!search.trim()) return materials;
    const terms = search.toLowerCase().split(/\s+/).filter(Boolean);
    return materials.filter((m) => {
      const haystack = [m.name, m.brand ?? "", ...m.keywords].join(" ").toLowerCase();
      return terms.every((t) => haystack.includes(t));
    });
  }, [materials, search]);

  // Group by category in display order
  const grouped = useMemo(() => {
    const map = new Map<MaterialCategory, Material[]>();
    for (const cat of MATERIAL_CATEGORY_ORDER) {
      const items = filtered.filter((m) => m.category === cat);
      if (items.length > 0) map.set(cat, items);
    }
    return map;
  }, [filtered]);

  const parseKeywords = (input: string): string[] =>
    input.split(",").map((s) => s.trim()).filter(Boolean);

  const handleAdd = useCallback(() => {
    if (!draft.name.trim()) return;
    addMaterial({
      name: draft.name.trim(),
      category: draft.category,
      brand: draft.brand.trim() || null,
      lambda: draft.lambda ? Number(draft.lambda) : null,
      lambdaWet: draft.lambdaWet ? Number(draft.lambdaWet) : null,
      mu: Number(draft.mu) || 1,
      rho: draft.rho ? Number(draft.rho) : null,
      rdFixed: null,
      sdFixed: null,
      keywords: parseKeywords(draft.keywords),
    });
    setDraft({ ...EMPTY_MAT_DRAFT });
    setShowAddForm(false);
  }, [addMaterial, draft]);

  return (
    <>
      {/* Actions */}
      <div className="mb-4 flex items-center justify-between gap-4">
        <input
          type="text"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="Zoek materiaal..."
          className="w-64 rounded-md border border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] text-on-surface px-3 py-1.5 text-sm focus:border-primary focus:outline-none"
        />
        <div className="flex gap-2">
          <button
            type="button"
            onClick={() => {
              if (window.confirm("Alle aanpassingen ongedaan maken en standaardwaarden herstellen?")) {
                resetAll();
              }
            }}
            className="rounded-md border border-[var(--oaec-border)] px-3 py-1.5 text-sm text-on-surface-secondary hover:bg-surface-alt"
          >
            Standaardwaarden herstellen
          </button>
          <Button onClick={() => setShowAddForm((v) => !v)}>
            + Materiaal toevoegen
          </Button>
        </div>
      </div>

      {/* Add form */}
      {showAddForm && (
        <div className="mb-4 rounded-lg border-2 border-dashed border-blue-300 bg-blue-600/15/50 p-4">
          <h3 className="mb-3 text-sm font-semibold text-on-surface-secondary">
            Nieuw materiaal toevoegen
          </h3>
          <MaterialAddForm
            draft={draft}
            onChange={(partial) => setDraft((prev) => ({ ...prev, ...partial }))}
            onSubmit={handleAdd}
            onCancel={() => {
              setShowAddForm(false);
              setDraft({ ...EMPTY_MAT_DRAFT });
            }}
          />
        </div>
      )}

      {/* Grouped table */}
      <div className="overflow-hidden rounded-lg border border-[var(--oaec-border)]">
        <table className="w-full border-collapse text-sm">
          <thead>
            <tr className="border-b-2 border-[var(--oaec-border)] bg-surface-alt text-left text-xs font-semibold uppercase tracking-wider text-on-surface-secondary">
              <th className="px-3 py-2.5">Naam</th>
              <th className="w-[120px] px-3 py-2.5">Merk</th>
              <th className="w-[100px] px-3 py-2.5 text-right">{"\u03C1"} [kg/m{"\u00B3"}]</th>
              <th className="w-[100px] px-3 py-2.5 text-right">{"\u03BB"} [W/mK]</th>
              <th className="w-[100px] px-3 py-2.5 text-right">{"\u03BB"} nat</th>
              <th className="w-[80px] px-3 py-2.5 text-right">{"\u03BC"} [-]</th>
              <th className="w-[60px] px-3 py-2.5" />
            </tr>
          </thead>
          <tbody>
            {[...grouped.entries()].map(([cat, items]) => (
              <MaterialCategoryGroup
                key={cat}
                category={cat}
                materials={items}
                editingId={editingId}
                onEdit={(id) => { setEditingId(id); setShowAddForm(false); }}
                onCancelEdit={() => setEditingId(null)}
                onUpdate={(id, partial) => { updateMaterial(id, partial); setEditingId(null); }}
                onRemove={removeMaterial}
                onReset={resetMaterial}
                isModified={isModified}
              />
            ))}
            {grouped.size === 0 && (
              <tr>
                <td colSpan={7} className="px-3 py-8 text-center text-sm text-on-surface-muted">
                  Geen materialen gevonden.
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </>
  );
}

/* ─── Material category group ─── */

interface MaterialCategoryGroupProps {
  category: MaterialCategory;
  materials: Material[];
  editingId: string | null;
  onEdit: (id: string) => void;
  onCancelEdit: () => void;
  onUpdate: (id: string, partial: Partial<Material>) => void;
  onRemove: (id: string) => void;
  onReset: (id: string) => void;
  isModified: (id: string) => boolean;
}

function MaterialCategoryGroup({
  category,
  materials,
  editingId,
  onEdit,
  onCancelEdit,
  onUpdate,
  onRemove,
  onReset,
  isModified,
}: MaterialCategoryGroupProps) {
  const [collapsed, setCollapsed] = useState(true);

  return (
    <>
      <tr
        className="cursor-pointer select-none bg-[var(--oaec-hover)] hover:bg-surface-alt"
        onClick={() => setCollapsed((v) => !v)}
      >
        <td
          colSpan={7}
          className="px-3 py-2 text-xs font-bold uppercase tracking-wider text-on-surface-muted"
        >
          <span className="mr-1.5 inline-block w-3 text-center text-[10px]">
            {collapsed ? "\u25B6" : "\u25BC"}
          </span>
          {MATERIAL_CATEGORY_LABELS[category]}
          <span className="ml-2 font-normal text-on-surface-muted">({materials.length})</span>
        </td>
      </tr>
      {!collapsed && materials.map((m) => (
        <MaterialRow
          key={m.id}
          material={m}
          isEditing={editingId === m.id}
          modified={isModified(m.id)}
          onEdit={() => onEdit(m.id)}
          onCancelEdit={onCancelEdit}
          onUpdate={(partial) => onUpdate(m.id, partial)}
          onRemove={() => onRemove(m.id)}
          onReset={m.isBuiltIn ? () => onReset(m.id) : undefined}
        />
      ))}
    </>
  );
}

/* ─── Material row (view + inline edit) ─── */

interface MaterialRowProps {
  material: Material;
  isEditing: boolean;
  modified: boolean;
  onEdit: () => void;
  onCancelEdit: () => void;
  onUpdate: (partial: Partial<Material>) => void;
  onRemove: () => void;
  onReset?: () => void;
}

function MaterialRow({
  material,
  isEditing,
  modified,
  onEdit,
  onCancelEdit,
  onUpdate,
  onRemove,
  onReset,
}: MaterialRowProps) {
  const [draft, setDraft] = useState<MaterialDraft>({
    name: "",
    category: "metselwerk",
    brand: "",
    lambda: "",
    lambdaWet: "",
    mu: "",
    rho: "",
    keywords: "",
  });

  const parseKeywords = (input: string): string[] =>
    input.split(",").map((s) => s.trim()).filter(Boolean);

  const handleStartEdit = useCallback(() => {
    setDraft({
      name: material.name,
      category: material.category,
      brand: material.brand ?? "",
      lambda: material.lambda !== null ? String(material.lambda) : "",
      lambdaWet: material.lambdaWet !== null ? String(material.lambdaWet) : "",
      mu: String(material.mu),
      rho: material.rho !== null ? String(material.rho) : "",
      keywords: material.keywords.join(", "),
    });
    onEdit();
  }, [material, onEdit]);

  const handleSave = useCallback(() => {
    if (!draft.name.trim()) return;
    onUpdate({
      name: draft.name.trim(),
      brand: draft.brand.trim() || null,
      lambda: draft.lambda ? Number(draft.lambda) : null,
      lambdaWet: draft.lambdaWet ? Number(draft.lambdaWet) : null,
      mu: Number(draft.mu) || 1,
      rho: draft.rho ? Number(draft.rho) : null,
      keywords: parseKeywords(draft.keywords),
    });
  }, [draft, onUpdate]);

  const handleCancel = useCallback(() => {
    onCancelEdit();
  }, [onCancelEdit]);

  if (isEditing) {
    return (
      <tr className="border-b border-[var(--oaec-border-subtle)] bg-[var(--oaec-accent-soft)]">
        <td className="px-2 py-1.5">
          <input
            type="text"
            value={draft.name}
            onChange={(e) => setDraft((p) => ({ ...p, name: e.target.value }))}
            className="w-full rounded border border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] text-on-surface px-2 py-1 text-sm focus:border-primary focus:outline-none"
            autoFocus
          />
          <input
            type="text"
            value={draft.keywords}
            onChange={(e) => setDraft((p) => ({ ...p, keywords: e.target.value }))}
            placeholder="Zoekwoorden (kommagescheiden)"
            className="mt-1 w-full rounded border border-[var(--oaec-border)] px-2 py-1 text-xs text-on-surface-secondary focus:border-primary focus:outline-none"
          />
        </td>
        <td className="px-2 py-1.5">
          <input
            type="text"
            value={draft.brand}
            onChange={(e) => setDraft((p) => ({ ...p, brand: e.target.value }))}
            placeholder="-"
            className="w-full rounded border border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] text-on-surface px-2 py-1 text-sm focus:border-primary focus:outline-none"
          />
        </td>
        <td className="px-2 py-1.5">
          <input
            type="number"
            value={draft.rho}
            onChange={(e) => setDraft((p) => ({ ...p, rho: e.target.value }))}
            step="any"
            className="w-full rounded border border-[var(--oaec-border)] px-2 py-1 text-right text-sm tabular-nums focus:border-primary focus:outline-none"
          />
        </td>
        <td className="px-2 py-1.5">
          <input
            type="number"
            value={draft.lambda}
            onChange={(e) => setDraft((p) => ({ ...p, lambda: e.target.value }))}
            step="any"
            className="w-full rounded border border-[var(--oaec-border)] px-2 py-1 text-right text-sm tabular-nums focus:border-primary focus:outline-none"
          />
        </td>
        <td className="px-2 py-1.5">
          <input
            type="number"
            value={draft.lambdaWet}
            onChange={(e) => setDraft((p) => ({ ...p, lambdaWet: e.target.value }))}
            step="any"
            className="w-full rounded border border-[var(--oaec-border)] px-2 py-1 text-right text-sm tabular-nums focus:border-primary focus:outline-none"
          />
        </td>
        <td className="px-2 py-1.5">
          <input
            type="number"
            value={draft.mu}
            onChange={(e) => setDraft((p) => ({ ...p, mu: e.target.value }))}
            step="any"
            className="w-full rounded border border-[var(--oaec-border)] px-2 py-1 text-right text-sm tabular-nums focus:border-primary focus:outline-none"
          />
        </td>
        <td className="px-2 py-1.5">
          <div className="flex justify-end gap-1">
            <button
              type="button"
              onClick={handleSave}
              className="rounded bg-blue-600 px-2 py-0.5 text-xs font-medium text-white hover:bg-blue-700"
            >
              Opslaan
            </button>
            <button
              type="button"
              onClick={handleCancel}
              className="rounded border border-[var(--oaec-border)] px-2 py-0.5 text-xs text-on-surface-secondary hover:bg-surface-alt"
            >
              Annuleer
            </button>
          </div>
        </td>
      </tr>
    );
  }

  return (
    <tr className="group border-b border-[var(--oaec-border-subtle)] hover:bg-[var(--oaec-hover)]/50">
      <td className="px-3 py-2 font-medium text-on-surface">
        {material.name}
        {!material.isBuiltIn && (
          <span className="ml-2 rounded bg-blue-600/20 px-1.5 py-0.5 text-[10px] font-semibold uppercase text-blue-400">
            Aangepast
          </span>
        )}
        {modified && (
          <span className="ml-2 rounded bg-amber-600/15 px-1.5 py-0.5 text-[10px] font-semibold uppercase text-amber-400">
            Gewijzigd
          </span>
        )}
      </td>
      <td className="px-3 py-2 text-on-surface-muted">
        {material.brand ?? <span className="text-on-surface-muted">-</span>}
      </td>
      <td className="px-3 py-2 text-right tabular-nums text-on-surface-secondary">
        {material.rho !== null ? material.rho : <span className="text-on-surface-muted">-</span>}
      </td>
      <td className="px-3 py-2 text-right tabular-nums text-on-surface-secondary">
        {material.lambda !== null ? material.lambda : <span className="text-on-surface-muted">-</span>}
      </td>
      <td className="px-3 py-2 text-right tabular-nums text-on-surface-secondary">
        {material.lambdaWet !== null ? material.lambdaWet : <span className="text-on-surface-muted">-</span>}
      </td>
      <td className="px-3 py-2 text-right tabular-nums text-on-surface-secondary">
        {material.mu}
      </td>
      <td className="px-3 py-2">
        <div className="flex justify-end gap-1 opacity-0 transition-opacity group-hover:opacity-100">
          <button
            type="button"
            onClick={handleStartEdit}
            className="rounded px-2 py-0.5 text-xs text-on-surface-muted hover:bg-surface-alt hover:text-on-surface-secondary"
            title="Bewerken"
          >
            Bewerk
          </button>
          {modified && onReset && (
            <button
              type="button"
              onClick={onReset}
              className="rounded px-2 py-0.5 text-xs text-amber-400 hover:bg-amber-600/15 hover:text-amber-400"
              title="Herstel naar standaardwaarden"
            >
              Herstel
            </button>
          )}
          <button
            type="button"
            onClick={onRemove}
            className="rounded px-2 py-0.5 text-xs text-red-400 hover:bg-red-600/15 hover:text-red-400"
            title="Verwijderen"
          >
            Verwijder
          </button>
        </div>
      </td>
    </tr>
  );
}

/* ─── Material add form ─── */

function MaterialAddForm({
  draft,
  onChange,
  onSubmit,
  onCancel,
}: {
  draft: MaterialDraft;
  onChange: (partial: Partial<MaterialDraft>) => void;
  onSubmit: () => void;
  onCancel: () => void;
}) {
  return (
    <div className="flex flex-wrap items-end gap-3">
      <label className="flex flex-1 flex-col gap-1 text-xs font-medium text-on-surface-secondary">
        Naam
        <input
          type="text"
          value={draft.name}
          onChange={(e) => onChange({ name: e.target.value })}
          placeholder="Bijv. PIR 023"
          className="rounded border border-[var(--oaec-border)] px-2 py-1.5 text-sm text-on-surface focus:border-primary focus:outline-none"
          autoFocus
        />
      </label>
      <label className="flex w-32 flex-col gap-1 text-xs font-medium text-on-surface-secondary">
        Categorie
        <select
          value={draft.category}
          onChange={(e) => onChange({ category: e.target.value as MaterialCategory })}
          className="rounded border border-[var(--oaec-border)] px-2 py-1.5 text-sm text-on-surface focus:border-primary focus:outline-none"
        >
          {MATERIAL_CATEGORY_ORDER.map((cat) => (
            <option key={cat} value={cat}>{MATERIAL_CATEGORY_LABELS[cat]}</option>
          ))}
        </select>
      </label>
      <label className="flex w-28 flex-col gap-1 text-xs font-medium text-on-surface-secondary">
        Merk
        <input
          type="text"
          value={draft.brand}
          onChange={(e) => onChange({ brand: e.target.value })}
          placeholder="of leeg"
          className="rounded border border-[var(--oaec-border)] px-2 py-1.5 text-sm text-on-surface focus:border-primary focus:outline-none"
        />
      </label>
      <label className="flex w-20 flex-col gap-1 text-xs font-medium text-on-surface-secondary">
        {"\u03C1"} [kg/m{"\u00B3"}]
        <input
          type="number"
          value={draft.rho}
          onChange={(e) => onChange({ rho: e.target.value })}
          step="any"
          className="rounded border border-[var(--oaec-border)] px-2 py-1.5 text-sm tabular-nums text-on-surface focus:border-primary focus:outline-none"
        />
      </label>
      <label className="flex w-20 flex-col gap-1 text-xs font-medium text-on-surface-secondary">
        {"\u03BB"} [W/mK]
        <input
          type="number"
          value={draft.lambda}
          onChange={(e) => onChange({ lambda: e.target.value })}
          step="any"
          className="rounded border border-[var(--oaec-border)] px-2 py-1.5 text-sm tabular-nums text-on-surface focus:border-primary focus:outline-none"
        />
      </label>
      <label className="flex w-20 flex-col gap-1 text-xs font-medium text-on-surface-secondary">
        {"\u03BB"} nat
        <input
          type="number"
          value={draft.lambdaWet}
          onChange={(e) => onChange({ lambdaWet: e.target.value })}
          step="any"
          className="rounded border border-[var(--oaec-border)] px-2 py-1.5 text-sm tabular-nums text-on-surface focus:border-primary focus:outline-none"
        />
      </label>
      <label className="flex w-20 flex-col gap-1 text-xs font-medium text-on-surface-secondary">
        {"\u03BC"} [-]
        <input
          type="number"
          value={draft.mu}
          onChange={(e) => onChange({ mu: e.target.value })}
          step="any"
          className="rounded border border-[var(--oaec-border)] px-2 py-1.5 text-sm tabular-nums text-on-surface focus:border-primary focus:outline-none"
        />
      </label>
      <label className="flex flex-1 flex-col gap-1 text-xs font-medium text-on-surface-secondary">
        Zoekwoorden
        <input
          type="text"
          value={draft.keywords}
          onChange={(e) => onChange({ keywords: e.target.value })}
          placeholder="kommagescheiden, bijv. pir, isolatie"
          className="rounded border border-[var(--oaec-border)] px-2 py-1.5 text-sm text-on-surface focus:border-primary focus:outline-none"
        />
      </label>
      <div className="flex gap-2">
        <Button onClick={onSubmit}>Toevoegen</Button>
        <button
          type="button"
          onClick={onCancel}
          className="rounded-md border border-[var(--oaec-border)] px-3 py-1.5 text-sm text-on-surface-secondary hover:bg-surface-alt"
        >
          Annuleer
        </button>
      </div>
    </div>
  );
}

// ============================================================
// Constructie row + form (existing, renamed)
// ============================================================

interface ConstructionRowProps {
  entry: CatalogueEntry;
  isEditing: boolean;
  modified: boolean;
  onEdit: () => void;
  onCancelEdit: () => void;
  onUpdate: (partial: Partial<CatalogueEntry>) => void;
  onDuplicate: () => void;
  onRemove: () => void;
  onReset?: () => void;
}

function ConstructionRow({
  entry,
  isEditing,
  modified,
  onEdit,
  onCancelEdit,
  onUpdate,
  onDuplicate,
  onRemove,
  onReset,
}: ConstructionRowProps) {
  const [draft, setDraft] = useState<Partial<CatalogueEntry>>({});
  const navigate = useNavigate();

  // Inline quick-rename state (apart van full edit form).
  const [isRenamingName, setIsRenamingName] = useState(false);
  const [draftName, setDraftName] = useState(entry.name);
  const nameInputRef = useRef<HTMLInputElement | null>(null);

  // Focus + select-all bij openen van rename-mode.
  useEffect(() => {
    if (isRenamingName && nameInputRef.current) {
      nameInputRef.current.focus();
      nameInputRef.current.select();
    }
  }, [isRenamingName]);

  const handleStartEdit = useCallback(() => {
    if (entry.layers?.length) {
      navigate(`/rc?edit=${entry.id}`);
      return;
    }
    setDraft({
      name: entry.name,
      uValue: entry.uValue,
      materialType: entry.materialType,
      verticalPosition: entry.verticalPosition,
    });
    onEdit();
  }, [entry, onEdit, navigate]);

  const handleSave = useCallback(() => {
    if (!draft.name?.trim()) return;
    onUpdate(draft);
    setDraft({});
  }, [draft, onUpdate]);

  const handleCancel = useCallback(() => {
    setDraft({});
    onCancelEdit();
  }, [onCancelEdit]);

  const startRenameName = (e: ReactMouseEvent<HTMLButtonElement>): void => {
    e.stopPropagation();
    setDraftName(entry.name);
    setIsRenamingName(true);
  };

  const cancelRenameName = (): void => {
    setIsRenamingName(false);
    setDraftName(entry.name);
  };

  const commitRenameName = (): void => {
    const trimmed = draftName.trim();
    if (trimmed.length === 0 || trimmed === entry.name) {
      cancelRenameName();
      return;
    }
    onUpdate({ name: trimmed });
    setIsRenamingName(false);
  };

  const handleNameKeyDown = (
    e: ReactKeyboardEvent<HTMLInputElement>,
  ): void => {
    e.stopPropagation();
    if (e.key === "Enter") {
      e.preventDefault();
      commitRenameName();
    } else if (e.key === "Escape") {
      e.preventDefault();
      cancelRenameName();
    }
  };

  if (isEditing) {
    return (
      <tr className="border-b border-[var(--oaec-border-subtle)] bg-[var(--oaec-accent-soft)]">
        <td className="px-3 py-2" colSpan={4}>
          <ConstructionForm
            draft={{
              name: draft.name ?? entry.name,
              category: entry.category,
              uValue: draft.uValue ?? entry.uValue,
              materialType: draft.materialType ?? entry.materialType,
              verticalPosition: draft.verticalPosition ?? entry.verticalPosition,
            }}
            onChange={(d) => setDraft((prev) => ({ ...prev, ...d }))}
            onSubmit={handleSave}
            onCancel={handleCancel}
            submitLabel="Opslaan"
          />
        </td>
        <td />
      </tr>
    );
  }

  return (
    <tr className="group border-b border-[var(--oaec-border-subtle)] hover:bg-[var(--oaec-hover)]/50">
      <td className="px-3 py-2.5 font-medium text-on-surface">
        <div className="flex items-center gap-2">
          {isRenamingName ? (
            <input
              ref={nameInputRef}
              type="text"
              value={draftName}
              onChange={(e) => setDraftName(e.target.value)}
              onKeyDown={handleNameKeyDown}
              onBlur={commitRenameName}
              onClick={(e) => e.stopPropagation()}
              onMouseDown={(e) => e.stopPropagation()}
              className={NAME_EDIT_INPUT_CLASS}
              aria-label="Constructienaam bewerken"
            />
          ) : (
            <>
              <span>{entry.name}</span>
              <button
                type="button"
                onClick={startRenameName}
                className="rounded p-0.5 text-on-surface-muted hover:bg-[var(--oaec-hover)] hover:text-on-surface-secondary"
                aria-label="Naam bewerken"
                title="Constructienaam bewerken"
              >
                <Pencil className={NAME_EDIT_ICON_SIZE_CLASS} />
              </button>
            </>
          )}
          {!entry.isBuiltIn && (
            <span className="rounded bg-blue-600/20 px-1.5 py-0.5 text-[10px] font-semibold uppercase text-blue-400">
              Aangepast
            </span>
          )}
          {modified && (
            <span className="rounded bg-amber-600/15 px-1.5 py-0.5 text-[10px] font-semibold uppercase text-amber-400">
              Gewijzigd
            </span>
          )}
        </div>
      </td>
      <td className="px-3 py-2.5 text-right tabular-nums text-on-surface-secondary">
        {entry.uValue.toFixed(2)}
        <span className="ml-1 text-xs text-on-surface-muted">W/m²K</span>
      </td>
      <td className="px-3 py-2.5 text-on-surface-secondary">
        {MATERIAL_TYPE_LABELS[entry.materialType]}
      </td>
      <td className="px-3 py-2.5 text-on-surface-secondary">
        {VERTICAL_POSITION_LABELS[entry.verticalPosition]}
      </td>
      <td className="px-3 py-2.5">
        <div className="flex justify-end gap-1 opacity-0 transition-opacity group-hover:opacity-100">
          <button
            type="button"
            onClick={handleStartEdit}
            className="rounded px-2 py-0.5 text-xs text-on-surface-muted hover:bg-surface-alt hover:text-on-surface-secondary"
            title="Bewerken"
          >
            Bewerk
          </button>
          <button
            type="button"
            onClick={onDuplicate}
            className="rounded px-2 py-0.5 text-xs text-on-surface-muted hover:bg-blue-600/20 hover:text-blue-400"
            title="Dupliceren"
          >
            Kopieer
          </button>
          {modified && onReset && (
            <button
              type="button"
              onClick={onReset}
              className="rounded px-2 py-0.5 text-xs text-amber-400 hover:bg-amber-600/15 hover:text-amber-400"
              title="Herstel naar standaardwaarden"
            >
              Herstel
            </button>
          )}
          <button
            type="button"
            onClick={onRemove}
            className="rounded px-2 py-0.5 text-xs text-red-400 hover:bg-red-600/15 hover:text-red-400"
            title="Verwijderen"
          >
            Verwijder
          </button>
        </div>
      </td>
    </tr>
  );
}

/* ─── Construction form ─── */

interface ConstructionFormProps {
  draft: Omit<CatalogueEntry, "id">;
  onChange: (partial: Partial<Omit<CatalogueEntry, "id">>) => void;
  onSubmit: () => void;
  onCancel: () => void;
  submitLabel: string;
}

function ConstructionForm({ draft, onChange, onSubmit, onCancel, submitLabel }: ConstructionFormProps) {
  return (
    <div className="flex flex-wrap items-end gap-3">
      <label className="flex flex-1 flex-col gap-1 text-xs font-medium text-on-surface-secondary">
        Naam
        <input
          type="text"
          value={draft.name}
          onChange={(e) => onChange({ name: e.target.value })}
          placeholder="Bijv. Buitenwand (metselwerk)"
          className="rounded border border-[var(--oaec-border)] px-2 py-1.5 text-sm text-on-surface focus:border-primary focus:outline-none"
          autoFocus
        />
      </label>
      <label className="flex w-28 flex-col gap-1 text-xs font-medium text-on-surface-secondary">
        U-waarde [W/m²K]
        <input
          type="number"
          value={draft.uValue}
          onChange={(e) => onChange({ uValue: Number(e.target.value) || 0 })}
          step="0.01"
          min="0"
          className="rounded border border-[var(--oaec-border)] px-2 py-1.5 text-sm tabular-nums text-on-surface focus:border-primary focus:outline-none"
        />
      </label>
      <label className="flex w-36 flex-col gap-1 text-xs font-medium text-on-surface-secondary">
        Materiaal
        <select
          value={draft.materialType}
          onChange={(e) => onChange({ materialType: e.target.value as MaterialType })}
          className="rounded border border-[var(--oaec-border)] px-2 py-1.5 text-sm text-on-surface focus:border-primary focus:outline-none"
        >
          {Object.entries(MATERIAL_TYPE_LABELS).map(([k, v]) => (
            <option key={k} value={k}>{v}</option>
          ))}
        </select>
      </label>
      <label className="flex w-28 flex-col gap-1 text-xs font-medium text-on-surface-secondary">
        Positie
        <select
          value={draft.verticalPosition}
          onChange={(e) => onChange({ verticalPosition: e.target.value as VerticalPosition })}
          className="rounded border border-[var(--oaec-border)] px-2 py-1.5 text-sm text-on-surface focus:border-primary focus:outline-none"
        >
          {Object.entries(VERTICAL_POSITION_LABELS).map(([k, v]) => (
            <option key={k} value={k}>{v}</option>
          ))}
        </select>
      </label>
      <div className="flex gap-2">
        <Button onClick={onSubmit}>{submitLabel}</Button>
        <button
          type="button"
          onClick={onCancel}
          className="rounded-md border border-[var(--oaec-border)] px-3 py-1.5 text-sm text-on-surface-secondary hover:bg-surface-alt"
        >
          Annuleer
        </button>
      </div>
    </div>
  );
}
