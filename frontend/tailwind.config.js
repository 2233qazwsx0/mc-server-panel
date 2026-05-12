/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        'nether': {
          50: '#FAFAFA',
          100: '#F5F5F5',
          200: '#EEEEEE',
          300: '#E0E0E0',
          400: '#BDBDBD',
          500: '#9E9E9E',
          600: '#757575',
          700: '#616161',
          800: '#424242',
          900: '#212121',
          950: '#0D0D0D',
        },
        
        'mc-green': {
          50: '#F0F4E8',
          100: '#E1E8D1',
          200: '#C3D1A3',
          300: '#A5BA75',
          400: '#87A347',
          500: '#5D7C15',
          600: '#4A6310',
          700: '#37480B',
          800: '#242F06',
          900: '#111501',
          light: '#7DA01F',
          dark: '#3D5C0A',
          glow: 'rgba(93, 124, 21, 0.4)',
        },
        
        'rust': {
          50: '#FDF5F0',
          100: '#FAEBDD',
          200: '#F5D7BB',
          300: '#F0C399',
          400: '#EBAF77',
          500: '#DEA584',
          600: '#D4896A',
          700: '#CA6D50',
          800: '#C05136',
          900: '#B6351C',
          light: '#E8B894',
          dark: '#C17A50',
          glow: 'rgba(222, 165, 132, 0.4)',
        },
        
        'status': {
          success: '#5D7C15',
          warning: '#DEA584',
          error: '#E53935',
          info: '#4FC3F7',
          online: '#5D7C15',
          offline: '#666666',
        },
        
        'text': {
          primary: 'var(--text-primary)',
          secondary: 'var(--text-secondary)',
          muted: 'var(--text-muted)',
          inverse: 'var(--text-inverse)',
        },
        
        'chart': {
          cyan: '#00D9FF',
          purple: '#9C6ADE',
          yellow: '#FFD93D',
        }
      },
      
      fontFamily: {
        'mono': [
          '"Fira Code"',
          '"JetBrains Mono"',
          '"Consolas"',
          'monospace'
        ],
        'display': [
          '"Orbitron"',
          '"Rajdhani"',
          'sans-serif'
        ],
        'ui': [
          '"Inter"',
          '"Segoe UI"',
          'system-ui',
          'sans-serif'
        ],
      },
      
      fontSize: {
        'terminal': {
          'sm': ['13px', { lineHeight: '1.6' }],
          'base': ['14px', { lineHeight: '1.7' }],
          'lg': ['15px', { lineHeight: '1.8' }],
        },
      },
      
      spacing: {
        'terminal': '1.7rem',
      },
      
      boxShadow: {
        'mc-glow': '0 0 20px rgba(93, 124, 21, 0.3)',
        'mc-glow-lg': '0 0 40px rgba(93, 124, 21, 0.4)',
        'rust-glow': '0 0 20px rgba(222, 165, 132, 0.3)',
        'rust-glow-lg': '0 0 40px rgba(222, 165, 132, 0.4)',
        'card': '0 4px 6px -1px rgba(0, 0, 0, 0.3), 0 2px 4px -1px rgba(0, 0, 0, 0.2)',
        'card-hover': '0 10px 15px -3px rgba(0, 0, 0, 0.4), 0 4px 6px -2px rgba(0, 0, 0, 0.3)',
        'inner-glow': 'inset 0 0 20px rgba(93, 124, 21, 0.1)',
        'inner-glow-rust': 'inset 0 0 20px rgba(222, 165, 132, 0.1)',
      },
      
      borderRadius: {
        'game': '4px',
        'game-lg': '8px',
      },
      
      animation: {
        'pulse-mc': 'pulse-mc 2s ease-in-out infinite',
        'pulse-rust': 'pulse-rust 2s ease-in-out infinite',
        'glow-breathe': 'glow-breathe 3s ease-in-out infinite',
        'gradient-shift': 'gradient-shift 8s ease infinite',
        'scan-line': 'scan-line 4s linear infinite',
        'data-update': 'data-update 0.5s ease-out',
        'click-feedback': 'click-feedback 0.15s ease-out',
      },
      
      keyframes: {
        'pulse-mc': {
          '0%, 100%': { opacity: '1' },
          '50%': { opacity: '0.7' },
        },
        'pulse-rust': {
          '0%, 100%': { opacity: '1' },
          '50%': { opacity: '0.6' },
        },
        'glow-breathe': {
          '0%, 100%': { 
            boxShadow: '0 0 20px rgba(93, 124, 21, 0.3)' 
          },
          '50%': { 
            boxShadow: '0 0 40px rgba(93, 124, 21, 0.5)' 
          },
        },
        'gradient-shift': {
          '0%, 100%': { backgroundPosition: '0% 50%' },
          '50%': { backgroundPosition: '100% 50%' },
        },
        'scan-line': {
          '0%': { transform: 'translateY(-100%)' },
          '100%': { transform: 'translateY(100%)' },
        },
        'data-update': {
          '0%': { transform: 'scale(1.05)', opacity: '0.8' },
          '100%': { transform: 'scale(1)', opacity: '1' },
        },
        'click-feedback': {
          '0%': { transform: 'scale(0.95)' },
          '100%': { transform: 'scale(1)' },
        },
      },
      
      transitionTimingFunction: {
        'bounce-soft': 'cubic-bezier(0.34, 1.56, 0.64, 1)',
        'smooth': 'cubic-bezier(0.4, 0, 0.2, 1)',
      },
      
      backdropBlur: {
        xs: '2px',
      },
      
      backgroundImage: {
        'gradient-mc': 'linear-gradient(135deg, #5D7C15 0%, #3D5C0A 100%)',
        'gradient-rust': 'linear-gradient(135deg, #DEA584 0%, #C17A50 100%)',
        'gradient-dark': 'linear-gradient(180deg, #1A1A1A 0%, #0D0D0D 100%)',
        'gradient-card': 'linear-gradient(180deg, #252525 0%, #1A1A1A 100%)',
        'gradient-mesh': 'radial-gradient(ellipse at top, #1A1A1A 0%, #0D0D0D 100%)',
      },
    },
  },
  plugins: [],
}
