/**
 * Auth hook that works in both web and Tauri environments.
 *
 * Web: queries `GET /api/v1/me` (the backend reads Authentik forward_auth
 * headers and returns the user's profile). Login/logout redirect to the
 * Authentik outpost endpoints.
 *
 * Tauri: returns `isLoggedIn: false` — desktop builds don't talk to the
 * Authentik-backed API.
 */
import { useCallback, useEffect, useState } from "react";

import { fetchAuthProfile, loginRedirect, logoutRedirect } from "../lib/auth";
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

export function useAuth(): AuthState {
  const [state, setState] = useState<AuthState>(NO_AUTH);

  const sync = useCallback(async () => {
    if (isTauri()) {
      setState(NO_AUTH);
      return;
    }

    const profile = await fetchAuthProfile();

    if (profile) {
      setState({
        isLoggedIn: true,
        userName: profile.name || profile.preferred_username || profile.email || null,
        login: () => {},
        logout: logoutRedirect,
      });
    } else {
      setState({
        isLoggedIn: false,
        userName: null,
        login: loginRedirect,
        logout: () => {},
      });
    }
  }, []);

  useEffect(() => {
    sync();
  }, [sync]);

  return state;
}
