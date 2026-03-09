import { Component, useEffect, useState, type ReactNode } from "react";
import { NavLink } from "react-router-dom";

import { isTauri } from "../../lib/backend";
import { useProjectStore } from "../../store/projectStore";

const NAV_MAIN = [
  { to: "/project", label: "Project", icon: "\u2302" },
  { to: "/rooms", label: "Vertrekken", icon: "\u25A6" },
  { to: "/modeller", label: "Modeller", icon: "\u2316" },
  { to: "/results", label: "Resultaten", icon: "\u2261" },
] as const;

const NAV_LIBRARY = [
  { to: "/library", label: "Bibliotheek", icon: "\u25E8" },
  { to: "/rc", label: "Rc-waarde", icon: "\u229E" },
  { to: "/materialen", label: "Materialen", icon: "\u25A4" },
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
              ? "bg-app-sidebar-active text-white font-medium"
              : "hover:bg-app-sidebar-hover hover:text-white"
          }`
        }
      >
        <span className="text-base">{icon}</span>
        {label}
      </NavLink>
    </li>
  );
}

/** Catches oidc-spa errors when OIDC is not bootstrapped. */
class OidcGuard extends Component<{ children: ReactNode }, { failed: boolean }> {
  state = { failed: false };
  static getDerivedStateFromError() {
    return { failed: true };
  }
  render() {
    return this.state.failed ? null : this.props.children;
  }
}

/** Lazily loads oidc-spa and shows Projects nav link when logged in. */
function ProjectsNavLink() {
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    import("../../lib/oidc")
      .then(({ getOidc }) => getOidc())
      .then((oidc) => {
        if (oidc.isUserLoggedIn) setVisible(true);
      })
      .catch(() => {});
  }, []);

  if (!visible) return null;
  return (
    <>
      <li className="my-2 border-t border-app-border" />
      <NavItem to="/projects" label="Projecten" icon="&#128193;" />
    </>
  );
}

/** Lazily loads oidc-spa for login/logout buttons. */
function AuthSection() {
  const [oidcState, setOidcState] = useState<{
    isLoggedIn: boolean;
    name?: string;
    login?: () => void;
    logout?: () => void;
  } | null>(null);

  useEffect(() => {
    import("../../lib/oidc")
      .then(({ getOidc }) => getOidc())
      .then((oidc) => {
        if (oidc.isUserLoggedIn) {
          const token = oidc.getDecodedIdToken();
          setOidcState({
            isLoggedIn: true,
            name:
              token.name ??
              token.preferred_username ??
              "Gebruiker",
            logout: () => oidc.logout({ redirectTo: "current page" }),
          });
        } else {
          setOidcState({
            isLoggedIn: false,
            login: () => oidc.login({ redirectUrl: window.location.href }),
          });
        }
      })
      .catch(() => {});
  }, []);

  if (!oidcState) return null;

  if (oidcState.isLoggedIn) {
    return (
      <div className="space-y-2">
        <div className="flex items-center gap-2">
          <div className="flex h-7 w-7 items-center justify-center rounded-full bg-primary/20 text-xs font-bold text-primary">
            {oidcState.name!.charAt(0).toUpperCase()}
          </div>
          <span className="truncate text-xs text-stone-300">{oidcState.name}</span>
        </div>
        <button
          onClick={oidcState.logout}
          className="w-full rounded-md border border-zinc-700 px-2 py-1 text-xs text-stone-400 transition-colors hover:bg-zinc-800 hover:text-white"
        >
          Uitloggen
        </button>
      </div>
    );
  }

  return (
    <button
      onClick={oidcState.login}
      className="w-full rounded-md bg-primary/20 px-2 py-1.5 text-xs font-medium text-primary transition-colors hover:bg-primary/30"
    >
      Inloggen
    </button>
  );
}

function SaveStatus() {
  const isDirty = useProjectStore((s) => s.isDirty);
  const activeProjectId = useProjectStore((s) => s.activeProjectId);

  if (!activeProjectId) return null;

  return (
    <div className="flex items-center gap-2 px-4 py-2 text-xs">
      <span
        className={`inline-block h-2 w-2 rounded-full ${isDirty ? "bg-amber-500" : "bg-green-500"}`}
      />
      <span className="text-stone-400">
        {isDirty ? "Niet opgeslagen" : "Opgeslagen"}
      </span>
    </div>
  );
}

export function Sidebar() {
  const isWeb = !isTauri();
  const hasTitleBar = !isWeb;

  return (
    <aside className={`fixed left-0 z-30 flex w-sidebar flex-col bg-app-sidebar-bg text-app-sidebar-text ${hasTitleBar ? "top-[32px] h-[calc(100vh-32px)]" : "top-0 h-screen"}`}>
      {/* Logo / title */}
      <div className="flex h-header items-center gap-2 border-b border-app-border px-4">
        <div
          className="h-6 w-6 rounded"
          style={{ background: "var(--gradient-amber, #D97706)" }}
        />
        <span className="font-heading text-sm font-bold text-white">ISSO 51</span>
      </div>

      {/* Navigation */}
      <nav className="flex-1 overflow-y-auto px-2 py-3">
        <ul className="space-y-0.5">
          {NAV_MAIN.map((item) => (
            <NavItem key={item.to} {...item} />
          ))}
          {isWeb && (
            <OidcGuard>
              <ProjectsNavLink />
            </OidcGuard>
          )}
          <li className="my-2 border-t border-app-border" />
          {NAV_LIBRARY.map((item) => (
            <NavItem key={item.to} {...item} />
          ))}
        </ul>
      </nav>

      {/* Save status */}
      <SaveStatus />

      {/* Footer */}
      <div className="space-y-3 border-t border-app-border px-4 py-3">
        {isWeb && (
          <OidcGuard>
            <AuthSection />
          </OidcGuard>
        )}
        <p className="text-2xs text-zinc-500">v0.1.0</p>
      </div>
    </aside>
  );
}
