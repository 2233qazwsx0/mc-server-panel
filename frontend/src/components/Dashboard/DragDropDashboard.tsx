import React, { useState } from 'react';

export interface DashboardWidget {
  id: string;
  type: 'metric' | 'chart' | 'terminal' | 'players' | 'activity' | 'custom';
  title: string;
  i18nKey?: string;
  grid: {
    x: number;
    y: number;
    w: number;
    h: number;
    minW?: number;
    minH?: number;
    maxW?: number;
    maxH?: number;
  };
  props?: Record<string, unknown>;
  actions?: Array<{
    label: string;
    icon?: React.ReactNode;
    onClick: () => void;
  }>;
}

interface DragDropDashboardProps {
  widgets: DashboardWidget[];
  onWidgetsChange: (widgets: DashboardWidget[]) => void;
  isEditMode?: boolean;
  onEditModeChange?: (editMode: boolean) => void;
  className?: string;
}

export const DragDropDashboard: React.FC<DragDropDashboardProps> = ({
  widgets,
  onWidgetsChange,
  isEditMode = false,
  onEditModeChange,
  className = '',
}) => {
  const [draggedWidget, setDraggedWidget] = useState<string | null>(null);

  const handleRemoveWidget = (widgetId: string) => {
    const updatedWidgets = widgets.filter(w => w.id !== widgetId);
    onWidgetsChange(updatedWidgets);
  };

  return (
    <div className={`relative ${className}`}>
      {isEditMode && (
        <div className="mb-4 p-4 bg-mc-green/10 border border-mc-green/30 rounded-lg flex items-center justify-between">
          <div className="flex items-center gap-2">
            <svg
              className="w-5 h-5 text-mc-green"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
              />
            </svg>
            <span className="text-text-primary font-medium">Edit Mode</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-sm text-text-secondary">
              Drag widgets to rearrange
            </span>
            <button
              onClick={() => onEditModeChange?.(false)}
              className="px-4 py-2 bg-mc-green hover:bg-mc-green-light text-white rounded-lg transition-colors text-sm"
            >
              Done
            </button>
          </div>
        </div>
      )}

      <div className="grid grid-cols-12 gap-4">
        {widgets.map(widget => (
          <div
            key={widget.id}
            className={`bg-nether-800 rounded-lg border border-nether-600 overflow-hidden transition-all ${
              isEditMode ? 'ring-2 ring-mc-green/50 cursor-move' : ''
            } ${
              widget.grid.w === 12 ? 'col-span-12' :
              widget.grid.w === 8 ? 'col-span-12 md:col-span-8' :
              widget.grid.w === 6 ? 'col-span-12 md:col-span-6' :
              widget.grid.w === 4 ? 'col-span-12 md:col-span-4' :
              widget.grid.w === 3 ? 'col-span-12 md:col-span-3' :
              'col-span-12'
            }`}
            style={{
              gridRowEnd: `span ${widget.grid.h}`,
              minHeight: `${widget.grid.h * 80}px`
            }}
          >
            {isEditMode && (
              <div className="bg-nether-700 px-3 py-2 flex items-center justify-between border-b border-nether-600">
                <span className="text-sm text-text-secondary">Drag to move</span>
                <button
                  onClick={() => handleRemoveWidget(widget.id)}
                  className="p-1 hover:bg-nether-600 rounded transition-colors"
                  aria-label={`Remove ${widget.title}`}
                >
                  <svg
                    className="w-4 h-4 text-status-error"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M6 18L18 6M6 6l12 12"
                    />
                  </svg>
                </button>
              </div>
            )}
            <div className="h-full p-4">
              <div className="flex items-center justify-between mb-3">
                <h3 className="text-base font-semibold text-text-primary">
                  {widget.title}
                </h3>
                {widget.actions && widget.actions.length > 0 && (
                  <div className="flex items-center gap-2">
                    {widget.actions.map((action, index) => (
                      <button
                        key={index}
                        onClick={action.onClick}
                        className="p-1.5 hover:bg-nether-700 rounded transition-colors"
                        title={action.label}
                      >
                        {action.icon || (
                          <svg
                            className="w-4 h-4 text-text-secondary"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                          >
                            <path
                              strokeLinecap="round"
                              strokeLinejoin="round"
                              strokeWidth={2}
                              d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z"
                            />
                          </svg>
                        )}
                      </button>
                    ))}
                  </div>
                )}
              </div>
              <div className="text-text-secondary text-sm">
                Widget content for &quot;{widget.type}&quot;
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

interface DashboardToolbarProps {
  isEditMode: boolean;
  onEditModeChange: (editMode: boolean) => void;
  onAddWidget: (widget: DashboardWidget) => void;
  availableWidgets?: DashboardWidget[];
  className?: string;
}

export const DashboardToolbar: React.FC<DashboardToolbarProps> = ({
  isEditMode,
  onEditModeChange,
  onAddWidget,
  availableWidgets = [],
  className = '',
}) => {
  const [showWidgetPicker, setShowWidgetPicker] = useState(false);

  const defaultWidgets: DashboardWidget[] = [
    {
      id: 'metric-cpu',
      type: 'metric',
      title: 'CPU Usage',
      grid: { x: 0, y: 0, w: 3, h: 2, minW: 2, minH: 2 },
    },
    {
      id: 'metric-memory',
      type: 'metric',
      title: 'Memory Usage',
      grid: { x: 3, y: 0, w: 3, h: 2, minW: 2, minH: 2 },
    },
    {
      id: 'metric-players',
      type: 'metric',
      title: 'Players Online',
      grid: { x: 6, y: 0, w: 3, h: 2, minW: 2, minH: 2 },
    },
    {
      id: 'metric-tps',
      type: 'metric',
      title: 'Server TPS',
      grid: { x: 9, y: 0, w: 3, h: 2, minW: 2, minH: 2 },
    },
    {
      id: 'chart-performance',
      type: 'chart',
      title: 'Performance Chart',
      grid: { x: 0, y: 2, w: 8, h: 4, minW: 4, minH: 3 },
    },
    {
      id: 'chart-network',
      type: 'chart',
      title: 'Network Traffic',
      grid: { x: 8, y: 2, w: 4, h: 4, minW: 3, minH: 3 },
    },
  ];

  const widgetsToShow = availableWidgets.length > 0 ? availableWidgets : defaultWidgets;

  return (
    <div className={`flex items-center gap-2 ${className}`}>
      <button
        onClick={() => onEditModeChange(!isEditMode)}
        className={`flex items-center gap-2 px-4 py-2 rounded-lg transition-colors ${
          isEditMode
            ? 'bg-mc-green text-white'
            : 'bg-nether-700 hover:bg-nether-600 text-text-primary'
        }`}
      >
        <svg
          className="w-5 h-5"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z"
          />
        </svg>
        {isEditMode ? 'Editing' : 'Edit Layout'}
      </button>

      {isEditMode && (
        <div className="relative">
          <button
            onClick={() => setShowWidgetPicker(!showWidgetPicker)}
            className="flex items-center gap-2 px-4 py-2 bg-nether-700 hover:bg-nether-600 text-text-primary rounded-lg transition-colors"
          >
            <svg
              className="w-5 h-5"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 6v6m0 0v6m0-6h6m-6 0H6"
              />
            </svg>
            Add Widget
          </button>

          {showWidgetPicker && (
            <>
              <div
                className="fixed inset-0 z-10"
                onClick={() => setShowWidgetPicker(false)}
              />
              <div className="absolute right-0 mt-2 w-64 bg-nether-800 border border-nether-600 rounded-lg shadow-xl z-20 overflow-hidden">
                <div className="p-2 max-h-96 overflow-y-auto">
                  {widgetsToShow.map(widget => (
                    <button
                      key={widget.id}
                      onClick={() => {
                        onAddWidget(widget);
                        setShowWidgetPicker(false);
                      }}
                      className="w-full flex items-center gap-3 p-3 hover:bg-nether-700 rounded-lg transition-colors text-left"
                    >
                      <div className="p-2 bg-nether-700 rounded-lg">
                        <svg
                          className="w-4 h-4 text-mc-green"
                          fill="none"
                          stroke="currentColor"
                          viewBox="0 0 24 24"
                        >
                          <path
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth={2}
                            d="M9 17V7m0 10a2 2 0 01-2 2H5a2 2 0 01-2-2V7a2 2 0 012-2h2a2 2 0 012 2m0 10a2 2 0 002 2h2a2 2 0 002-2M9 7a2 2 0 012-2h2a2 2 0 012 2m0 10V7m0 10a2 2 0 002 2h2a2 2 0 002-2V7a2 2 0 00-2-2h-2a2 2 0 00-2 2"
                          />
                        </svg>
                      </div>
                      <div>
                        <div className="text-sm font-medium text-text-primary">
                          {widget.title}
                        </div>
                        <div className="text-xs text-text-muted capitalize">
                          {widget.type}
                        </div>
                      </div>
                    </button>
                  ))}
                </div>
              </div>
            </>
          )}
        </div>
      )}
    </div>
  );
};
