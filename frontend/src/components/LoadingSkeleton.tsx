interface SkeletonProps {
  className?: string;
  style?: React.CSSProperties;
}

export function Skeleton({ className = '', style }: SkeletonProps) {
  return (
    <div
      className={`animate-pulse bg-nether-700 rounded ${className}`}
      style={style}
      aria-hidden="true"
    />
  );
}

export function MetricCardSkeleton() {
  return (
    <div className="metric-card">
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <Skeleton className="h-4 w-20 mb-2" />
          <Skeleton className="h-8 w-24 mb-2" />
          <Skeleton className="h-3 w-16" />
        </div>
        <Skeleton className="w-12 h-12 rounded-lg" />
      </div>
      <div className="mt-4">
        <Skeleton className="h-2 w-full rounded-full" />
        <Skeleton className="h-3 w-10 mt-1 ml-auto" />
      </div>
    </div>
  );
}

export function ChartSkeleton() {
  return (
    <div className="chart-container">
      <Skeleton className="h-6 w-40 mb-4" />
      <div className="h-64 flex items-end justify-around gap-2 px-4">
        {[...Array(12)].map((_, i) => (
          <Skeleton
            key={i}
            className="flex-1"
            style={{ height: `${30 + Math.random() * 60}%` }}
          />
        ))}
      </div>
    </div>
  );
}

export function TerminalSkeleton() {
  return (
    <div className="terminal-container flex-1">
      <div className="terminal-header">
        <Skeleton className="w-3 h-3 rounded-full" />
        <Skeleton className="w-3 h-3 rounded-full" />
        <Skeleton className="w-3 h-3 rounded-full" />
        <Skeleton className="h-4 w-32 ml-2" />
      </div>
      <div className="terminal-body space-y-2 p-4">
        {[...Array(8)].map((_, i) => (
          <Skeleton
            key={i}
            className="h-4"
            style={{ width: `${60 + Math.random() * 40}%` }}
          />
        ))}
      </div>
      <div className="terminal-input-wrapper">
        <Skeleton className="w-4 h-4" />
        <Skeleton className="h-4 flex-1" />
      </div>
    </div>
  );
}

export function FileTableSkeleton() {
  return (
    <div className="game-card flex-1 overflow-hidden">
      <div className="border-b border-nether-600 p-4 bg-gradient-to-r from-nether-800 to-nether-900">
        <Skeleton className="h-8 w-full max-w-md" />
      </div>
      <div className="flex-1 overflow-auto p-4">
        <div className="hidden md:block">
          <table className="file-table">
            <thead className="file-table-header">
              <tr>
                <th className="file-table-header-cell"><Skeleton className="h-4 w-16" /></th>
                <th className="file-table-header-cell hidden lg:table-cell"><Skeleton className="h-4 w-12" /></th>
                <th className="file-table-header-cell hidden sm:table-cell"><Skeleton className="h-4 w-20" /></th>
                <th className="file-table-header-cell text-right"><Skeleton className="h-4 w-16 ml-auto" /></th>
              </tr>
            </thead>
            <tbody>
              {[...Array(6)].map((_, i) => (
                <tr key={i} className="file-table-row">
                  <td className="file-table-cell">
                    <div className="flex items-center gap-3">
                      <Skeleton className="w-5 h-5" />
                      <Skeleton className="h-4 w-32" />
                    </div>
                  </td>
                  <td className="file-table-cell hidden lg:table-cell">
                    <Skeleton className="h-4 w-12" />
                  </td>
                  <td className="file-table-cell hidden sm:table-cell">
                    <Skeleton className="h-4 w-24" />
                  </td>
                  <td className="file-table-cell text-right">
                    <Skeleton className="h-8 w-24 ml-auto" />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        <div className="md:hidden space-y-3">
          {[...Array(4)].map((_, i) => (
            <div key={i} className="p-4 bg-nether-800 rounded-lg">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <Skeleton className="w-6 h-6" />
                  <Skeleton className="h-4 w-24" />
                </div>
                <Skeleton className="h-8 w-16" />
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

export function LoadingOverlay({ message = 'Loading…' }: { message?: string }) {
  return (
    <div
      className="absolute inset-0 bg-nether-900/80 backdrop-blur-sm flex items-center justify-center z-50"
      role="status"
      aria-live="polite"
    >
      <div className="flex flex-col items-center gap-4">
        <div className="relative w-12 h-12">
          <div className="absolute inset-0 border-4 border-nether-600 rounded-full" />
          <div className="absolute inset-0 border-4 border-transparent border-t-mc-green rounded-full animate-spin" />
        </div>
        <p className="text-text-secondary font-mono text-sm">{message}</p>
      </div>
    </div>
  );
}
