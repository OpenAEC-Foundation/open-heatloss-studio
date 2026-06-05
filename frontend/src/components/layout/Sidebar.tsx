import { useEffect, useMemo, useState } from "react";
import { NavLink, useLocation } from "react-router-dom";
import { useTranslation } from "react-i18next";

import { isTauri } from "../../lib/backend";
import { useProjectStore } from "../../store/projectStore";

/* ─── SVG Icon components (inline, no dependency) ─── */

function IconHome({ className }: { className?: string }) {
  return (
    <svg className={className} width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M3 9l9-7 9 7v11a2 2 0 01-2 2H5a2 2 0 01-2-2z" />
      <polyline points="9 22 9 12 15 12 15 22" />
    </svg>
  );
}

function IconGrid({ className }: { className?: string }) {
  return (
    <svg className={className} width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <rect x="3" y="3" width="7" height="7" />
      <rect x="14" y="3" width="7" height="7" />
      <rect x="3" y="14" width="7" height="7" />
      <rect x="14" y="14" width="7" height="7" />
    </svg>
  );
}

function IconCube({ className }: { className?: string }) {
  return (
    <svg className={className} width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M21 16V8a2 2 0 00-1-1.73l-7-4a2 2 0 00-2 0l-7 4A2 2 0 003 8v8a2 2 0 001 1.73l7 4a2 2 0 002 0l7-4A2 2 0 0021 16z" />
      <polyline points="3.27 6.96 12 12.01 20.73 6.96" />
      <line x1="12" y1="22.08" x2="12" y2="12" />
    </svg>
  );
}

function IconBarChart({ className }: { className?: string }) {
  return (
    <svg className={className} width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <line x1="18" y1="20" x2="18" y2="10" />
      <line x1="12" y1="20" x2="12" y2="4" />
      <line x1="6" y1="20" x2="6" y2="14" />
    </svg>
  );
}

function IconBook({ className }: { className?: string }) {
  return (
    <svg className={className} width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M4 19.5A2.5 2.5 0 016.5 17H20" />
      <path d="M6.5 2H20v20H6.5A2.5 2.5 0 014 19.5v-15A2.5 2.5 0 016.5 2z" />
    </svg>
  );
}

function IconLayers({ className }: { className?: string }) {
  return (
    <svg className={className} width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polygon points="12 2 2 7 12 12 22 7 12 2" />
      <polyline points="2 17 12 22 22 17" />
      <polyline points="2 12 12 17 22 12" />
    </svg>
  );
}

function IconSwatches({ className }: { className?: string }) {
  return (
    <svg className={className} width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <rect x="2" y="2" width="8" height="8" rx="1" />
      <rect x="14" y="2" width="8" height="8" rx="1" />
      <rect x="2" y="14" width="8" height="8" rx="1" />
      <circle cx="18" cy="18" r="4" />
    </svg>
  );
}

function IconFolder({ className }: { className?: string }) {
  return (
    <svg className={className} width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z" />
    </svg>
  );
}

function IconIfc({ className }: { className?: string }) {
  return (
    <svg className={className} width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round">
      <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" />
      <polyline points="14 2 14 8 20 8" />
      <path d="M8.5 17.5l3 1.5 3-1.5v-3l-3-1.5-3 1.5z" />
      <line x1="11.5" y1="19" x2="11.5" y2="16" />
      <line x1="8.5" y1="14.5" x2="11.5" y2="16" />
      <line x1="14.5" y1="14.5" x2="11.5" y2="16" />
    </svg>
  );
}

function IconClipboardList({ className }: { className?: string }) {
  return (
    <svg className={className} width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M16 4h2a2 2 0 012 2v14a2 2 0 01-2 2H6a2 2 0 01-2-2V6a2 2 0 012-2h2" />
      <rect x="8" y="2" width="8" height="4" rx="1" ry="1" />
      <line x1="8" y1="11" x2="16" y2="11" />
      <line x1="8" y1="15" x2="16" y2="15" />
      <line x1="8" y1="19" x2="12" y2="19" />
    </svg>
  );
}

function IconChevron({ className, expanded }: { className?: string; expanded: boolean }) {
  return (
    <svg
      className={`${className ?? ""} transition-transform duration-150 ${expanded ? "rotate-90" : ""}`}
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <polyline points="9 18 15 12 9 6" />
    </svg>
  );
}

/* ─── Types ─── */

type IconComponent = React.ComponentType<{ className?: string }>;

type GroupKey = "project" | "warmteverlies" | "tojuli" | "rcwaarde" | "library";

type NavItemSpec = {
  to: string;
  labelKey: string;
  Icon: IconComponent;
  disabled?: boolean;
  /** i18n key for tooltip when disabled */
  disabledTitleKey?: string;
};

type NavGroupSpec = {
  key: GroupKey;
  titleKey: string;
  /** Default collapsed state (used on first load before localStorage hydrates) */
  defaultCollapsed: boolean;
  items: ReadonlyArray<NavItemSpec>;
};

/* ─── Nav data ─── */

const NAV_GROUPS: ReadonlyArray<NavGroupSpec> = [
  {
    key: "project",
    titleKey: "sidebar.groups.project",
    defaultCollapsed: false,
    items: [
      { to: "/project", labelKey: "sidebar.project", Icon: IconHome },
      { to: "/rooms", labelKey: "sidebar.rooms", Icon: IconGrid },
      { to: "/constructies", labelKey: "sidebar.constructions", Icon: IconClipboardList },
      { to: "/modeller", labelKey: "sidebar.modeller", Icon: IconCube },
      { to: "/ifc", labelKey: "sidebar.ifc", Icon: IconIfc },
    ],
  },
  {
    key: "warmteverlies",
    titleKey: "sidebar.groups.warmteverlies",
    defaultCollapsed: false,
    items: [
      { to: "/warmteverlies/instellingen", labelKey: "sidebar.warmteverliesInstellingen", Icon: IconLayers },
      { to: "/results", labelKey: "sidebar.results", Icon: IconBarChart },
    ],
  },
  {
    key: "tojuli",
    titleKey: "sidebar.groups.tojuli",
    defaultCollapsed: false,
    items: [
      {
        to: "/tojuli",
        labelKey: "sidebar.tojuli.full",
        Icon: IconBarChart,
      },
      {
        to: "/tojuli/quick",
        labelKey: "sidebar.tojuli.quick",
        Icon: IconBarChart,
      },
    ],
  },
  {
    key: "rcwaarde",
    titleKey: "sidebar.groups.rcwaarde",
    defaultCollapsed: false,
    items: [
      { to: "/rc", labelKey: "sidebar.rcValue", Icon: IconLayers },
      { to: "/uw", labelKey: "sidebar.uwValue", Icon: IconLayers },
      {
        to: "/rc-compare",
        labelKey: "sidebar.rcCompare",
        Icon: IconLayers,
      },
    ],
  },
  {
    key: "library",
    titleKey: "sidebar.groups.library",
    defaultCollapsed: true,
    items: [
      { to: "/materialen", labelKey: "sidebar.materials", Icon: IconSwatches },
      { to: "/library", labelKey: "sidebar.library", Icon: IconBook },
    ],
  },
];

/* ─── Persistent collapsed state ─── */

const STORAGE_KEY = "sidebar.groupCollapsed";

type CollapsedMap = Record<GroupKey, boolean>;

function readStoredCollapsed(): Partial<CollapsedMap> {
  if (typeof window === "undefined") return {};
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) return {};
    const parsed = JSON.parse(raw) as unknown;
    if (parsed && typeof parsed === "object") {
      return parsed as Partial<CollapsedMap>;
    }
  } catch {
    // ignore corrupt localStorage
  }
  return {};
}

function writeStoredCollapsed(map: CollapsedMap): void {
  if (typeof window === "undefined") return;
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(map));
  } catch {
    // quota or privacy mode — ignore
  }
}

function useGroupCollapsed(): [CollapsedMap, (key: GroupKey) => void] {
  const [state, setState] = useState<CollapsedMap>(() => {
    const stored = readStoredCollapsed();
    const initial = {} as CollapsedMap;
    for (const group of NAV_GROUPS) {
      initial[group.key] =
        typeof stored[group.key] === "boolean" ? (stored[group.key] as boolean) : group.defaultCollapsed;
    }
    return initial;
  });

  const toggle = (key: GroupKey) => {
    setState((prev) => {
      const next = { ...prev, [key]: !prev[key] };
      writeStoredCollapsed(next);
      return next;
    });
  };

  return [state, toggle];
}

/* ─── Components ─── */

function NavItem({ to, labelKey, Icon }: { to: string; labelKey: string; Icon: IconComponent }) {
  const { t } = useTranslation();
  return (
    <li>
      <NavLink
        to={to}
        className={({ isActive }) =>
          `flex items-center gap-3 rounded px-3 py-2 text-sm transition-colors
          ${
            isActive
              ? "bg-primary font-medium text-on-accent"
              : "text-on-surface-muted hover:bg-[var(--oaec-hover)] hover:text-on-surface"
          }`
        }
      >
        {({ isActive }) => (
          <>
            <Icon className={isActive ? "text-white" : "text-scaffold-gray"} />
            {t(labelKey)}
          </>
        )}
      </NavLink>
    </li>
  );
}

function DisabledNavItem({ labelKey, Icon, titleKey }: { labelKey: string; Icon: IconComponent; titleKey?: string }) {
  const { t } = useTranslation();
  const title = titleKey ? t(titleKey) : undefined;
  return (
    <li>
      <div
        aria-disabled="true"
        title={title}
        className="flex items-center gap-3 rounded px-3 py-2 text-sm text-on-surface-muted opacity-50 cursor-not-allowed select-none"
      >
        <Icon className="text-scaffold-gray" />
        {t(labelKey)}
      </div>
    </li>
  );
}

/** Shows Projects nav link in web mode. */
function ProjectsNavLink() {
  return <NavItem to="/projects" labelKey="sidebar.projects" Icon={IconFolder} />;
}

function NavGroup({
  group,
  expanded,
  onToggle,
}: {
  group: NavGroupSpec;
  expanded: boolean;
  onToggle: () => void;
}) {
  const { t } = useTranslation();
  const regionId = `sidebar-group-${group.key}`;
  return (
    <div className="mb-1">
      <button
        type="button"
        onClick={onToggle}
        aria-expanded={expanded}
        aria-controls={regionId}
        className="flex w-full items-center gap-1.5 px-3 pb-1.5 pt-3 font-mono text-2xs font-medium uppercase tracking-wider text-scaffold-gray hover:text-on-surface transition-colors"
      >
        <IconChevron expanded={expanded} className="text-scaffold-gray" />
        <span>{t(group.titleKey)}</span>
      </button>
      <div
        id={regionId}
        role="region"
        aria-label={t(group.titleKey) ?? undefined}
        className={`overflow-hidden transition-all duration-150 ${expanded ? "max-h-[600px] opacity-100" : "max-h-0 opacity-0"}`}
      >
        <ul className="space-y-0.5">
          {group.items.map((item) =>
            item.disabled ? (
              <DisabledNavItem
                key={`${group.key}-${item.labelKey}`}
                labelKey={item.labelKey}
                Icon={item.Icon}
                titleKey={item.disabledTitleKey}
              />
            ) : (
              <NavItem key={item.to} to={item.to} labelKey={item.labelKey} Icon={item.Icon} />
            ),
          )}
        </ul>
      </div>
    </div>
  );
}

function SaveStatus() {
  const { t } = useTranslation();
  const isDirty = useProjectStore((s) => s.isDirty);
  const activeProjectId = useProjectStore((s) => s.activeProjectId);

  if (!activeProjectId) return null;

  return (
    <div className="flex items-center gap-2 px-3 py-2 text-xs text-scaffold-gray">
      <span
        className={`inline-block h-2 w-2 rounded-full ${isDirty ? "bg-amber-500" : "bg-green-500"}`}
      />
      <span>{isDirty ? t("sidebar.unsaved") : t("sidebar.saved")}</span>
    </div>
  );
}

export function Sidebar() {
  const isWeb = !isTauri();
  const [collapsed, toggle] = useGroupCollapsed();
  const location = useLocation();

  // Auto-expand the group that contains the active route, without persisting the override.
  const effectiveExpanded = useMemo(() => {
    const result = {} as Record<GroupKey, boolean>;
    for (const group of NAV_GROUPS) {
      const groupHasActiveRoute = group.items.some(
        (item) => !item.disabled && item.to !== "" && location.pathname.startsWith(item.to),
      );
      // expanded = !collapsed; auto-expand overrides collapsed when route matches
      result[group.key] = !collapsed[group.key] || groupHasActiveRoute;
    }
    return result;
  }, [collapsed, location.pathname]);

  // Self-heal: if stored map is missing keys (e.g. after adding new groups), persist defaults once.
  useEffect(() => {
    const stored = readStoredCollapsed();
    const missing = NAV_GROUPS.some((g) => typeof stored[g.key] !== "boolean");
    if (missing) {
      const merged = {} as CollapsedMap;
      for (const g of NAV_GROUPS) {
        merged[g.key] =
          typeof stored[g.key] === "boolean" ? (stored[g.key] as boolean) : g.defaultCollapsed;
      }
      writeStoredCollapsed(merged);
    }
  }, []);

  return (
    <aside className="flex w-sidebar shrink-0 flex-col border-r border-[var(--oaec-border-subtle)] bg-surface-alt text-on-surface-secondary overflow-hidden">
      {/* Navigation */}
      <nav className="flex-1 overflow-y-auto px-3 py-4">
        {isWeb && (
          <>
            <ul className="mb-2 space-y-0.5">
              <ProjectsNavLink />
            </ul>
            <div className="mx-3 my-2 border-t border-[var(--oaec-border-subtle)]" />
          </>
        )}
        {NAV_GROUPS.map((group, idx) => (
          <div key={group.key}>
            <NavGroup
              group={group}
              expanded={effectiveExpanded[group.key]}
              onToggle={() => toggle(group.key)}
            />
            {idx < NAV_GROUPS.length - 1 && (
              <div className="mx-3 my-2 border-t border-[var(--oaec-border-subtle)]" />
            )}
          </div>
        ))}
      </nav>

      {/* Save status */}
      <SaveStatus />

      {/* Footer */}
      <div className="border-t border-[var(--oaec-border-subtle)] px-4 py-3">
        <p className="text-2xs text-scaffold-gray">v{__APP_VERSION__}</p>
      </div>
    </aside>
  );
}
