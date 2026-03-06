/**
 * API-client voor OpenAEC Reports.
 *
 * In development gaat het verzoek via de Vite dev proxy die de API key
 * server-side toevoegt. In productie handelt de reverse proxy (nginx/caddy)
 * hetzelfde af. De key komt NOOIT in de client bundle.
 */

/**
 * Genereer een PDF rapport via de OpenAEC Reports API (v2).
 *
 * @param reportData - BM Reports JSON conform report.schema.json
 * @returns PDF als Blob
 */
export async function generateReportDirect(
  reportData: Record<string, unknown>,
): Promise<Blob> {
  const res = await fetch("/api/report/generate", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(reportData),
  });

  if (!res.ok) {
    const err = await res.json().catch(() => ({ detail: res.statusText }));
    throw new Error(
      (err as { detail?: string }).detail ?? `Rapport generatie mislukt (${res.status})`,
    );
  }

  return res.blob();
}
