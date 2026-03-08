import { useCallback, useMemo, useState } from "react";
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
}

const EMPTY_MAT_DRAFT: MaterialDraft = {
  name: "",
  category: "metselwerk",
  brand: "",
  lambda: "",
  lambdaWet: "",
  mu: "",
  rho: "",
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
      />

      <div className="p-4">
        {/* Top-level toggle */}
        <div className="mb-4 flex gap-1 rounded-lg border border-stone-200 bg-stone-100 p-1">
          <button
            type="button"
            onClick={() => setSection("constructies")}
            className={`rounded-md px-5 py-2 text-sm font-medium transition-colors ${
              section === "constructies"
                ? "bg-white text-stone-900 shadow-sm"
                : "text-stone-500 hover:text-stone-700"
            }`}
          >
            Constructies
          </button>
          <button
            type="button"
            onClick={() => setSection("materialen")}
            className={`rounded-md px-5 py-2 text-sm font-medium transition-colors ${
              section === "materialen"
                ? "bg-white text-stone-900 shadow-sm"
                : "text-stone-500 hover:text-stone-700"
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
            className="rounded-md border border-stone-300 px-3 py-1.5 text-sm text-stone-600 hover:bg-stone-100"
          >
            Standaardwaarden herstellen
          </button>
          <Button onClick={handleStartAdd}>+ Constructie toevoegen</Button>
        </div>
      </div>

      {/* Category tabs */}
      <div className="mb-4 flex gap-1 rounded-lg border border-stone-200 bg-stone-100 p-1">
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
                  ? "bg-white text-stone-900 shadow-sm"
                  : "text-stone-500 hover:text-stone-700"
              }`}
          >
            <span>{CONSTR_CATEGORY_ICONS[cat]}</span>
            {CATALOGUE_CATEGORY_LABELS[cat]}
            <span className="ml-1 rounded-full bg-stone-200/80 px-1.5 py-0.5 text-xs tabular-nums text-stone-500">
              {categoryCounts[cat] ?? 0}
            </span>
          </button>
        ))}
      </div>

      {/* Add form */}
      {showAddForm && (
        <div className="mb-4 rounded-lg border-2 border-dashed border-blue-300 bg-blue-50/50 p-4">
          <h3 className="mb-3 text-sm font-semibold text-stone-700">
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
      <div className="overflow-hidden rounded-lg border border-stone-200">
        <table className="w-full border-collapse text-sm">
          <thead>
            <tr className="border-b-2 border-stone-300 bg-stone-100 text-left text-xs font-semibold uppercase tracking-wider text-stone-600">
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
                <td colSpan={5} className="px-3 py-8 text-center text-sm text-stone-400">
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
  const removeMaterial = useMaterialsStore((s) => s.removeMaterial);
  const resetAll = useMaterialsStore((s) => s.resetAll);

  const [search, setSearch] = useState("");
  const [showAddForm, setShowAddForm] = useState(false);
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
      keywords: [],
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
          className="w-64 rounded-md border border-stone-200 px-3 py-1.5 text-sm focus:border-blue-400 focus:outline-none"
        />
        <div className="flex gap-2">
          <button
            type="button"
            onClick={() => {
              if (window.confirm("Alle aanpassingen ongedaan maken en standaardwaarden herstellen?")) {
                resetAll();
              }
            }}
            className="rounded-md border border-stone-300 px-3 py-1.5 text-sm text-stone-600 hover:bg-stone-100"
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
        <div className="mb-4 rounded-lg border-2 border-dashed border-blue-300 bg-blue-50/50 p-4">
          <h3 className="mb-3 text-sm font-semibold text-stone-700">
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
      <div className="overflow-hidden rounded-lg border border-stone-200">
        <table className="w-full border-collapse text-sm">
          <thead>
            <tr className="border-b-2 border-stone-300 bg-stone-100 text-left text-xs font-semibold uppercase tracking-wider text-stone-600">
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
              <MaterialCategoryGroup key={cat} category={cat} materials={items} onRemove={removeMaterial} />
            ))}
            {grouped.size === 0 && (
              <tr>
                <td colSpan={7} className="px-3 py-8 text-center text-sm text-stone-400">
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

function MaterialCategoryGroup({
  category,
  materials,
  onRemove,
}: {
  category: MaterialCategory;
  materials: Material[];
  onRemove: (id: string) => void;
}) {
  const [collapsed, setCollapsed] = useState(true);

  return (
    <>
      <tr
        className="cursor-pointer select-none bg-stone-50 hover:bg-stone-100"
        onClick={() => setCollapsed((v) => !v)}
      >
        <td
          colSpan={7}
          className="px-3 py-2 text-xs font-bold uppercase tracking-wider text-stone-500"
        >
          <span className="mr-1.5 inline-block w-3 text-center text-[10px]">
            {collapsed ? "\u25B6" : "\u25BC"}
          </span>
          {MATERIAL_CATEGORY_LABELS[category]}
          <span className="ml-2 font-normal text-stone-400">({materials.length})</span>
        </td>
      </tr>
      {!collapsed && materials.map((m) => (
        <tr key={m.id} className="group border-b border-stone-100 hover:bg-stone-50/50">
          <td className="px-3 py-2 font-medium text-stone-800">
            {m.name}
            {!m.isBuiltIn && (
              <span className="ml-2 rounded bg-blue-100 px-1.5 py-0.5 text-[10px] font-semibold uppercase text-blue-600">
                Aangepast
              </span>
            )}
          </td>
          <td className="px-3 py-2 text-stone-500">
            {m.brand ?? <span className="text-stone-300">-</span>}
          </td>
          <td className="px-3 py-2 text-right tabular-nums text-stone-700">
            {m.rho !== null ? m.rho : <span className="text-stone-300">-</span>}
          </td>
          <td className="px-3 py-2 text-right tabular-nums text-stone-700">
            {m.lambda !== null ? m.lambda : <span className="text-stone-300">-</span>}
          </td>
          <td className="px-3 py-2 text-right tabular-nums text-stone-700">
            {m.lambdaWet !== null ? m.lambdaWet : <span className="text-stone-300">-</span>}
          </td>
          <td className="px-3 py-2 text-right tabular-nums text-stone-700">
            {m.mu}
          </td>
          <td className="px-3 py-2">
            {!m.isBuiltIn && (
              <button
                type="button"
                onClick={() => onRemove(m.id)}
                className="rounded px-2 py-0.5 text-xs text-red-400 opacity-0 transition-opacity hover:bg-red-50 hover:text-red-600 group-hover:opacity-100"
                title="Verwijderen"
              >
                Verwijder
              </button>
            )}
          </td>
        </tr>
      ))}
    </>
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
      <label className="flex flex-1 flex-col gap-1 text-xs font-medium text-stone-600">
        Naam
        <input
          type="text"
          value={draft.name}
          onChange={(e) => onChange({ name: e.target.value })}
          placeholder="Bijv. PIR 023"
          className="rounded border border-stone-300 px-2 py-1.5 text-sm text-stone-900 focus:border-blue-400 focus:outline-none"
          autoFocus
        />
      </label>
      <label className="flex w-32 flex-col gap-1 text-xs font-medium text-stone-600">
        Categorie
        <select
          value={draft.category}
          onChange={(e) => onChange({ category: e.target.value as MaterialCategory })}
          className="rounded border border-stone-300 px-2 py-1.5 text-sm text-stone-900 focus:border-blue-400 focus:outline-none"
        >
          {MATERIAL_CATEGORY_ORDER.map((cat) => (
            <option key={cat} value={cat}>{MATERIAL_CATEGORY_LABELS[cat]}</option>
          ))}
        </select>
      </label>
      <label className="flex w-28 flex-col gap-1 text-xs font-medium text-stone-600">
        Merk
        <input
          type="text"
          value={draft.brand}
          onChange={(e) => onChange({ brand: e.target.value })}
          placeholder="of leeg"
          className="rounded border border-stone-300 px-2 py-1.5 text-sm text-stone-900 focus:border-blue-400 focus:outline-none"
        />
      </label>
      <label className="flex w-20 flex-col gap-1 text-xs font-medium text-stone-600">
        {"\u03C1"} [kg/m{"\u00B3"}]
        <input
          type="number"
          value={draft.rho}
          onChange={(e) => onChange({ rho: e.target.value })}
          step="any"
          className="rounded border border-stone-300 px-2 py-1.5 text-sm tabular-nums text-stone-900 focus:border-blue-400 focus:outline-none"
        />
      </label>
      <label className="flex w-20 flex-col gap-1 text-xs font-medium text-stone-600">
        {"\u03BB"} [W/mK]
        <input
          type="number"
          value={draft.lambda}
          onChange={(e) => onChange({ lambda: e.target.value })}
          step="any"
          className="rounded border border-stone-300 px-2 py-1.5 text-sm tabular-nums text-stone-900 focus:border-blue-400 focus:outline-none"
        />
      </label>
      <label className="flex w-20 flex-col gap-1 text-xs font-medium text-stone-600">
        {"\u03BB"} nat
        <input
          type="number"
          value={draft.lambdaWet}
          onChange={(e) => onChange({ lambdaWet: e.target.value })}
          step="any"
          className="rounded border border-stone-300 px-2 py-1.5 text-sm tabular-nums text-stone-900 focus:border-blue-400 focus:outline-none"
        />
      </label>
      <label className="flex w-20 flex-col gap-1 text-xs font-medium text-stone-600">
        {"\u03BC"} [-]
        <input
          type="number"
          value={draft.mu}
          onChange={(e) => onChange({ mu: e.target.value })}
          step="any"
          className="rounded border border-stone-300 px-2 py-1.5 text-sm tabular-nums text-stone-900 focus:border-blue-400 focus:outline-none"
        />
      </label>
      <div className="flex gap-2">
        <Button onClick={onSubmit}>Toevoegen</Button>
        <button
          type="button"
          onClick={onCancel}
          className="rounded-md border border-stone-300 px-3 py-1.5 text-sm text-stone-600 hover:bg-stone-100"
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

  if (isEditing) {
    return (
      <tr className="border-b border-stone-100 bg-amber-50/50">
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
    <tr className="group border-b border-stone-100 hover:bg-stone-50/50">
      <td className="px-3 py-2.5 font-medium text-stone-800">
        {entry.name}
        {!entry.isBuiltIn && (
          <span className="ml-2 rounded bg-blue-100 px-1.5 py-0.5 text-[10px] font-semibold uppercase text-blue-600">
            Aangepast
          </span>
        )}
        {modified && (
          <span className="ml-2 rounded bg-amber-100 px-1.5 py-0.5 text-[10px] font-semibold uppercase text-amber-600">
            Gewijzigd
          </span>
        )}
      </td>
      <td className="px-3 py-2.5 text-right tabular-nums text-stone-700">
        {entry.uValue.toFixed(2)}
        <span className="ml-1 text-xs text-stone-400">W/m²K</span>
      </td>
      <td className="px-3 py-2.5 text-stone-600">
        {MATERIAL_TYPE_LABELS[entry.materialType]}
      </td>
      <td className="px-3 py-2.5 text-stone-600">
        {VERTICAL_POSITION_LABELS[entry.verticalPosition]}
      </td>
      <td className="px-3 py-2.5">
        <div className="flex justify-end gap-1 opacity-0 transition-opacity group-hover:opacity-100">
          <button
            type="button"
            onClick={handleStartEdit}
            className="rounded px-2 py-0.5 text-xs text-stone-500 hover:bg-stone-200 hover:text-stone-700"
            title="Bewerken"
          >
            Bewerk
          </button>
          <button
            type="button"
            onClick={onDuplicate}
            className="rounded px-2 py-0.5 text-xs text-stone-500 hover:bg-blue-100 hover:text-blue-700"
            title="Dupliceren"
          >
            Kopieer
          </button>
          {modified && onReset && (
            <button
              type="button"
              onClick={onReset}
              className="rounded px-2 py-0.5 text-xs text-amber-600 hover:bg-amber-50 hover:text-amber-700"
              title="Herstel naar standaardwaarden"
            >
              Herstel
            </button>
          )}
          <button
            type="button"
            onClick={onRemove}
            className="rounded px-2 py-0.5 text-xs text-red-400 hover:bg-red-50 hover:text-red-600"
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
      <label className="flex flex-1 flex-col gap-1 text-xs font-medium text-stone-600">
        Naam
        <input
          type="text"
          value={draft.name}
          onChange={(e) => onChange({ name: e.target.value })}
          placeholder="Bijv. Buitenwand (metselwerk)"
          className="rounded border border-stone-300 px-2 py-1.5 text-sm text-stone-900 focus:border-blue-400 focus:outline-none"
          autoFocus
        />
      </label>
      <label className="flex w-28 flex-col gap-1 text-xs font-medium text-stone-600">
        U-waarde [W/m²K]
        <input
          type="number"
          value={draft.uValue}
          onChange={(e) => onChange({ uValue: Number(e.target.value) || 0 })}
          step="0.01"
          min="0"
          className="rounded border border-stone-300 px-2 py-1.5 text-sm tabular-nums text-stone-900 focus:border-blue-400 focus:outline-none"
        />
      </label>
      <label className="flex w-36 flex-col gap-1 text-xs font-medium text-stone-600">
        Materiaal
        <select
          value={draft.materialType}
          onChange={(e) => onChange({ materialType: e.target.value as MaterialType })}
          className="rounded border border-stone-300 px-2 py-1.5 text-sm text-stone-900 focus:border-blue-400 focus:outline-none"
        >
          {Object.entries(MATERIAL_TYPE_LABELS).map(([k, v]) => (
            <option key={k} value={k}>{v}</option>
          ))}
        </select>
      </label>
      <label className="flex w-28 flex-col gap-1 text-xs font-medium text-stone-600">
        Positie
        <select
          value={draft.verticalPosition}
          onChange={(e) => onChange({ verticalPosition: e.target.value as VerticalPosition })}
          className="rounded border border-stone-300 px-2 py-1.5 text-sm text-stone-900 focus:border-blue-400 focus:outline-none"
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
          className="rounded-md border border-stone-300 px-3 py-1.5 text-sm text-stone-600 hover:bg-stone-100"
        >
          Annuleer
        </button>
      </div>
    </div>
  );
}
