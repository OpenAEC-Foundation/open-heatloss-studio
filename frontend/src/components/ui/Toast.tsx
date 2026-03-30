import { useToastStore, type ToastType } from "../../store/toastStore";

const TYPE_STYLES: Record<ToastType, string> = {
  success: "bg-[var(--oaec-bg-lighter)] border border-l-[3px] border-[var(--oaec-border)] border-l-green-500 text-on-surface",
  error: "bg-[var(--oaec-bg-lighter)] border border-l-[3px] border-[var(--oaec-border)] border-l-red-500 text-on-surface",
  info: "bg-[var(--oaec-bg-lighter)] border border-l-[3px] border-[var(--oaec-border)] border-l-blue-500 text-on-surface",
};

export function ToastContainer() {
  const toasts = useToastStore((s) => s.toasts);
  const removeToast = useToastStore((s) => s.removeToast);

  if (toasts.length === 0) return null;

  return (
    <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2">
      {toasts.map((toast) => (
        <div
          key={toast.id}
          className={`animate-toast-in flex items-center gap-2 rounded-md px-4 py-2.5 text-sm shadow-lg ${TYPE_STYLES[toast.type]}`}
        >
          <span>{toast.message}</span>
          <button
            onClick={() => removeToast(toast.id)}
            className="ml-2 opacity-70 transition-opacity hover:opacity-100"
            aria-label="Sluiten"
          >
            &times;
          </button>
        </div>
      ))}
    </div>
  );
}
