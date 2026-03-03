import { NavLink } from "react-router-dom";

const NAV_ITEMS = [
  { to: "/project", label: "Project", icon: "\u2302" },
  { to: "/results", label: "Resultaten", icon: "\u2261" },
] as const;

export function Sidebar() {
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
            <li key={item.to}>
              <NavLink
                to={item.to}
                className={({ isActive }) =>
                  `flex items-center gap-2.5 rounded-md px-3 py-1.5 text-sm transition-colors
                  ${
                    isActive
                      ? "bg-zinc-800 text-white font-medium"
                      : "hover:bg-zinc-800/60 hover:text-white"
                  }`
                }
              >
                <span className="text-base">{item.icon}</span>
                {item.label}
              </NavLink>
            </li>
          ))}
        </ul>
      </nav>

      {/* Footer */}
      <div className="border-t border-zinc-800 px-4 py-2">
        <p className="text-2xs text-zinc-500">v0.1.0</p>
      </div>
    </aside>
  );
}
