/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        // Custom ContextPaste palette
        cp: {
          bg: "var(--cp-bg)",
          surface: "var(--cp-surface)",
          border: "var(--cp-border)",
          text: "var(--cp-text)",
          muted: "var(--cp-muted)",
          accent: "var(--cp-accent)",
        },
        // Content type badge colors
        badge: {
          url: "#3B82F6",
          json: "#10B981",
          sql: "#8B5CF6",
          shell: "#F59E0B",
          code: "#6366F1",
          aws: "#FF9900",
          credential: "#EF4444",
          markdown: "#64748B",
          html: "#E34F26",
          yaml: "#CB171E",
          email: "#06B6D4",
          ip: "#84CC16",
          filepath: "#78716C",
          connection: "#D946EF",
          plain: "#94A3B8",
        },
      },
      animation: {
        "fade-in": "fadeIn 150ms ease-out",
        "slide-up": "slideUp 150ms ease-out",
        "ghost-flash": "ghostFlash 600ms ease-out",
      },
      keyframes: {
        fadeIn: {
          "0%": { opacity: "0" },
          "100%": { opacity: "1" },
        },
        slideUp: {
          "0%": { opacity: "0", transform: "translateY(8px)" },
          "100%": { opacity: "1", transform: "translateY(0)" },
        },
        ghostFlash: {
          "0%": { backgroundColor: "rgba(34, 197, 94, 0.2)" },
          "100%": { backgroundColor: "transparent" },
        },
      },
    },
  },
  plugins: [],
};
