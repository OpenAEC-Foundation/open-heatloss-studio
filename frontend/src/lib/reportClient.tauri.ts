/**
 * Tauri-only report client: generate PDF locally via Rust.
 *
 * Used in desktop builds. Browser builds use reportClient.ts's HTTP fallback.
 */
import { invoke } from "@tauri-apps/api/core";

export async function generateReportTauri(
  reportData: Record<string, unknown>,
): Promise<Blob> {
  // Tauri returns Vec<u8> as number[] over IPC; convert to Uint8Array.
  const result = await invoke<number[]>("generate_report_pdf_bytes", {
    report: reportData,
  });
  const bytes = new Uint8Array(result);
  return new Blob([bytes], { type: "application/pdf" });
}
