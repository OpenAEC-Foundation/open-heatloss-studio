/**
 * Safe auth hook that works in both web and Tauri environments.
 *
 * Returns auth state without throwing when OIDC is not initialized.
 */
import { useCallback, useEffect, useState } from "react";

import { isTauri } from "../lib/backend";

interface AuthState {
  isLoggedIn: boolean;
  userName: string | null;
  login: () => void;
  logout: () => void;
}

const NO_AUTH: AuthState = {
  isLoggedIn: false,
  userName: null,
  login: () => {},
  logout: () => {},
};

/**
 * Returns auth state. Safe to call anywhere — returns `isLoggedIn: false`
 * in Tauri or when OIDC is not configured.
 */
export function useAuth(): AuthState {
  const [state, setState] = useState<AuthState>(NO_AUTH);

  const sync = useCallback(async () => {
    if (isTauri()) return;

    try {
      const { getOidc } = await import("../lib/oidc");
      const oidc = await getOidc();

      if (oidc.isUserLoggedIn) {
        const decoded = oidc.getDecodedIdToken();
        setState({
          isLoggedIn: true,
          userName: decoded.name ?? decoded.preferred_username ?? null,
          login: () => {},
          logout: () => oidc.logout({ redirectTo: "current page" }),
        });
      } else {
        setState({
          isLoggedIn: false,
          userName: null,
          login: () => oidc.login({ redirectUrl: window.location.href }),
          logout: () => {},
        });
      }
    } catch {
      // OIDC not initialized — stay with NO_AUTH.
    }
  }, []);

  useEffect(() => {
    sync();
  }, [sync]);

  return state;
}
