# Minecraft Server Panel - Frontend

A modern, cyberpunk-themed Minecraft server management panel frontend.

## Tech Stack

- **React 18** - UI framework
- **TypeScript** - Type safety
- **Vite** - Build tool
- **Tailwind CSS** - Styling
- **Recharts** - Charts
- **Lucide React** - Icons
- **React Router** - Navigation

## Features

1. **Dashboard** - Real-time server metrics
   - CPU Usage chart
   - Memory Usage chart
   - TPS (Ticks Per Second)
   - Online Players

2. **Console** - Terminal interface
   - Black/Green terminal styling
   - Auto-scroll to bottom
   - Command input

3. **File Manager** - File browsing
   - Directory navigation
   - File operations (download, edit, delete)

## Getting Started

### Installation

```bash
cd frontend
npm install
```

### Development

```bash
npm run dev
```

The app will be available at `http://localhost:3000`

### Build

```bash
npm run build
```

## Project Structure

```
frontend/
├── src/
│   ├── components/       # Reusable UI components
│   │   ├── Layout.tsx    # Main layout with sidebar
│   │   ├── Sidebar.tsx   # Navigation sidebar
│   │   ├── MetricCard.tsx # Metric display card
│   │   └── LineChart.tsx # Area chart component
│   ├── pages/           # Page components
│   │   ├── Dashboard.tsx
│   │   ├── Terminal.tsx
│   │   └── Files.tsx
│   ├── hooks/           # Custom hooks
│   │   ├── useWebSocket.ts
│   │   └── useServerStatus.ts
│   ├── contexts/        # React contexts
│   │   └── ServerContext.tsx
│   ├── types/           # TypeScript types
│   │   └── index.ts
│   ├── App.tsx
│   ├── main.tsx
│   └── index.css        # Global styles & Tailwind config
├── index.html
├── package.json
├── vite.config.ts
├── tsconfig.json
└── tailwind.config.js
```
