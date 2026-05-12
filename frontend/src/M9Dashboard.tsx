import React, { useState, useEffect } from 'react';
import { DragDropDashboard, DashboardWidget, DashboardToolbar } from './components/Dashboard/DragDropDashboard';
import { ThemeToggle, LanguageSelector } from './components/Theme/ThemeToggle';
import { NotificationBell, useNotifications } from './components/Notifications/NotificationProvider';
import { CommandPalette, CommandPaletteTrigger } from './components/CommandPalette/CommandPalette';
import { BatchOperationsWizard, BatchItem, BatchOperation } from './components/BatchOperations/BatchOperationsWizard';
import { PerformanceChart, TpsChart, PlayersOnlineChart, NetworkChart } from './components/Charts/Charts';
import { PWAInstallButton, PWANotificationPermission, PWAOfflineIndicator } from './components/PWA/PWAComponents';
import { SkipLink, ReducedMotionProvider } from './components/Accessibility/Accessibility';
import { KeyboardShortcutsHelp, useKeyboardShortcuts } from './components/KeyboardShortcuts/KeyboardShortcuts';

const samplePerformanceData = [
  { time: '00:00', cpu: 35, memory: 45, tps: 20 },
  { time: '04:00', cpu: 28, memory: 42, tps: 20 },
  { time: '08:00', cpu: 55, memory: 58, tps: 19.5 },
  { time: '12:00', cpu: 72, memory: 65, tps: 18.2 },
  { time: '16:00', cpu: 68, memory: 62, tps: 18.8 },
  { time: '20:00', cpu: 45, memory: 52, tps: 19.8 },
];

const sampleTpsData = [
  { time: '00:00', tps: 20 },
  { time: '04:00', tps: 20 },
  { time: '08:00', tps: 19.5 },
  { time: '12:00', tps: 18.2 },
  { time: '16:00', tps: 18.8 },
  { time: '20:00', tps: 19.8 },
];

const samplePlayersData = [
  { time: '00:00', players: 5, maxPlayers: 20 },
  { time: '04:00', players: 2, maxPlayers: 20 },
  { time: '08:00', players: 12, maxPlayers: 20 },
  { time: '12:00', players: 18, maxPlayers: 20 },
  { time: '16:00', players: 15, maxPlayers: 20 },
  { time: '20:00', players: 20, maxPlayers: 20 },
];

const sampleNetworkData = [
  { time: '00:00', bytesIn: 50000, bytesOut: 80000 },
  { time: '04:00', bytesIn: 20000, bytesOut: 30000 },
  { time: '08:00', bytesIn: 150000, bytesOut: 200000 },
  { time: '12:00', bytesIn: 250000, bytesOut: 350000 },
  { time: '16:00', bytesIn: 180000, bytesOut: 250000 },
  { time: '20:00', bytesIn: 220000, bytesOut: 300000 },
];

const defaultWidgets: DashboardWidget[] = [
  {
    id: 'metric-cpu',
    type: 'metric',
    title: 'CPU Usage',
    grid: { x: 0, y: 0, w: 3, h: 2, minW: 2, minH: 2 },
    props: { value: 72, unit: '%', trend: 'up' },
  },
  {
    id: 'metric-memory',
    type: 'metric',
    title: 'Memory Usage',
    grid: { x: 3, y: 0, w: 3, h: 2, minW: 2, minH: 2 },
    props: { value: 65, unit: '%', trend: 'stable' },
  },
  {
    id: 'metric-players',
    type: 'metric',
    title: 'Players Online',
    grid: { x: 6, y: 0, w: 3, h: 2, minW: 2, minH: 2 },
    props: { value: 18, unit: '/ 20', trend: 'up' },
  },
  {
    id: 'metric-tps',
    type: 'metric',
    title: 'Server TPS',
    grid: { x: 9, y: 0, w: 3, h: 2, minW: 2, minH: 2 },
    props: { value: 18.2, unit: '/ 20', trend: 'down' },
  },
  {
    id: 'chart-performance',
    type: 'chart',
    title: 'Performance',
    grid: { x: 0, y: 2, w: 8, h: 4, minW: 4, minH: 3 },
  },
  {
    id: 'chart-network',
    type: 'chart',
    title: 'Network Traffic',
    grid: { x: 8, y: 2, w: 4, h: 4, minW: 3, minH: 3 },
  },
  {
    id: 'chart-players',
    type: 'chart',
    title: 'Player Activity',
    grid: { x: 0, y: 6, w: 6, h: 3, minW: 3, minH: 3 },
  },
  {
    id: 'chart-tps',
    type: 'chart',
    title: 'TPS History',
    grid: { x: 6, y: 6, w: 6, h: 3, minW: 3, minH: 3 },
  },
];

interface M9DashboardProps {
  initialWidgets?: DashboardWidget[];
}

const M9DashboardComponent: React.FC<M9DashboardProps> = ({ initialWidgets }) => {
  const { addNotification } = useNotifications();
  const [widgets, setWidgets] = useState<DashboardWidget[]>(initialWidgets || defaultWidgets);
  const [isEditMode, setIsEditMode] = useState(false);
  const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState(false);
  const [isBatchWizardOpen, setIsBatchWizardOpen] = useState(false);
  const [showShortcutsHelp, setShowShortcutsHelp] = useState(false);

  const [batchItems, setBatchItems] = useState<BatchItem[]>([
    { id: '1', name: 'Server 1', type: 'Server', selected: false },
    { id: '2', name: 'Server 2', type: 'Server', selected: false },
    { id: '3', name: 'Server 3', type: 'Server', selected: false },
  ]);

  const batchOperations: BatchOperation[] = [
    {
      id: 'start',
      name: 'Start Servers',
      description: 'Start all selected servers',
      action: async (items: BatchItem[]) => {
        await new Promise(resolve => setTimeout(resolve, 1000));
        addNotification({
          type: 'success',
          title: 'Servers Started',
          message: `Successfully started ${items.length} servers`,
        });
      },
    },
    {
      id: 'stop',
      name: 'Stop Servers',
      description: 'Stop all selected servers',
      confirmRequired: true,
      action: async (items: BatchItem[]) => {
        await new Promise(resolve => setTimeout(resolve, 1000));
        addNotification({
          type: 'info',
          title: 'Servers Stopped',
          message: `Successfully stopped ${items.length} servers`,
        });
      },
    },
    {
      id: 'backup',
      name: 'Backup Servers',
      description: 'Create backups of selected servers',
      requiresInput: {
        label: 'Backup Name (optional)',
        placeholder: 'Auto-generated',
        type: 'text',
      },
      action: async (items: BatchItem[]) => {
        await new Promise(resolve => setTimeout(resolve, 1500));
        addNotification({
          type: 'success',
          title: 'Backup Complete',
          message: `Successfully backed up ${items.length} servers`,
        });
      },
    },
  ];

  const shortcuts = [
    { key: 'k', ctrl: true, description: 'Open command palette', action: () => setIsCommandPaletteOpen(true) },
    { key: '?', shift: true, description: 'Show keyboard shortcuts', action: () => setShowShortcutsHelp(true) },
    { key: 'e', description: 'Toggle edit mode', action: () => setIsEditMode(!isEditMode) },
  ];

  useKeyboardShortcuts({ shortcuts, enable: true });

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
        e.preventDefault();
        setIsCommandPaletteOpen(true);
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, []);

  const handleCommandExecute = (command: string) => {
    addNotification({
      type: 'info',
      title: 'Command Executed',
      message: command,
    });
  };

  const renderWidgetContent = (widget: DashboardWidget) => {
    switch (widget.id) {
      case 'metric-cpu':
        return (
          <div className="flex flex-col justify-center h-full">
            <div className="text-4xl font-bold text-text-primary mb-2">
              {widget.props?.value || 0}%
            </div>
            <div className="text-sm text-text-secondary mb-2">CPU Usage</div>
            <div className="w-full bg-nether-700 rounded-full h-2">
              <div
                className="bg-mc-green h-2 rounded-full transition-all"
                style={{ width: `${widget.props?.value || 0}%` }}
              />
            </div>
          </div>
        );
      case 'metric-memory':
        return (
          <div className="flex flex-col justify-center h-full">
            <div className="text-4xl font-bold text-text-primary mb-2">
              {widget.props?.value || 0}%
            </div>
            <div className="text-sm text-text-secondary mb-2">Memory Usage</div>
            <div className="w-full bg-nether-700 rounded-full h-2">
              <div
                className="bg-rust h-2 rounded-full transition-all"
                style={{ width: `${widget.props?.value || 0}%` }}
              />
            </div>
          </div>
        );
      case 'metric-players':
        return (
          <div className="flex flex-col justify-center h-full">
            <div className="text-4xl font-bold text-text-primary mb-2">
              {widget.props?.value || 0}
              <span className="text-xl text-text-secondary">{widget.props?.unit || ''}</span>
            </div>
            <div className="text-sm text-text-secondary">Players Online</div>
          </div>
        );
      case 'metric-tps':
        return (
          <div className="flex flex-col justify-center h-full">
            <div className="text-4xl font-bold text-text-primary mb-2">
              {widget.props?.value || 0}
              <span className="text-xl text-text-secondary">/ 20</span>
            </div>
            <div className="text-sm text-text-secondary">Server TPS</div>
            <div className="flex items-center gap-2 mt-2">
              <span className={`w-2 h-2 rounded-full ${(widget.props?.value as number || 0) >= 18 ? 'bg-mc-green' : 'bg-rust'}`} />
              <span className="text-xs text-text-muted">Excellent</span>
            </div>
          </div>
        );
      case 'chart-performance':
        return <PerformanceChart data={samplePerformanceData} height={widget.grid.h * 30 - 60} />;
      case 'chart-network':
        return <NetworkChart data={sampleNetworkData} height={widget.grid.h * 30 - 60} />;
      case 'chart-players':
        return <PlayersOnlineChart data={samplePlayersData} height={widget.grid.h * 30 - 60} />;
      case 'chart-tps':
        return <TpsChart data={sampleTpsData} height={widget.grid.h * 30 - 60} />;
      default:
        return (
          <div className="flex items-center justify-center h-full text-text-muted">
            Widget content placeholder
          </div>
        );
    }
  };

  const updatedWidgets = widgets.map(widget => ({
    ...widget,
    props: widget.props,
  }));

  return (
    <ReducedMotionProvider>
      <div className="min-h-screen bg-nether-900" data-theme="dark">
        <SkipLink />

        <PWAOfflineIndicator />

        <header className="sticky top-0 z-40 bg-nether-800/95 backdrop-blur-sm border-b border-nether-600">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <div className="flex items-center justify-between h-16">
              <div className="flex items-center gap-4">
                <h1 className="text-xl font-bold text-text-primary">M9 Dashboard</h1>
                <DashboardToolbar
                  isEditMode={isEditMode}
                  onEditModeChange={setIsEditMode}
                  onAddWidget={(widget: DashboardWidget) => setWidgets([...widgets, widget])}
                  availableWidgets={defaultWidgets}
                />
              </div>

              <div className="flex items-center gap-2">
                <PWAInstallButton />
                <CommandPaletteTrigger onClick={() => setIsCommandPaletteOpen(true)} />
                <NotificationBell />
                <ThemeToggle variant="dropdown" showLabel />
                <LanguageSelector variant="dropdown" showLabel />
                <button
                  onClick={() => setShowShortcutsHelp(true)}
                  className="p-2 rounded-lg hover:bg-nether-700 transition-colors"
                  aria-label="Keyboard shortcuts"
                >
                  <svg className="w-5 h-5 text-text-secondary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093m0 3h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                </button>
              </div>
            </div>
          </div>
        </header>

        <main id="main-content" className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
          <div className="mb-6">
            <h2 className="text-2xl font-bold text-text-primary mb-2">Dashboard</h2>
            <p className="text-text-secondary">
              Welcome to Minecraft Server Panel
            </p>
          </div>

          <DragDropDashboard
            widgets={updatedWidgets}
            onWidgetsChange={setWidgets}
            isEditMode={isEditMode}
            onEditModeChange={setIsEditMode}
          />

          <div className="mt-8 grid grid-cols-1 md:grid-cols-3 gap-6">
            <div className="bg-nether-800 rounded-lg border border-nether-600 p-6">
              <h3 className="text-lg font-semibold text-text-primary mb-4">Quick Actions</h3>
              <div className="space-y-3">
                <button
                  onClick={() => setIsBatchWizardOpen(true)}
                  className="w-full flex items-center gap-3 p-3 bg-nether-700 hover:bg-nether-600 rounded-lg transition-colors"
                >
                  <svg className="w-5 h-5 text-mc-green" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
                  </svg>
                  <span className="text-text-primary">Batch Operations</span>
                </button>
                <button className="w-full flex items-center gap-3 p-3 bg-nether-700 hover:bg-nether-600 rounded-lg transition-colors">
                  <svg className="w-5 h-5 text-mc-green" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                  </svg>
                  <span className="text-text-primary">Create Backup</span>
                </button>
                <button className="w-full flex items-center gap-3 p-3 bg-nether-700 hover:bg-nether-600 rounded-lg transition-colors">
                  <svg className="w-5 h-5 text-mc-green" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                  </svg>
                  <span className="text-text-primary">Restart Server</span>
                </button>
              </div>
            </div>

            <div className="bg-nether-800 rounded-lg border border-nether-600 p-6">
              <h3 className="text-lg font-semibold text-text-primary mb-4">Accessibility</h3>
              <div className="space-y-3">
                <PWANotificationPermission />
                <div className="flex items-center gap-2 text-sm text-text-secondary">
                  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                  </svg>
                  <span>Reduced motion: Active</span>
                </div>
              </div>
            </div>

            <div className="bg-nether-800 rounded-lg border border-nether-600 p-6">
              <h3 className="text-lg font-semibold text-text-primary mb-4">i18n Support</h3>
              <div className="space-y-2 text-sm text-text-secondary">
                <p>Current language: en</p>
                <p>Supported: English, 中文</p>
                <p>Theme: dark</p>
              </div>
            </div>
          </div>
        </main>

        <CommandPalette
          isOpen={isCommandPaletteOpen}
          onClose={() => setIsCommandPaletteOpen(false)}
          onExecute={handleCommandExecute}
        />

        <BatchOperationsWizard
          isOpen={isBatchWizardOpen}
          onClose={() => setIsBatchWizardOpen(false)}
          items={batchItems}
          onItemsChange={setBatchItems}
          operations={batchOperations}
        />

        <KeyboardShortcutsHelp
          categories={[
            {
              name: 'Global',
              shortcuts: shortcuts,
            },
          ]}
          isOpen={showShortcutsHelp}
          onClose={() => setShowShortcutsHelp(false)}
        />
      </div>
    </ReducedMotionProvider>
  );
};

export const M9Dashboard = M9DashboardComponent;
export { M9Dashboard as default };
