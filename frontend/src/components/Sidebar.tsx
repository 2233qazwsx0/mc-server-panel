import { NavLink, useLocation } from 'react-router-dom';
import { LayoutDashboard, Terminal, Folder, Server, X } from 'lucide-react';
import { clsx } from 'clsx';
import { useServer } from '@/contexts/ServerContext';

interface SidebarProps {
  onClose?: () => void;
}

export function Sidebar({ onClose }: SidebarProps) {
  const { serverStatus, isConnected } = useServer();
  const location = useLocation();

  const navItems = [
    { path: '/', icon: LayoutDashboard, label: '仪表盘' },
    { path: '/terminal', icon: Terminal, label: '控制台' },
    { path: '/files', icon: Folder, label: '文件管理' },
  ];

  return (
    <aside className="w-64 bg-nether-800 border-r border-nether-600 flex flex-col h-full">
      <div className="p-4 border-b border-nether-600">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 bg-gradient-to-br from-mc-green to-rust rounded-lg flex items-center justify-center shadow-mc-glow">
              <Server className="w-6 h-6 text-text-primary" aria-hidden="true" />
            </div>
            <div>
              <h1 className="font-display text-lg font-bold gradient-text-primary">MC 服务器</h1>
            <div className="flex items-center gap-2">
              <div className={clsx(
                'w-2 h-2 rounded-full',
                serverStatus.online ? 'status-dot-online' : 'status-dot-offline'
              )} 
              aria-hidden="true" 
              />
              <span className={clsx(
                'text-xs font-mono',
                serverStatus.online ? 'text-mc-green' : 'text-text-muted'
              )}>
                {serverStatus.online ? '在线' : '离线'}
              </span>
            </div>
            </div>
          </div>
          {onClose && (
            <button
              onClick={onClose}
              className="lg:hidden p-2 text-text-secondary hover:text-text-primary transition-colors duration-200"
              aria-label="Close sidebar"
            >
              <X className="w-5 h-5" aria-hidden="true" />
            </button>
          )}
        </div>
      </div>

      <nav className="flex-1 p-4 space-y-2">
        {navItems.map((item) => (
          <NavLink
            key={item.path}
            to={item.path}
            end={item.path === '/'}
            onClick={() => onClose?.()}
            className={({ isActive }) =>
              clsx(
                'nav-item',
                isActive && 'nav-item-active'
              )
            }
            aria-current={location.pathname === item.path ? 'page' : undefined}
          >
            <item.icon className="w-5 h-5" aria-hidden="true" />
            <span className="font-mono text-sm">{item.label}</span>
          </NavLink>
        ))}
      </nav>

      <div className="p-4 border-t border-nether-600">
        <div className="flex items-center gap-2 text-xs text-text-muted">
          <div className={clsx(
            'w-2 h-2 rounded-full',
            isConnected ? 'status-dot-online' : 'status-dot-offline'
          )} 
          aria-hidden="true" 
          />
          <span className="font-mono">
            {isConnected ? 'WebSocket 已连接' : 'WebSocket 已断开'}
          </span>
        </div>
      </div>
    </aside>
  );
}
