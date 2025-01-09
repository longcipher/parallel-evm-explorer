/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/pages/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/components/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/app/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    extend: {
      colors: {
        background: "var(--background)",
        foreground: "var(--foreground)",
      },
      backgroundColor: (theme) => ({
        ...theme("colors"),
        primary: "#3490dc",
        secondary: "#ffed4a",
        cpurple: "#CE93D8",
        cblue: "#90CAF9",
        cgreen: "#008000",
        cgray: '#828585',
      }),
    },
  },
  plugins: [],
};
