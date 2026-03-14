/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'm-bg': '#0C0C12',     
        'm-surface': '#161620', 
        'm-accent': '#8B5CF6',  
        'm-accent2': '#EC4899', 
        'm-text': '#DCDCEB',    
      }
    },
  },
  plugins: [],
}