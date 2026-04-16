/**
 * Auth header helper.
 *
 * After the Authentik forward_auth migration the browser no longer carries a
 * Bearer token — Caddy authenticates the request via the `authentik_session`
 * cookie and forwards the user identity to the backend as `X-Authentik-*`
 * headers. Browser fetches just need to include credentials so the cookie
 * travels with the request.
 *
 * This stub remains so legacy callers (e.g. `backend.ts::importIfcServer`)
 * compile; it always returns `null`.
 */
export async function getBearerToken(): Promise<string | null> {
  return null;
}
