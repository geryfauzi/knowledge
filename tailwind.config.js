/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{svelte,js,ts}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        zed: {
          bg: {
            base: '#18181b',
            surface: '#121215',
            elevated: '#202024',
            active: '#27272a',
            hover: '#1e1e22'
          },
          border: {
            subtle: '#222226',
            DEFAULT: '#27272a',
            focused: '#3f3f46'
          },
          text: {
            primary: '#f4f4f5',
            secondary: '#a1a1aa',
            muted: '#71717a'
          },
          accent: {
            DEFAULT: '#38bdf8',
            hover: '#0ea5e9',
            subtle: 'rgba(56, 189, 248, 0.12)'
          }
        }
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', '-apple-system', 'Segoe UI', 'Roboto', 'sans-serif'],
        mono: ['JetBrains Mono', 'Fira Code', 'Consolas', 'monospace']
      }
    }
  },
  plugins: []
};
