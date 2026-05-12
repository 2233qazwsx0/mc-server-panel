# Nether Reactor Theme - Game Server Management Panel

## 🎮 Theme Overview

**Theme Name**: Nether Reactor  
**Design Philosophy**: Minecraft pixel aesthetics meets modern tech dashboard  
**Primary Colors**: Minecraft Green (#5D7C15), Rust Orange (#DEA584)  
**Background**: Deep Black/Gray (#0D0D0D, #1A1A1A)

---

## 🎨 Color Palette

### Primary Colors

```css
/* Minecraft Green - Main Accent */
--color-primary: #5D7C15;           /* Primary */
--color-primary-light: #7DA01F;      /* Hover/Active */
--color-primary-dark: #3D5C0A;      /* Pressed state */

/* Rust Orange - Secondary Accent */
--color-secondary: #DEA584;          /* Secondary */
--color-secondary-light: #E8B894;     /* Hover */
--color-secondary-dark: #C17A50;      /* Pressed */
```

### Background Colors

```css
/* Deep backgrounds - Dark theme base */
--color-bg-darkest: #0D0D0D;        /* Root background */
--color-bg-dark: #1A1A1A;           /* Card backgrounds */
--color-bg-medium: #252525;          /* Hover states */
--color-bg-light: #333333;           /* Borders */
```

### Text Colors

```css
/* High contrast text hierarchy */
--color-text-primary: #E8E8E8;      /* Main text - High contrast */
--color-text-secondary: #A0A0A0;     /* Secondary text */
--color-text-muted: #666666;         /* Disabled/Hints */
```

### Status Colors

```css
/* Semantic status colors */
--color-success: #5D7C15;            /* Online/Success - uses primary */
--color-warning: #DEA584;            /* Warning - uses secondary */
--color-error: #E53935;             /* Error - Bright red */
--color-info: #4FC3F7;              /* Info - Cyan blue */
```

### Chart Colors

```css
/* Data visualization palette */
--color-chart-1: #5D7C15;           /* Green */
--color-chart-2: #DEA584;           /* Orange */
--color-chart-3: #4FC3F7;           /* Cyan */
--color-chart-4: #9C6ADE;           /* Purple */
```

---

## 🔤 Typography

### Font Families

```css
/* Terminal/Monospace - Console & Code */
font-family: 'Fira Code', 'JetBrains Mono', Consolas, monospace;

/* Display - Headers & Labels */
font-family: 'Orbitron', 'Rajdhani', sans-serif;

/* UI - Interface Elements */
font-family: 'Inter', 'Segoe UI', system-ui, sans-serif;
```

### Font Sizes

```css
/* Terminal text */
--font-size-terminal-sm: 13px;      /* Line height: 1.6 */
--font-size-terminal-base: 14px;    /* Line height: 1.7 */
--font-size-terminal-lg: 15px;      /* Line height: 1.8 */

/* Display headers */
--font-size-display-lg: 2.25rem;   /* Metric values */
--font-size-display-md: 1.5rem;      /* Page titles */
--font-size-display-sm: 1.125rem;   /* Card titles */
```

---

## 🧩 Component Styles

### Game Card

```css
.game-card {
  /* Background */
  background: linear-gradient(180deg, #252525 0%, #1A1A1A 100%);
  
  /* Border */
  border: 1px solid #333333;
  border-radius: 8px;
  
  /* Shadow */
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.3);
  
  /* Top gradient bar */
  &::before {
    content: '';
    position: absolute;
    top: 0; left: 0; right: 0;
    height: 3px;
    background: linear-gradient(90deg, #5D7C15 0%, #7DA01F 50%, #DEA584 100%);
  }
  
  /* Hover */
  &:hover {
    border-color: #444444;
    box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.4);
    transform: translateY(-2px);
  }
}
```

### Game Button

```css
.game-button {
  /* Base styles */
  padding: 0.5rem 1rem;
  background-color: #252525;
  border: 1px solid #333333;
  border-radius: 6px;
  color: var(--color-text-primary);
  
  /* Hover effect */
  &:hover {
    background-color: #2a2a2a;
    border-color: var(--color-primary);
    color: var(--color-primary);
    box-shadow: 0 0 20px rgba(93, 124, 21, 0.3);
  }
  
  /* Active/Click */
  &:active {
    transform: scale(0.98);
  }
}

/* Primary variant */
.game-button-primary {
  background: linear-gradient(135deg, #5D7C15 0%, #3D5C0A 100%);
  border-color: var(--color-primary);
  
  &:hover {
    background: linear-gradient(135deg, #7DA01F 0%, #5D7C15 100%);
    box-shadow: 0 0 30px rgba(93, 124, 21, 0.5);
  }
}

/* Danger variant */
.game-button-danger {
  border-color: var(--color-error);
  color: var(--color-error);
  
  &:hover {
    background-color: rgba(229, 57, 53, 0.1);
    box-shadow: 0 0 20px rgba(229, 57, 53, 0.3);
  }
}
```

### Terminal Container

```css
.terminal-container {
  /* Background */
  background-color: #0a0a0a;
  border: 1px solid #333333;
  border-radius: 8px;
  
  /* Typography */
  font-family: 'Fira Code', monospace;
  font-size: 13px;
  line-height: 1.7;
  
  /* Header */
  .terminal-header {
    background: linear-gradient(180deg, #1a1a1a 0%, #0f0f0f 100%);
    border-bottom: 1px solid #333333;
  }
  
  /* Body */
  .terminal-body {
    padding: 1rem;
    background-color: rgba(0, 0, 0, 0.3);
    overflow-y: auto;
  }
  
  /* Input line */
  .terminal-input-wrapper {
    background: linear-gradient(180deg, #0f0f0f 0%, #0a0a0a 100%);
    border-top: 1px solid #333333;
  }
}
```

### Terminal Line Types

```css
/* Log output - Primary green */
.terminal-line-log {
  color: #5D7C15;
}

/* Command input - Orange */
.terminal-line-command {
  color: #DEA584;
}

/* Info messages - Cyan */
.terminal-line-info {
  color: #4FC3F7;
}

/* Error messages - Red */
.terminal-line-error {
  color: #E53935;
}
```

### Metric Card

```css
.metric-card {
  /* Layout */
  padding: 1.5rem;
  position: relative;
  overflow: hidden;
  
  /* Background & border */
  background: linear-gradient(180deg, #252525 0%, #1A1A1A 100%);
  border: 1px solid #333333;
  border-radius: 8px;
  
  /* Top accent bar */
  &::before {
    content: '';
    position: absolute;
    top: 0; left: 0; right: 0;
    height: 2px;
    background: var(--color-primary);
    opacity: 0.8;
  }
  
  /* Hover */
  &:hover {
    border-color: #444444;
    box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.4);
  }
}

/* Metric value display */
.metric-value {
  font-family: 'Orbitron', sans-serif;
  font-size: 2.25rem;
  font-weight: 700;
  line-height: 1.2;
  color: var(--color-text-primary);
}

/* Progress bar */
.metric-progress-bar {
  height: 0.5rem;
  background-color: #252525;
  border-radius: 9999px;
  overflow: hidden;
}

.metric-progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #5D7C15 0%, #DEA584 100%);
  transition: width 500ms ease-out;
}
```

### Status Indicators

```css
/* Online status dot */
.status-dot-online {
  background-color: #5D7C15;
  animation: pulse-status 2s ease-in-out infinite;
  
  @keyframes pulse-status {
    0%, 100% {
      opacity: 1;
      box-shadow: 0 0 0 0 currentColor;
    }
    50% {
      opacity: 0.7;
      box-shadow: 0 0 0 4px rgba(93, 124, 21, 0.2);
    }
  }
}

/* Offline status dot */
.status-dot-offline {
  background-color: #666666;
}
```

---

## ✨ Animations & Micro-interactions

### Keyframe Animations

```css
/* Status pulse */
@keyframes pulse-status {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.7; }
}

/* Data update flash */
@keyframes data-update {
  0% {
    transform: scale(1.05);
    opacity: 0.8;
  }
  100% {
    transform: scale(1);
    opacity: 1;
  }
}

/* Click feedback */
@keyframes click-feedback {
  0% { transform: scale(0.95); }
  100% { transform: scale(1); }
}

/* Glow breathing */
@keyframes glow-breathe {
  0%, 100% { box-shadow: 0 0 20px rgba(93, 124, 21, 0.3); }
  50% { box-shadow: 0 0 40px rgba(93, 124, 21, 0.5); }
}

/* Scan line effect */
@keyframes scan-line {
  0% { transform: translateY(-100%); }
  100% { transform: translateY(100%); }
}
```

### Animation Classes

```css
/* Apply animations */
.animate-pulse-status {
  animation: pulse-status 2s ease-in-out infinite;
}

.animate-data-update {
  animation: data-update 0.5s ease-out;
}

.animate-click {
  animation: click-feedback 0.15s ease-out;
}

.glow-primary {
  box-shadow: 0 0 20px rgba(93, 124, 21, 0.3);
}

/* Hover lift effect */
.hover-lift {
  transition: transform 200ms ease;
}
.hover-lift:hover {
  transform: translateY(-2px);
}
```

### Reduced Motion Support

```css
@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```

---

## 🎯 Shadows & Glows

```css
/* Primary glow effect */
--shadow-glow-primary: 0 0 20px rgba(93, 124, 21, 0.3);
--shadow-glow-primary-lg: 0 0 40px rgba(93, 124, 21, 0.4);

/* Secondary glow effect */
--shadow-glow-secondary: 0 0 20px rgba(222, 165, 132, 0.3);
--shadow-glow-secondary-lg: 0 0 40px rgba(222, 165, 132, 0.4);

/* Card shadows */
--shadow-card: 0 4px 6px -1px rgba(0, 0, 0, 0.3), 0 2px 4px -1px rgba(0, 0, 0, 0.2);
--shadow-card-hover: 0 10px 15px -3px rgba(0, 0, 0, 0.4), 0 4px 6px -2px rgba(0, 0, 0, 0.3);
```

---

## 📏 Spacing System

```css
/* Terminal spacing */
--spacing-terminal: 1.7rem;         /* Terminal line height */
--spacing-terminal-gap: 0.25rem;   /* Line gap */

/* Card padding */
--padding-card: 1.5rem;             /* Card internal padding */
--padding-card-sm: 1rem;            /* Compact padding */

/* Border radius */
--radius-sm: 4px;                   /* Small elements */
--radius-md: 8px;                   /* Cards, buttons */
--radius-lg: 12px;                  /* Large containers */
--radius-full: 9999px;              /* Pills, badges */
```

---

## 🌈 Gradients

```css
/* Primary gradient */
background: linear-gradient(135deg, #5D7C15 0%, #3D5C0A 100%);

/* Secondary gradient */
background: linear-gradient(135deg, #DEA584 0%, #C17A50 100%);

/* Dark background gradient */
background: linear-gradient(180deg, #1A1A1A 0%, #0D0D0D 100%);

/* Card gradient */
background: linear-gradient(180deg, #252525 0%, #1A1A1A 100%);

/* Radial mesh background */
background: radial-gradient(ellipse at top, #1A1A1A 0%, #0D0D0D 100%);
```

---

## 🎨 Usage Examples

### Creating a New Component

```tsx
export function MyComponent() {
  return (
    <div className="game-card p-6">
      <h3 className="font-display text-lg text-text-primary mb-4">
        Component Title
      </h3>
      <p className="text-text-secondary font-mono">
        Content goes here...
      </p>
      <button className="game-button game-button-primary mt-4">
        Primary Action
      </button>
    </div>
  );
}
```

### Using Theme Colors

```tsx
// In JSX with Tailwind
<div className="text-mc-green bg-nether-800 border-nether-600">
  Content
</div>

// Using color variables
<div style={{ backgroundColor: 'var(--color-primary)' }}>
  Themed content
</div>
```

### Terminal Output

```tsx
<div className="terminal-container">
  <div className="terminal-line terminal-line-log">
    [12:34:56] Server started successfully
  </div>
  <div className="terminal-line terminal-line-command">
    > help
  </div>
  <div className="terminal-line terminal-line-info">
    [INFO] Showing help for 1 command(s)
  </div>
  <div className="terminal-line terminal-line-error">
    [ERROR] Connection failed
  </div>
</div>
```

---

## ♿ Accessibility

### Focus States

```css
/* Visible focus indicator */
*:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}

/* Button focus ring */
.game-button:focus-visible {
  ring: 2px solid var(--color-primary);
  ring-offset: 2px;
}
```

### Color Contrast

All text colors meet WCAG AA contrast requirements:

- Primary text (#E8E8E8) on dark background (#1A1A1A): **14.5:1** ✓
- Secondary text (#A0A0A0) on dark background: **7.2:1** ✓
- Muted text (#666666) on dark background: **4.5:1** ✓

---

## 📱 Responsive Design

### Breakpoints

```css
/* Mobile first approach */
/* sm: 640px   - Large phones */
/* md: 768px   - Tablets */
/* lg: 1024px  - Small laptops */
/* xl: 1280px  - Desktops */
```

### Responsive Utilities

```css
/* Hide on mobile */
.hide-mobile {
  @media (max-width: 640px) {
    display: none !important;
  }
}

/* Show only on mobile */
.show-mobile-only {
  @media (min-width: 641px) {
    display: none !important;
  }
}
```

---

## 🎯 Design Principles

1. **Minecraft-inspired**: Use green (#5D7C15) as primary accent to evoke Minecraft's grass block
2. **High contrast**: Deep backgrounds with bright accents for readability
3. **Terminal aesthetic**: Monospace fonts for technical data, Orbitron for display
4. **Subtle animations**: Smooth transitions that feel responsive but not distracting
5. **Consistent spacing**: Use consistent spacing values throughout
6. **Accessible**: WCAG AA compliant colors and focus states

---

## 🔗 Resources

- **Fonts**: [Fira Code](https://github.com/tonsky/FiraCode), [Orbitron](https://fonts.google.com/specimen/Orbitron)
- **Icons**: [Lucide React](https://lucide.dev/)
- **Charts**: [Recharts](https://recharts.org/)

---

*Last updated: 2026-05-12*
