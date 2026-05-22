import { create } from "zustand";

export type ToastType = "success" | "error" | "info";

/** Optional action button rendered inside a toast (e.g. "Log in again"). */
export interface ToastAction {
  label: string;
  onClick: () => void;
}

interface Toast {
  id: string;
  message: string;
  type: ToastType;
  action?: ToastAction;
}

interface ToastStore {
  toasts: Toast[];
  addToast: (
    message: string,
    type: ToastType,
    duration?: number,
    action?: ToastAction,
  ) => void;
  removeToast: (id: string) => void;
}

const DEFAULT_DURATION_MS = 3000;

let nextId = 0;

export const useToastStore = create<ToastStore>()((set) => ({
  toasts: [],

  addToast: (message, type, duration = DEFAULT_DURATION_MS, action) => {
    const id = String(++nextId);
    set((state) => ({
      toasts: [...state.toasts, { id, message, type, action }],
    }));

    setTimeout(() => {
      set((state) => ({
        toasts: state.toasts.filter((t) => t.id !== id),
      }));
    }, duration);
  },

  removeToast: (id) =>
    set((state) => ({
      toasts: state.toasts.filter((t) => t.id !== id),
    })),
}));
