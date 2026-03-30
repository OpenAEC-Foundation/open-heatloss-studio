import type { Config } from "tailwindcss";

const config: Config = {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  darkMode: ["selector", "[data-theme='openaec']"],
  theme: {
    extend: {
      colors: {
        /* ═══════════════════════════════════════════════════════
           CSS VAR BRIDGE — Theme-aware Tailwind tokens
           Shell-kleuren die automatisch het thema volgen.
           ═══════════════════════════════════════════════════════ */
        surface:        "var(--theme-bg)",
        "surface-alt":  "var(--theme-bg-lighter)",
        content:        "var(--theme-bg)",
        "on-surface":   "var(--theme-text)",
        "on-surface-secondary": "var(--theme-text-secondary)",
        "on-surface-muted": "var(--theme-text-muted)",
        accent:         "var(--theme-accent)",
        "accent-hover": "var(--theme-accent-hover)",
        "on-accent":    "var(--theme-accent-text)",

        /* ═══════════════════════════════════════════════════════
           DESIGN SYSTEM — Vaste brand kleuren
           ═══════════════════════════════════════════════════════ */
        primary: {
          DEFAULT: "#D97706",
          hover: "#EA580C",
          light: "#FFFBEB",
          border: "#F59E0B",
        },
        concrete: "#F5F5F4",
        "deep-forge": "#36363E",
        "night-build": "#2A2A32",
        "scaffold-gray": "#A1A1AA",
        "signal-orange": "#EA580C",
        "blueprint-white": "#FAFAF9",

        /* ═══════════════════════════════════════════════════════
           DOMAIN — Boundary types (via CSS vars)
           ═══════════════════════════════════════════════════════ */
        boundary: {
          exterior:              "var(--domain-boundary-exterior, #3b82f6)",
          "exterior-bg":         "var(--domain-boundary-exterior-bg, rgba(59, 130, 246, 0.15))",
          unheated:              "var(--domain-boundary-unheated, #8b5cf6)",
          "unheated-bg":         "var(--domain-boundary-unheated-bg, rgba(139, 92, 246, 0.15))",
          "adjacent-room":       "var(--domain-boundary-adjacent-room, #22c55e)",
          "adjacent-room-bg":    "var(--domain-boundary-adjacent-room-bg, rgba(34, 197, 94, 0.15))",
          "adjacent-building":   "var(--domain-boundary-adjacent-building, #F59E0B)",
          "adjacent-building-bg":"var(--domain-boundary-adjacent-building-bg, rgba(245, 158, 11, 0.15))",
          ground:                "var(--domain-boundary-ground, #92400e)",
          "ground-bg":           "var(--domain-boundary-ground-bg, rgba(146, 64, 14, 0.15))",
        },
      },
      fontFamily: {
        heading: ['"Space Grotesk"', "system-ui", "sans-serif"],
        sans: ['"Inter"', "system-ui", "sans-serif"],
        mono: ['"JetBrains Mono"', '"Fira Code"', "Consolas", "monospace"],
      },
      fontSize: {
        "2xs": ["0.6875rem", { lineHeight: "1.25" }],
        xs: ["0.75rem", { lineHeight: "1.25" }],
        sm: ["0.8125rem", { lineHeight: "1.5" }],
        base: ["0.875rem", { lineHeight: "1.5" }],
        lg: ["1rem", { lineHeight: "1.5" }],
        xl: ["1.25rem", { lineHeight: "1.25" }],
        "2xl": ["1.5rem", { lineHeight: "1.25" }],
        "3xl": ["1.875rem", { lineHeight: "1.25" }],
      },
      spacing: {
        sidebar: "260px",
        topbar: "56px",
        header: "48px",
      },
      borderRadius: {
        sm: "0.25rem",
        md: "0.5rem",
        lg: "0.75rem",
        xl: "1rem",
      },
      boxShadow: {
        sm: "0 1px 2px rgba(0, 0, 0, 0.05)",
        md: "0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -2px rgba(0, 0, 0, 0.1)",
        lg: "0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -4px rgba(0, 0, 0, 0.1)",
        dialog: "var(--theme-dialog-shadow)",
        panel: "var(--theme-panel-shadow)",
        popover: "var(--theme-popover-shadow)",
      },
      keyframes: {
        "toast-in": {
          "0%": { opacity: "0", transform: "translateY(8px)" },
          "100%": { opacity: "1", transform: "translateY(0)" },
        },
      },
      animation: {
        "toast-in": "toast-in 0.2s ease-out",
      },
    },
  },
  plugins: [],
};

export default config;
