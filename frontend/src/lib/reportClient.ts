/**
 * API-client voor OpenAEC Reports.
 *
 * Desktop (Tauri): genereert lokaal via Rust openaec-layout engine.
 * Browser: POST naar warmteverlies-backend `/api/v1/report/generate` proxy.
 *
 * De runtime-detectie kijkt per call (niet gecached) zodat tests of
 * webview-inits die later mounten de juiste branch krijgen.
 */

import { getBearerToken } from "./authHeader";
import { generateReportTauri } from "./reportClient.tauri";

const REPORTS_URL = "/api/v1/report/generate";

function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/**
 * Genereer een PDF rapport.
 *
 * Onder Tauri: lokaal via openaec-layout (geen netwerk-call).
 * Anders: HTTP POST naar de warmteverlies-backend proxy.
 *
 * @param reportData - BM Reports JSON conform report.schema.json
 * @returns PDF als Blob
 */
export async function generateReportDirect(
  reportData: Record<string, unknown>,
): Promise<Blob> {
  if (isTauriRuntime()) {
    if (import.meta.env.DEV) {
      console.log("[report] Tauri runtime — lokale PDF-generatie via Rust");
    }
    return generateReportTauri(reportData);
  }
  return generateReportHttp(reportData);
}

async function generateReportHttp(
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
    console.log(
      "[report] POST",
      REPORTS_URL,
      token ? "(met token)" : "(zonder token)",
    );
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
    throw new Error(
      "Server retourneerde geen PDF — controleer de backend configuratie.",
    );
  }

  return res.blob();
}
