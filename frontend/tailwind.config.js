/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // 主背景色系 - 深色主题
        'nether': {
          900: '#0D0D0D', // 最深背景
          800: '#1A1A1A', // 卡片背景
          700: '#252525', // 悬停状态
          600: '#333333', // 边框
          500: '#444444', // 次要边框
        },
        
        // Minecraft 绿 - 主强调色
        'mc-green': {
          DEFAULT: '#5D7C15',
          light: '#7DA01F',
          dark: '#3D5C0A',
          glow: 'rgba(93, 124, 21, 0.4)',
        },
        
        // Rust 橙 - 次要强调色
        'rust': {
          DEFAULT: '#DEA584',
          light: '#E8B894',
          dark: '#C17A50',
          glow: 'rgba(222, 165, 132, 0.4)',
        },
        
        // 状态色
        'status': {
          success: '#5D7C15',
          warning: '#DEA584',
          error: '#E53935',
          info: '#4FC3F7',
          online: '#5D7C15',
          offline: '#666666',
        },
        
        // 文字色
        'text': {
          primary: '#E8E8E8',
          secondary: '#A0A0A0',
          muted: '#666666',
          inverse: '#0D0D0D',
        },
        
        // 保留一些赛博朋克色用于图表
        'chart': {
          cyan: '#00D9FF',
          purple: '#9C6ADE',
          yellow: '#FFD93D',
        }
      },
      
      fontFamily: {
        // 等宽字体 - 终端和控制台
        'mono': [
          '"Fira Code"',
          '"JetBrains Mono"',
          '"Consolas"',
          'monospace'
        ],
        
        // 显示字体 - 标题和标签
        'display': [
          '"Orbitron"',
          '"Rajdhani"',
          'sans-serif'
        ],
        
        // UI字体 - 界面元素
        'ui': [
          '"Inter"',
          '"Segoe UI"',
          'system-ui',
          'sans-serif'
        ],
      },
      
      fontSize: {
        // 终端字体大小
        'terminal': {
          'sm': ['13px', { lineHeight: '1.6' }],
          'base': ['14px', { lineHeight: '1.7' }],
          'lg': ['15px', { lineHeight: '1.8' }],
        },
      },
      
      spacing: {
        // 终端行高
        'terminal': '1.7rem',
      },
      
      boxShadow: {
        // Minecraft 绿光效果
        'mc-glow': '0 0 20px rgba(93, 124, 21, 0.3)',
        'mc-glow-lg': '0 0 40px rgba(93, 124, 21, 0.4)',
        
        // Rust 橙光效果
        'rust-glow': '0 0 20px rgba(222, 165, 132, 0.3)',
        'rust-glow-lg': '0 0 40px rgba(222, 165, 132, 0.4)',
        
        // 卡片阴影
        'card': '0 4px 6px -1px rgba(0, 0, 0, 0.3), 0 2px 4px -1px rgba(0, 0, 0, 0.2)',
        'card-hover': '0 10px 15px -3px rgba(0, 0, 0, 0.4), 0 4px 6px -2px rgba(0, 0, 0, 0.3)',
        
        // 内发光
        'inner-glow': 'inset 0 0 20px rgba(93, 124, 21, 0.1)',
        'inner-glow-rust': 'inset 0 0 20px rgba(222, 165, 132, 0.1)',
      },
      
      borderRadius: {
        // 游戏风格圆角
        'game': '4px',
        'game-lg': '8px',
      },
      
      animation: {
        // 脉冲动画
        'pulse-mc': 'pulse-mc 2s ease-in-out infinite',
        'pulse-rust': 'pulse-rust 2s ease-in-out infinite',
        
        // 发光呼吸效果
        'glow-breathe': 'glow-breathe 3s ease-in-out infinite',
        
        // 渐变动画
        'gradient-shift': 'gradient-shift 8s ease infinite',
        
        // 扫描线效果
        'scan-line': 'scan-line 4s linear infinite',
        
        // 数据更新动画
        'data-update': 'data-update 0.5s ease-out',
        
        // 按钮点击反馈
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
        // 更平滑的缓动函数
        'bounce-soft': 'cubic-bezier(0.34, 1.56, 0.64, 1)',
        'smooth': 'cubic-bezier(0.4, 0, 0.2, 1)',
      },
      
      backdropBlur: {
        xs: '2px',
      },
      
      // 渐变预设
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
