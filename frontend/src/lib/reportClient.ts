/**
 * API-client voor OpenAEC Reports.
 *
 * Stuurt het OIDC Bearer token van de ingelogde gebruiker mee.
 * De Reports API (of reverse proxy ervoor) valideert de autorisatie.
 * Geen API keys in de frontend — per-user access control via SSO.
 */

const REPORTS_URL = "/api/v1/report/generate";

/**
 * Haal het Bearer token op als de gebruiker is ingelogd.
 * Timeout na 3 seconden om een hangende OIDC call te voorkomen.
 */
async function getBearerToken(): Promise<string | null> {
  try {
    const timeout = new Promise<null>((resolve) => setTimeout(() => resolve(null), 5000));
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

/**
 * Genereer een PDF rapport via de OpenAEC Reports API (v2).
 *
 * @param reportData - BM Reports JSON conform report.schema.json
 * @returns PDF als Blob
 */
export async function generateReportDirect(
  reportData: Record<string, unknown>,
): Promise<Blob> {
  const token = await getBearerToken();

  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };
  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  }

  console.log("[report] POST", REPORTS_URL, token ? "(met token)" : "(zonder token)");

  const res = await fetch(REPORTS_URL, {
    method: "POST",
    headers,
    body: JSON.stringify(reportData),
  });

  console.log("[report] Response:", res.status, res.statusText);

  if (!res.ok) {
    const body = await res.text().catch(() => "");
    let detail: string;
    try {
      const json = JSON.parse(body) as { detail?: string };
      detail = json.detail ?? `Rapport generatie mislukt (${res.status})`;
    } catch {
      detail = body || res.statusText || `HTTP ${res.status}`;
    }
    console.error("[report] Fout response:", res.status, detail);
    throw new Error(detail);
  }

  const contentType = res.headers.get("content-type") || "";
  if (!contentType.includes("application/pdf")) {
    console.error("[report] Onverwacht content-type:", contentType);
    throw new Error("Server retourneerde geen PDF — controleer de backend configuratie.");
  }

  return res.blob();
}
