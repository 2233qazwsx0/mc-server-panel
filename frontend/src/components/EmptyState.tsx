import { ReactNode } from 'react';
import { FolderOpen, FileX, Database, Server, WifiOff } from 'lucide-react';
import { clsx } from 'clsx';

interface EmptyStateProps {
  icon?: 'folder' | 'file' | 'database' | 'server' | 'wifi';
  title: string;
  description?: string;
  action?: ReactNode;
  className?: string;
}

export function EmptyState({ icon = 'folder', title, description, action, className }: EmptyStateProps) {
  const icons = {
    folder: FolderOpen,
    file: FileX,
    database: Database,
    server: Server,
    wifi: WifiOff,
  };

  const iconColors = {
    folder: 'text-chart-purple',
    file: 'text-text-muted',
    database: 'text-mc-green',
    server: 'text-rust',
    wifi: 'text-status-error',
  };

  const Icon = icons[icon];

  return (
    <div className={clsx('flex flex-col items-center justify-center py-12 px-4', className)}>
      <div className={clsx(
        'w-20 h-20 rounded-full bg-nether-800 border border-nether-600 flex items-center justify-center mb-6',
        iconColors[icon]
      )}>
        <Icon className="w-10 h-10" aria-hidden="true" />
      </div>
      <h3 className="font-display text-xl text-text-primary mb-2">{title}</h3>
      {description && (
        <p className="text-text-secondary text-center max-w-md mb-6">{description}</p>
      )}
      {action && <div className="mt-2">{action}</div>}
    </div>
  );
}

export function ServerOfflineState({ onStart }: { onStart?: () => void }) {
  return (
    <EmptyState
      icon="server"
      title="服务器离线"
      description="启动 Minecraft 服务器以查看实时指标并管理您的服务器。"
      action={
        onStart && (
          <button
            onClick={onStart}
            className="game-button game-button-primary"
          >
            启动服务器
          </button>
        )
      }
    />
  );
}

export function NoFilesState() {
  return (
    <EmptyState
      icon="file"
      title="未找到文件"
      description="此目录为空或没有可访问的文件。"
    />
  );
}

export function NoPlayersState() {
  return (
    <EmptyState
      icon="database"
      title="没有在线玩家"
      description="当前没有玩家连接到服务器。"
    />
  );
}

export function ConnectionLostState({ onRetry }: { onRetry?: () => void }) {
  return (
    <EmptyState
      icon="wifi"
      title="连接已断开"
      description="无法连接到服务器。请检查您的网络连接并重试。"
      action={
        onRetry && (
          <button
            onClick={onRetry}
            className="game-button game-button-primary"
          >
            重试连接
          </button>
        )
      }
    />
  );
}
