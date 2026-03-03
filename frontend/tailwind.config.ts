import type { Config } from "tailwindcss";

const config: Config = {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  darkMode: ["selector", "[data-theme='dark']"],
  theme: {
    extend: {
      colors: {
        primary: {
          DEFAULT: "#D97706",
          hover: "#EA580C",
          light: "#FFFBEB",
          border: "#F59E0B",
        },
        boundary: {
          exterior: "#3b82f6",
          "exterior-bg": "#dbeafe",
          unheated: "#8b5cf6",
          "unheated-bg": "#ede9fe",
          "adjacent-room": "#22c55e",
          "adjacent-room-bg": "#dcfce7",
          "adjacent-building": "#F59E0B",
          "adjacent-building-bg": "#fef3c7",
          ground: "#92400e",
          "ground-bg": "#fef3c7",
        },
      },
      fontFamily: {
        heading: ['"Space Grotesk"', "system-ui", "sans-serif"],
        sans: ['"Inter"', "system-ui", "sans-serif"],
        mono: ['"JetBrains Mono"', '"Fira Code"', "Consolas", "monospace"],
      },
      fontSize: {
        "2xs": ["0.6875rem", { lineHeight: "1.25" }],  // 11px
        xs: ["0.75rem", { lineHeight: "1.25" }],        // 12px
        sm: ["0.8125rem", { lineHeight: "1.5" }],       // 13px
        base: ["0.875rem", { lineHeight: "1.5" }],      // 14px
        lg: ["1rem", { lineHeight: "1.5" }],             // 16px
        xl: ["1.25rem", { lineHeight: "1.25" }],         // 20px
        "2xl": ["1.5rem", { lineHeight: "1.25" }],       // 24px
        "3xl": ["1.875rem", { lineHeight: "1.25" }],     // 30px
      },
      spacing: {
        sidebar: "220px",
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
      },
    },
  },
  plugins: [],
};

export default config;
