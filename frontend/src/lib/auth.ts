/**
 * Authentik forward_auth helpers.
 *
 * The browser cookie `authentik_session` is set by Authentik when the user
 * logs in via Caddy. Login/logout happen through the Authentik outpost
 * endpoints — there is no JS-side session state to keep.
 */
import { API_PREFIX } from "./constants";

/** User profile returned by `GET /api/v1/me`. */
export interface AuthProfile {
  id: string;
  email: string;
  name: string;
  preferred_username: string;
  first_seen_at: string;
  last_login_at: string;
}

/**
 * Fetch the current user's profile.
 *
 * Returns `null` when the request returns 401 (not signed in / outside the
 * Caddy forward_auth perimeter — typical for local `vite dev` runs).
 */
export async function fetchAuthProfile(): Promise<AuthProfile | null> {
  try {
    const res = await fetch(`${API_PREFIX}/me`, {
      method: "GET",
      credentials: "include",
      headers: { Accept: "application/json" },
    });

    if (res.status === 401 || res.status === 403) {
      return null;
    }
    if (!res.ok) {
      return null;
    }
    return (await res.json()) as AuthProfile;
  } catch {
    return null;
  }
}

/**
 * Trigger an Authentik login redirect that returns the user to the current
 * URL after authentication.
 *
 * Authentik's outpost listens at `/outpost.goauthentik.io/start` and accepts
 * an `rd` query parameter for the post-login destination.
 */
export function loginRedirect(): void {
  const rd = encodeURIComponent(window.location.href);
  window.location.assign(`/outpost.goauthentik.io/start?rd=${rd}`);
}

/**
 * Trigger an Authentik logout — clears the `authentik_session` cookie and
 * redirects the user to the Authentik logout flow.
 */
export function logoutRedirect(): void {
  window.location.assign("/outpost.goauthentik.io/sign_out");
}
