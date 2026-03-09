import { create } from "zustand";
import { persist } from "zustand/middleware";

export type Theme = "dark" | "light" | "blue" | "highContrast";

export const THEME_LABELS: Record<Theme, string> = {
  dark: "Dark",
  light: "Light",
  blue: "Blue",
  highContrast: "High Contrast",
};

const THEME_ORDER: Theme[] = ["dark", "light", "blue", "highContrast"];

interface ThemeState {
  theme: Theme;
  setTheme: (theme: Theme) => void;
  cycleTheme: () => void;
}

/** Apply theme to the DOM. */
function applyTheme(theme: Theme) {
  document.documentElement.setAttribute("data-theme", theme);
}

export const useThemeStore = create<ThemeState>()(
  persist(
    (set, get) => ({
      theme: "dark",
      setTheme: (theme) => {
        applyTheme(theme);
        set({ theme });
      },
      cycleTheme: () => {
        const current = get().theme;
        const idx = THEME_ORDER.indexOf(current);
        const next = THEME_ORDER[(idx + 1) % THEME_ORDER.length];
        applyTheme(next);
        set({ theme: next });
      },
    }),
    {
      name: "isso51-theme",
      onRehydrate: () => {
        return (state) => {
          if (state) applyTheme(state.theme);
        };
      },
    },
  ),
);
