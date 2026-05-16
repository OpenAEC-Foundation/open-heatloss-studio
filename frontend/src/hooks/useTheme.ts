import { useCallback, useEffect, useState } from "react";

type Theme = "light" | "openaec" | "openaec-forge";

const STORAGE_KEY = "isso51-theme";
const THEME_CYCLE: Theme[] = ["light", "openaec-forge", "openaec"];

function isTheme(value: string | null): value is Theme {
  return value === "light" || value === "openaec" || value === "openaec-forge";
}

function getInitialTheme(): Theme {
  if (typeof window === "undefined") return "light";
  const stored = localStorage.getItem(STORAGE_KEY);
  if (isTheme(stored)) return stored;
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "openaec" : "light";
}

export function useTheme() {
  const [theme, setThemeState] = useState<Theme>(getInitialTheme);

  useEffect(() => {
    document.documentElement.setAttribute("data-theme", theme);
    localStorage.setItem(STORAGE_KEY, theme);
  }, [theme]);

  const toggle = useCallback(() => {
    setThemeState((prev) => {
      const idx = THEME_CYCLE.indexOf(prev);
      return THEME_CYCLE[(idx + 1) % THEME_CYCLE.length]!;
    });
  }, []);

  return { theme, toggle } as const;
}
