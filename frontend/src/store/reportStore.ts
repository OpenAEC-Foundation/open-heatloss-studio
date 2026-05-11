/**
 * Cached PDF state for the Rapport-page.
 *
 * Gegenereerde PDF blijft in een Blob URL hangen zodat de page 'm in een
 * iframe kan tonen. De ribbon-tab regenereert bij druk op "Genereren" met
 * de huidige page-size + orientation. Blob URLs worden gerevoked wanneer
 * een nieuwe gegenereerd wordt om memory leaks te voorkomen.
 */
import { create } from "zustand";
import { persist } from "zustand/middleware";

export type ReportPageSize = "A4" | "A3";
export type ReportOrientation = "portrait" | "landscape";

/** Toggle-bare secties in het PDF rapport. Default: alles aan. */
export interface ReportSections {
  colofon: boolean;
  toc: boolean;
  uitgangspunten: boolean;
  constructies: boolean;
  vertrekkenOverzicht: boolean;
  perVertrek: boolean;
  diagrammen: boolean;
  gebouwresultaten: boolean;
  backcover: boolean;
}

export const DEFAULT_SECTIONS: ReportSections = {
  colofon: true,
  toc: true,
  uitgangspunten: true,
  constructies: true,
  vertrekkenOverzicht: true,
  perVertrek: true,
  diagrammen: true,
  gebouwresultaten: true,
  backcover: true,
};

interface ReportStore {
  pageSize: ReportPageSize;
  orientation: ReportOrientation;
  sections: ReportSections;
  /** Blob URL van laatst-gegenereerde PDF, of null als nog niet gegenereerd. */
  pdfBlobUrl: string | null;
  /** Timestamp van laatste generatie (voor cache-busting iframe). */
  generatedAt: number | null;

  setPageSize: (size: ReportPageSize) => void;
  setOrientation: (o: ReportOrientation) => void;
  setSection: (key: keyof ReportSections, value: boolean) => void;
  resetSections: () => void;
  setPdfBlobUrl: (url: string | null) => void;
  clear: () => void;
}

export const useReportStore = create<ReportStore>()(
  persist(
    (set, get) => ({
      pageSize: "A4",
      orientation: "portrait",
      sections: DEFAULT_SECTIONS,
      pdfBlobUrl: null,
      generatedAt: null,
      setPageSize: (pageSize) => set({ pageSize }),
      setOrientation: (orientation) => set({ orientation }),
      setSection: (key, value) =>
        set({ sections: { ...get().sections, [key]: value } }),
      resetSections: () => set({ sections: DEFAULT_SECTIONS }),
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
    }),
    {
      name: "ohs-report-options",
      version: 1,
      // Persist alleen de gebruikersinstellingen, niet de blob URL.
      partialize: (state) => ({
        pageSize: state.pageSize,
        orientation: state.orientation,
        sections: state.sections,
      }),
    },
  ),
);
