/**
 * Full-viewport modal for zoomed chart viewing.
 *
 * Renders children (typically an SVG chart) at a larger size.
 * Click backdrop or press Escape to close.
 */
import { useCallback, useEffect } from "react";

interface ChartZoomModalProps {
  open: boolean;
  onClose: () => void;
  title?: string;
  children: React.ReactNode;
}

export function ChartZoomModal({ open, onClose, title, children }: ChartZoomModalProps) {
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    },
    [onClose],
  );

  useEffect(() => {
    if (!open) return;
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [open, handleKeyDown]);

  if (!open) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60"
      onClick={onClose}
    >
      <div
        className="relative max-h-[90vh] w-[90vw] max-w-5xl overflow-auto rounded-lg bg-white p-6 shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        {title && (
          <div className="mb-4 flex items-center justify-between">
            <h3 className="text-sm font-semibold text-stone-700">{title}</h3>
            <button
              onClick={onClose}
              className="rounded px-2 py-1 text-xs text-stone-400 hover:bg-stone-100 hover:text-stone-600"
            >
              Sluiten
            </button>
          </div>
        )}
        {children}
      </div>
    </div>
  );
}
