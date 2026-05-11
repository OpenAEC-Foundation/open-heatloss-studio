/**
 * Cached PDF state for the Rapport-page.
 *
 * Gegenereerde PDF blijft in een Blob URL hangen zodat de page 'm in een
 * iframe kan tonen. De ribbon-tab regenereert bij druk op "Genereren" met
 * de huidige page-size + orientation. Blob URLs worden gerevoked wanneer
 * een nieuwe gegenereerd wordt om memory leaks te voorkomen.
 */
import { create } from "zustand";

export type ReportPageSize = "A4" | "A3";
export type ReportOrientation = "portrait" | "landscape";

interface ReportStore {
  pageSize: ReportPageSize;
  orientation: ReportOrientation;
  /** Blob URL van laatst-gegenereerde PDF, of null als nog niet gegenereerd. */
  pdfBlobUrl: string | null;
  /** Timestamp van laatste generatie (voor cache-busting iframe). */
  generatedAt: number | null;

  setPageSize: (size: ReportPageSize) => void;
  setOrientation: (o: ReportOrientation) => void;
  setPdfBlobUrl: (url: string | null) => void;
  clear: () => void;
}

export const useReportStore = create<ReportStore>((set, get) => ({
  pageSize: "A4",
  orientation: "portrait",
  pdfBlobUrl: null,
  generatedAt: null,
  setPageSize: (pageSize) => set({ pageSize }),
  setOrientation: (orientation) => set({ orientation }),
  setPdfBlobUrl: (url) => {
    const prev = get().pdfBlobUrl;
    if (prev && prev !== url) {
      try { URL.revokeObjectURL(prev); } catch { /* already revoked */ }
    }
    set({ pdfBlobUrl: url, generatedAt: url ? Date.now() : null });
  },
  clear: () => {
    const prev = get().pdfBlobUrl;
    if (prev) {
      try { URL.revokeObjectURL(prev); } catch { /* already revoked */ }
    }
    set({ pdfBlobUrl: null, generatedAt: null });
  },
}));
