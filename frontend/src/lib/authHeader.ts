/**
 * Retrieve the OIDC Bearer token for authenticated API calls.
 * Returns null if user is not logged in or OIDC is unavailable.
 * Timeout after 5 seconds to prevent hanging on OIDC init.
 */
export async function getBearerToken(): Promise<string | null> {
  try {
    const timeout = new Promise<null>((resolve) =>
      setTimeout(() => resolve(null), 5000),
    );
    const tokenPromise = (async () => {
      const { getOidc } = await import("./oidc");
      const oidc = await getOidc();
      if (oidc.isUserLoggedIn) {
        return oidc.getAccessToken();
      }
      return null;
    })();
    return await Promise.race([tokenPromise, timeout]);
  } catch {
    return null;
  }
}
