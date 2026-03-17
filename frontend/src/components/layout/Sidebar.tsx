import { NavLink } from "react-router-dom";
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

/* ─── Nav data with icon components ─── */

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

const NAV_MAIN = [
  { to: "/project", labelKey: "sidebar.project", Icon: IconHome },
  { to: "/rooms", labelKey: "sidebar.rooms", Icon: IconGrid },
  { to: "/constructies", labelKey: "sidebar.constructions", Icon: IconClipboardList },
  { to: "/modeller", labelKey: "sidebar.modeller", Icon: IconCube },
  { to: "/results", labelKey: "sidebar.results", Icon: IconBarChart },
] as const;

const NAV_LIBRARY = [
  { to: "/library", labelKey: "sidebar.library", Icon: IconBook },
  { to: "/rc", labelKey: "sidebar.rcValue", Icon: IconLayers },
  { to: "/materialen", labelKey: "sidebar.materials", Icon: IconSwatches },
] as const;

/* ─── Components ─── */

function NavItem({ to, labelKey, Icon }: { to: string; labelKey: string; Icon: React.ComponentType<{ className?: string }> }) {
  const { t } = useTranslation();
  return (
    <li>
      <NavLink
        to={to}
        className={({ isActive }) =>
          `flex items-center gap-3 rounded px-3 py-2 text-sm transition-colors
          ${
            isActive
              ? "bg-primary font-medium text-white"
              : "text-[#57534E] hover:bg-stone-200 hover:text-deep-forge"
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

/** Shows Projects nav link in web mode. */
function ProjectsNavLink() {
  return (
    <>
      <li className="mx-3 my-3 border-t border-stone-200" />
      <NavItem to="/projects" labelKey="sidebar.projects" Icon={IconFolder} />
    </>
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
  const { t } = useTranslation();
  const isWeb = !isTauri();

  return (
    <aside className="flex w-sidebar shrink-0 flex-col border-r border-stone-200 bg-concrete text-stone-600 overflow-hidden">
      {/* Navigation */}
      <nav className="flex-1 overflow-y-auto px-3 py-4">
        {/* Main section */}
        <p className="px-3 pb-1.5 pt-3 font-mono text-2xs font-medium uppercase tracking-wider text-scaffold-gray">
          {t("sidebar.calculation")}
        </p>
        <ul className="space-y-0.5">
          {NAV_MAIN.map((item) => (
            <NavItem key={item.to} {...item} />
          ))}
          {isWeb && <ProjectsNavLink />}
        </ul>

        {/* Divider */}
        <div className="mx-3 my-3 border-t border-stone-200" />

        {/* Library section */}
        <p className="px-3 pb-1.5 pt-3 font-mono text-2xs font-medium uppercase tracking-wider text-scaffold-gray">
          {t("sidebar.tools")}
        </p>
        <ul className="space-y-0.5">
          {NAV_LIBRARY.map((item) => (
            <NavItem key={item.to} {...item} />
          ))}
        </ul>
      </nav>

      {/* Save status */}
      <SaveStatus />

      {/* Footer */}
      <div className="border-t border-stone-200 px-4 py-3">
        <p className="text-2xs text-scaffold-gray">v0.1.0</p>
      </div>
    </aside>
  );
}
