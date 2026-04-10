/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.rs", "./style/**/*.css"],
  theme: {
    extend: {
      colors: {
        brand: {
          50: "#effdf5",
          100: "#d8f8e6",
          500: "#24b675",
          700: "#177852",
          900: "#0f5038"
        }
      },
      boxShadow: {
        panel: "0 8px 26px -18px rgba(14, 116, 82, 0.55)"
      }
    }
  },
  plugins: []
};
