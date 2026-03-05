import { NavLink } from "react-router-dom";

import { isTauri } from "../../lib/backend";
import { useOidc } from "../../lib/oidc";

const NAV_ITEMS = [
  { to: "/project", label: "Project", icon: "\u2302" },
  { to: "/rooms", label: "Vertrekken", icon: "\u25A6" },
  { to: "/library", label: "Bibliotheek", icon: "\u25E8" },
  { to: "/results", label: "Resultaten", icon: "\u2261" },
] as const;

function NavItem({ to, label, icon }: { to: string; label: string; icon: string }) {
  return (
    <li>
      <NavLink
        to={to}
        className={({ isActive }) =>
          `flex items-center gap-2.5 rounded-md px-3 py-1.5 text-sm transition-colors
          ${
            isActive
              ? "bg-zinc-800 text-white font-medium"
              : "hover:bg-zinc-800/60 hover:text-white"
          }`
        }
      >
        <span className="text-base">{icon}</span>
        {label}
      </NavLink>
    </li>
  );
}

/** Projecten nav link — only shown when logged in. */
function ProjectsNavLink() {
  const oidc = useOidc();
  if (!oidc.isUserLoggedIn) return null;
  return (
    <>
      <li className="my-2 border-t border-zinc-800" />
      <NavItem to="/projects" label="Projecten" icon="&#128193;" />
    </>
  );
}

/** Auth section in the sidebar footer. */
function AuthSection() {
  const oidc = useOidc();

  if (oidc.isUserLoggedIn) {
    const name =
      oidc.decodedIdToken.name ??
      oidc.decodedIdToken.preferred_username ??
      "Gebruiker";
    return (
      <div className="space-y-2">
        <div className="flex items-center gap-2">
          <div className="flex h-7 w-7 items-center justify-center rounded-full bg-primary/20 text-xs font-bold text-primary">
            {name.charAt(0).toUpperCase()}
          </div>
          <span className="truncate text-xs text-stone-300">{name}</span>
        </div>
        <button
          onClick={() => oidc.logout({ redirectTo: "current page" })}
          className="w-full rounded-md border border-zinc-700 px-2 py-1 text-xs text-stone-400 transition-colors hover:bg-zinc-800 hover:text-white"
        >
          Uitloggen
        </button>
      </div>
    );
  }

  return (
    <button
      onClick={() => oidc.login({ redirectUrl: window.location.href })}
      className="w-full rounded-md bg-primary/20 px-2 py-1.5 text-xs font-medium text-primary transition-colors hover:bg-primary/30"
    >
      Inloggen
    </button>
  );
}

export function Sidebar() {
  const isWeb = !isTauri();

  return (
    <aside className="fixed left-0 top-0 z-30 flex h-screen w-sidebar flex-col bg-zinc-900 text-stone-300">
      {/* Logo / title */}
      <div className="flex h-header items-center gap-2 border-b border-zinc-800 px-4">
        <div
          className="h-6 w-6 rounded"
          style={{ background: "var(--gradient-amber, #D97706)" }}
        />
        <span className="font-heading text-sm font-bold text-white">ISSO 51</span>
      </div>

      {/* Navigation */}
      <nav className="flex-1 overflow-y-auto px-2 py-3">
        <ul className="space-y-0.5">
          {NAV_ITEMS.map((item) => (
            <NavItem key={item.to} {...item} />
          ))}
          {isWeb && <ProjectsNavLink />}
        </ul>
      </nav>

      {/* Footer */}
      <div className="space-y-3 border-t border-zinc-800 px-4 py-3">
        {isWeb && <AuthSection />}
        <p className="text-2xs text-zinc-500">v0.1.0</p>
      </div>
    </aside>
  );
}
