/**
 * Extensies-paneel — backstage view voor extensies/plugins.
 *
 * Pattern gespiegeld op OpenAEC-style-book/project-templates/Tauri+React.
 * Mock data voor nu — er is nog geen runtime plugin-loader. De installed-
 * sectie laat de core "extensies" zien die in v0.1 mee-gebundeld zijn
 * (IFC import, PDF report engine, MCP server). Browse-sectie geeft een
 * preview van toekomstige plugins (BAG-import, Vabi-converter, etc.).
 *
 * Toekomst: koppel aan een echte plugin-runtime (WASM modules of Tauri
 * sidecars), met een manifest-schema voor installatie via .zip upload.
 */
import { useState } from "react";
import { useTranslation } from "react-i18next";
import "./ExtensionManagerPanel.css";

interface InstalledExtension {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  category: string;
  enabled: boolean;
  /** Built-in extensions kunnen niet worden uitgeschakeld. */
  builtIn?: boolean;
}

interface CatalogEntry {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  category: string;
}

/** Built-in onderdelen die conceptueel als "extensie" presenteren. */
const SAMPLE_INSTALLED: InstalledExtension[] = [
  {
    id: "isso51-core",
    name: "ISSO 51 Rekenkern",
    version: "0.1.1",
    description:
      "Pure Rust implementatie van ISSO 51:2023 transmissie/ventilatie/infiltratie/opwarm/systeem-verliezen.",
    author: "OpenAEC Foundation",
    category: "Rekenkern",
    enabled: true,
    builtIn: true,
  },
  {
    id: "ifc-importer",
    name: "IFC Importer",
    version: "0.1.0",
    description:
      "Importeer IFC4/IFC4X3 STEP bestanden naar constructie-elementen en ruimtes (via Python sidecar).",
    author: "OpenAEC Foundation",
    category: "Import/Export",
    enabled: true,
    builtIn: true,
  },
  {
    id: "openaec-reports",
    name: "PDF Rapport-engine",
    version: "0.2.0-alpha",
    description:
      "Native Rust PDF generator via openaec-layout (printpdf/lopdf). Liberation Sans embedded fonts.",
    author: "OpenAEC Foundation",
    category: "Rapportage",
    enabled: true,
    builtIn: true,
  },
  {
    id: "mcp-server",
    name: "MCP Server",
    version: "0.1.0",
    description:
      "Model Context Protocol server voor Claude Desktop / Claude Code integratie (calculate · generate_pdf · …).",
    author: "OpenAEC Foundation",
    category: "Integratie",
    enabled: true,
    builtIn: true,
  },
  {
    id: "glaser-engine",
    name: "Glaser dampdiffusie",
    version: "0.1.0",
    description:
      "NEN-EN-ISO 13788 Glaser-methode voor condensatie-analyse op constructie-opbouwen.",
    author: "OpenAEC Foundation",
    category: "Rekenkern",
    enabled: true,
    builtIn: true,
  },
];

/** Roadmap / toekomstige plugins (preview). */
const SAMPLE_CATALOG: CatalogEntry[] = [
  {
    id: "bag-import",
    name: "BAG Adres-import",
    version: "0.1.0",
    description:
      "Postcode + huisnummer → automatische geometrie + bouwjaar via de Basisregistratie Adressen en Gebouwen.",
    author: "Community (roadmap)",
    category: "Import/Export",
  },
  {
    id: "vabi-converter",
    name: "Vabi Elements converter",
    version: "0.1.0",
    description:
      "Lees Vabi Elements projecten in en converteer naar Open Heatloss Studio formaat (.ifcenergy).",
    author: "Community (roadmap)",
    category: "Import/Export",
  },
  {
    id: "isso53",
    name: "ISSO 53 Utiliteit",
    version: "0.1.0",
    description:
      "Uitbreiding voor warmteverliesberekeningen aan utiliteitsgebouwen (kantoren, scholen, ziekenhuizen).",
    author: "OpenAEC Foundation (roadmap)",
    category: "Rekenkern",
  },
  {
    id: "isso57",
    name: "ISSO 57 Vloerverwarming",
    version: "0.1.0",
    description:
      "Detail-berekening voor vloerverwarming-systemen (warmteflux, leiding-spacing, opbouwhoogte).",
    author: "OpenAEC Foundation (roadmap)",
    category: "Rekenkern",
  },
  {
    id: "tenant-branding",
    name: "Huisstijl plugin",
    version: "0.1.0",
    description:
      "Tenant-specifieke PDF-rapport huisstijl (logo, accent-kleur, contact-footer, decoratieve voorbladen).",
    author: "OpenAEC Foundation (roadmap)",
    category: "Rapportage",
  },
];

const CATEGORY_COLORS: Record<string, string> = {
  Rekenkern: "#60a5fa",
  "Import/Export": "#22d3ee",
  Rapportage: "#a78bfa",
  Integratie: "#34d399",
  Utility: "#a1a1aa",
  Other: "#71717a",
};

export default function ExtensionManagerPanel() {
  const { t } = useTranslation("backstage");
  const [tab, setTab] = useState<"installed" | "browse">("installed");
  const [search, setSearch] = useState("");
  const [extensions, setExtensions] = useState(SAMPLE_INSTALLED);

  const toggleExtension = (id: string) => {
    setExtensions((prev) =>
      prev.map((ext) =>
        ext.id === id && !ext.builtIn ? { ...ext, enabled: !ext.enabled } : ext,
      ),
    );
  };

  const matches = (e: { name: string; description: string }) =>
    !search ||
    e.name.toLowerCase().includes(search.toLowerCase()) ||
    e.description.toLowerCase().includes(search.toLowerCase());

  const filteredInstalled = extensions.filter(matches);
  const filteredCatalog = SAMPLE_CATALOG.filter(matches);

  return (
    <div className="ext-manager">
      <h2 className="ext-manager-title">{t("extensions")}</h2>

      <div className="ext-tabs">
        <button
          className={`ext-tab${tab === "installed" ? " active" : ""}`}
          onClick={() => setTab("installed")}
        >
          {t("extInstalled")} ({extensions.length})
        </button>
        <button
          className={`ext-tab${tab === "browse" ? " active" : ""}`}
          onClick={() => setTab("browse")}
        >
          {t("extBrowse")}
        </button>
      </div>

      <div className="ext-search-row">
        <input
          type="text"
          className="ext-search"
          placeholder={t("extSearchPlaceholder")}
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
        <button
          className="ext-upload-btn"
          title={t("extInstallFile")}
          disabled
        >
          + ZIP
        </button>
      </div>

      <div className="ext-list">
        {tab === "installed" &&
          filteredInstalled.map((ext) => (
            <div
              key={ext.id}
              className={`ext-card${ext.enabled ? "" : " disabled"}`}
            >
              <div className="ext-card-header">
                <span
                  className="ext-category-badge"
                  style={{
                    background: CATEGORY_COLORS[ext.category] || "#71717a",
                  }}
                >
                  {ext.category}
                </span>
                <span className="ext-version">v{ext.version}</span>
              </div>
              <div className="ext-card-body">
                <strong className="ext-name">{ext.name}</strong>
                <p className="ext-desc">{ext.description}</p>
                <span className="ext-author">{ext.author}</span>
              </div>
              <div className="ext-card-actions">
                {ext.builtIn ? (
                  <span className="ext-installed-badge" title="Onderdeel van core">
                    core
                  </span>
                ) : (
                  <label className="ext-toggle">
                    <input
                      type="checkbox"
                      checked={ext.enabled}
                      onChange={() => toggleExtension(ext.id)}
                    />
                    <span className="ext-toggle-slider" />
                  </label>
                )}
              </div>
            </div>
          ))}

        {tab === "browse" &&
          filteredCatalog.map((ext) => {
            const isInstalled = extensions.some((e) => e.id === ext.id);
            return (
              <div key={ext.id} className="ext-card">
                <div className="ext-card-header">
                  <span
                    className="ext-category-badge"
                    style={{
                      background: CATEGORY_COLORS[ext.category] || "#71717a",
                    }}
                  >
                    {ext.category}
                  </span>
                  <span className="ext-version">v{ext.version}</span>
                </div>
                <div className="ext-card-body">
                  <strong className="ext-name">{ext.name}</strong>
                  <p className="ext-desc">{ext.description}</p>
                  <span className="ext-author">{ext.author}</span>
                </div>
                <div className="ext-card-actions">
                  {isInstalled ? (
                    <span className="ext-installed-badge">
                      {t("extInstalledBadge")}
                    </span>
                  ) : (
                    <button className="ext-install-btn" disabled>
                      {t("extInstallBtn")}
                    </button>
                  )}
                </div>
              </div>
            );
          })}
      </div>
    </div>
  );
}
