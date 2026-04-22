/**
 * API-client voor OpenAEC Reports.
 *
 * Stuurt het OIDC Bearer token van de ingelogde gebruiker mee.
 * POST naar warmteverlies-backend `/api/v1/report/generate`, die de request
 * doorproxyt naar report-rs met een Authentik service-token (svc-warmteverlies)
 * + `X-Original-Tenant` header. Tenant-templating gebeurt in report-rs o.b.v.
 * die header. Geen API keys in de frontend — per-user access control via SSO.
 */

import { getBearerToken } from "./authHeader";

const REPORTS_URL = "/api/v1/report/generate";

/**
 * Genereer een PDF rapport via de warmteverlies backend proxy.
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

  if (import.meta.env.DEV) {
    console.log("[report] POST", REPORTS_URL, token ? "(met token)" : "(zonder token)");
  }

  const res = await fetch(REPORTS_URL, {
    method: "POST",
    headers,
    body: JSON.stringify(reportData),
  });

  if (import.meta.env.DEV) {
    console.log("[report] Response:", res.status, res.statusText);
  }

  if (!res.ok) {
    const errorBody = await res.text().catch(() => "");
    let detail: string;
    try {
      const json = JSON.parse(errorBody) as { detail?: string };
      detail = json.detail ?? `Rapport generatie mislukt (${res.status})`;
    } catch {
      detail = errorBody || res.statusText || `HTTP ${res.status}`;
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
