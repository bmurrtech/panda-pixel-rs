/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.rs",
    "./index.html",
  ],
  theme: {
    extend: {
      colors: {
        'bg': '#131415',
        'container-bg': '#131415',
      },
    },
  },
  plugins: [],
}
