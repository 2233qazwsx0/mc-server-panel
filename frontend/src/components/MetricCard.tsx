import { ReactNode } from 'react';
import { clsx } from 'clsx';

interface MetricCardProps {
  title: string;
  value: string | number;
  icon: ReactNode;
  color: 'cyan' | 'green' | 'yellow' | 'purple' | 'red';
  subtitle?: string;
  progress?: number;
}

const colorClasses = {
  cyan: 'text-chart-cyan border-chart-cyan/30 hover:border-chart-cyan',
  green: 'text-mc-green border-mc-green/30 hover:border-mc-green',
  yellow: 'text-rust border-rust/30 hover:border-rust',
  purple: 'text-chart-purple border-chart-purple/30 hover:border-chart-purple',
  red: 'text-status-error border-status-error/30 hover:border-status-error',
};

const colorBgClasses = {
  cyan: 'rgba(0, 217, 255, 0.1)',
  green: 'rgba(93, 124, 21, 0.1)',
  yellow: 'rgba(222, 165, 132, 0.1)',
  purple: 'rgba(156, 106, 222, 0.1)',
  red: 'rgba(229, 57, 53, 0.1)',
};

const progressColorClasses = {
  cyan: 'bg-chart-cyan',
  green: 'bg-mc-green',
  yellow: 'bg-rust',
  purple: 'bg-chart-purple',
  red: 'bg-status-error',
};

export function MetricCard({ title, value, icon, color, subtitle, progress }: MetricCardProps) {
  return (
    <div className={clsx(
      'metric-card group',
      colorClasses[color]
    )}>
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <p className="metric-title">{title}</p>
          <p className={clsx('metric-value', colorClasses[color])}>
            {value}
          </p>
          {subtitle && (
            <p className="metric-subtitle">{subtitle}</p>
          )}
        </div>
        <div 
          className="metric-icon group-hover:scale-110 transition-transform duration-200"
          style={{ backgroundColor: colorBgClasses[color] }}
        >
          {icon}
        </div>
      </div>
      {progress !== undefined && (
        <div className="metric-progress">
          <div className="metric-progress-bar">
            <div
              className={clsx('metric-progress-fill animate-data-update', progressColorClasses[color])}
              style={{ width: `${Math.min(progress, 100)}%` }}
            />
          </div>
          <p className="metric-progress-label">{progress.toFixed(1)}%</p>
        </div>
      )}
    </div>
  );
}
