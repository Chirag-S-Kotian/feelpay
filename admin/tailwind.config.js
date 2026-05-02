/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        spotify: {
          black: "#121212",
          surface: "#191414",
          white: "#FFFFFF",
          gray: "#B3B3B3",
          green: "#1DB954",
        },
      },
      fontFamily: {
        sans: ["Inter", "sans-serif"],
      },
      borderRadius: {
        full: "500px",
      },
      spacing: {
        xs: "8px",
        sm: "16px",
        md: "32px",
      },
    },
  },
  plugins: [],
};
