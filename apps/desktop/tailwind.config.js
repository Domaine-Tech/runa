/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{svelte,js,ts}"],
  theme: {
    extend: {
      colors: {
        runa: {
          bg: "rgb(var(--runa-bg) / <alpha-value>)",
          panel: "rgb(var(--runa-panel) / <alpha-value>)",
          text: "rgb(var(--runa-text) / <alpha-value>)",
          muted: "rgb(var(--runa-muted) / <alpha-value>)",
          accent: "rgb(var(--runa-accent) / <alpha-value>)",
          "accent-fg": "rgb(var(--runa-accent-fg) / <alpha-value>)",
          chip: "rgb(var(--runa-chip) / <alpha-value>)",
          chipHover: "rgb(var(--runa-chip-hover) / <alpha-value>)",
          border: "rgb(var(--runa-border) / <alpha-value>)",
          "insert-bg": "rgb(var(--runa-insert-bg) / <alpha-value>)",
          "insert-fg": "rgb(var(--runa-insert-fg) / <alpha-value>)",
          "delete-bg": "rgb(var(--runa-delete-bg) / <alpha-value>)",
          "delete-fg": "rgb(var(--runa-delete-fg) / <alpha-value>)",
          danger: "rgb(var(--runa-danger) / <alpha-value>)",
        },
      },
      spacing: {
        1: "var(--runa-space-1)",
        2: "var(--runa-space-2)",
        3: "var(--runa-space-3)",
        4: "var(--runa-space-4)",
      },
      fontSize: {
        xs: ["var(--runa-text-xs)", { lineHeight: "1rem" }],
        sm: ["var(--runa-text-sm)", { lineHeight: "1.25rem" }],
      },
    },
  },
  plugins: [],
};
