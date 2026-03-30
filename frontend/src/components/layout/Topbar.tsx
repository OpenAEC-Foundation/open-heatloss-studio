import { Component, useEffect, useRef, useState, type ReactNode } from "react";

import { isTauri } from "../../lib/backend";

/** Error boundary: catches oidc-spa errors when OIDC is not bootstrapped. */
class OidcGuard extends Component<{ children: ReactNode }, { failed: boolean }> {
  state = { failed: false };
  static getDerivedStateFromError() {
    return { failed: true };
  }
  render() {
    return this.state.failed ? null : this.props.children;
  }
}

function UserAvatar() {
  const [oidcState, setOidcState] = useState<{
    isLoggedIn: boolean;
    name?: string;
    login?: () => void;
    logout?: () => void;
  } | null>(null);
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

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

  // Close dropdown on outside click
  useEffect(() => {
    function handleClick(e: MouseEvent) {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        setDropdownOpen(false);
      }
    }
    if (dropdownOpen) document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, [dropdownOpen]);

  if (!oidcState) return null;

  if (!oidcState.isLoggedIn) {
    return (
      <button
        onClick={oidcState.login}
        className="rounded-md bg-primary/20 px-3 py-1.5 text-xs font-medium text-primary transition-colors hover:bg-primary/30"
      >
        Inloggen
      </button>
    );
  }

  const initial = oidcState.name!.charAt(0).toUpperCase();

  return (
    <div className="relative" ref={dropdownRef}>
      <button
        onClick={() => setDropdownOpen(!dropdownOpen)}
        className="flex h-8 w-8 items-center justify-center rounded-full border-2 border-primary bg-primary/20 text-xs font-bold text-primary transition-colors hover:bg-primary/30"
        title={oidcState.name}
      >
        {initial}
      </button>
      {dropdownOpen && (
        <div className="absolute right-0 top-full mt-2 w-48 rounded-md border border-[var(--oaec-border)] bg-[var(--oaec-bg-lighter)] py-1 shadow-lg">
          <div className="border-b border-[var(--oaec-border-subtle)] px-3 py-2">
            <p className="truncate text-sm font-medium text-on-surface">
              {oidcState.name}
            </p>
          </div>
          <button
            onClick={oidcState.logout}
            className="w-full px-3 py-2 text-left text-sm text-on-surface-secondary transition-colors hover:bg-[var(--oaec-hover)]"
          >
            Uitloggen
          </button>
        </div>
      )}
    </div>
  );
}

export function Topbar() {
  const isWeb = !isTauri();

  return (
    <header className="fixed left-0 right-0 top-0 z-40 flex h-topbar items-center gap-4 border-b border-[#27272A] bg-deep-forge px-4">
      {/* Left: hamburger + logo + brand */}
      <div className="flex items-center gap-3">
        {/* Hamburger (placeholder for future sidebar collapse) */}
        <button className="flex h-8 w-8 items-center justify-center rounded text-scaffold-gray transition-colors hover:bg-[#27272A] hover:text-white">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
            <path d="M3 6h18M3 12h18M3 18h18" />
          </svg>
        </button>

        {/* Logo */}
        <div className="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-primary">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="white" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M3 21h18M5 21V7l8-4v18M19 21V11l-6-4" />
          </svg>
        </div>

        {/* Brand */}
        <span className="font-heading text-[15px] font-bold whitespace-nowrap">
          <span className="text-white">Open</span>
          <span className="text-primary">AEC</span>
        </span>
      </div>

      {/* Center: search */}
      <div className="mx-auto flex-1" style={{ maxWidth: 320 }}>
        <div className="relative">
          <svg
            className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-scaffold-gray"
            width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round"
          >
            <circle cx="11" cy="11" r="8" />
            <path d="M21 21l-4.35-4.35" />
          </svg>
          <input
            type="text"
            placeholder="Zoeken..."
            className="w-full rounded-lg border-none bg-[#27272A] py-2 pl-9 pr-3 text-sm text-blueprint-white placeholder:text-scaffold-gray focus:outline-none focus:ring-2 focus:ring-primary"
            readOnly
          />
        </div>
      </div>

      {/* Right: icons + avatar */}
      <div className="ml-auto flex items-center gap-4">
        {/* Notification icon */}
        <button className="flex h-8 w-8 items-center justify-center rounded-md text-scaffold-gray transition-colors hover:bg-[#27272A] hover:text-white">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M18 8A6 6 0 006 8c0 7-3 9-3 9h18s-3-2-3-9M13.73 21a2 2 0 01-3.46 0" />
          </svg>
        </button>

        {/* Settings icon */}
        <button className="flex h-8 w-8 items-center justify-center rounded-md text-scaffold-gray transition-colors hover:bg-[#27272A] hover:text-white">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="12" cy="12" r="3" />
            <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z" />
          </svg>
        </button>

        {/* User avatar / login */}
        {isWeb && (
          <OidcGuard>
            <UserAvatar />
          </OidcGuard>
        )}
      </div>
    </header>
  );
}
