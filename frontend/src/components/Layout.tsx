import { ReactNode } from 'react';
import { Sidebar } from './Sidebar';
import { useServer } from '@/contexts/ServerContext';
import { Play, Square, RotateCcw, Menu, X } from 'lucide-react';
import { useState, useEffect } from 'react';
import { clsx } from 'clsx';

interface LayoutProps {
  children: ReactNode;
}

export function Layout({ children }: LayoutProps) {
  const { serverStatus, startServer, stopServer, restartServer } = useServer();
  const [sidebarOpen, setSidebarOpen] = useState(false);
  
  // Close sidebar on resize to desktop
  useEffect(() => {
    const handleResize = () => {
      if (window.innerWidth >= 1024) {
        setSidebarOpen(false);
      }
    };
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  return (
    <div className="flex h-screen overflow-hidden">
      {/* Mobile Sidebar Overlay */}
      {sidebarOpen && (
        <div 
          className="fixed inset-0 bg-black/60 z-40 lg:hidden backdrop-blur-sm"
          onClick={() => setSidebarOpen(false)}
        />
      )}
      
      {/* Sidebar - mobile slide in */}
      <div className={clsx(
        "fixed lg:static inset-y-0 left-0 z-50 transform transition-transform duration-300 lg:transform-none",
        sidebarOpen ? "translate-x-0" : "-translate-x-full"
      )}>
        <Sidebar onClose={() => setSidebarOpen(false)} />
      </div>
      
      <main className="flex-1 flex flex-col overflow-hidden">
        <header className="h-16 bg-nether-800 border-b border-nether-600 px-4 lg:px-6 flex items-center justify-between">
          <div className="flex items-center gap-4">
            {/* Mobile menu toggle */}
            <button 
              onClick={() => setSidebarOpen(!sidebarOpen)}
              className="lg:hidden p-2 text-text-secondary hover:text-mc-green transition-colors duration-200"
              aria-label={sidebarOpen ? "关闭菜单" : "打开菜单"}
            >
              {sidebarOpen ? <X className="w-5 h-5" /> : <Menu className="w-5 h-5" />}
            </button>
            <h2 className="font-display text-xl text-text-primary">服务器控制</h2>
          </div>
          <div className="flex items-center gap-2 lg:gap-3">
            {!serverStatus.online ? (
              <button
                onClick={startServer}
                className="game-button game-button-primary flex items-center gap-2 hover-lift"
                aria-label="启动 Minecraft 服务器"
              >
                <Play className="w-4 h-4" aria-hidden="true" />
                <span className="font-mono text-sm hidden sm:inline">启动服务器</span>
                <span className="font-mono text-sm sm:hidden">启动</span>
              </button>
            ) : (
              <>
                <button
                  onClick={restartServer}
                  className="game-button flex items-center gap-2 hover-lift"
                  aria-label="重启 Minecraft 服务器"
                >
                  <RotateCcw className="w-4 h-4" aria-hidden="true" />
                  <span className="font-mono text-sm hidden sm:inline">重启</span>
                </button>
                <button
                  onClick={stopServer}
                  className="game-button game-button-danger flex items-center gap-2 hover-lift"
                  aria-label="停止 Minecraft 服务器"
                >
                  <Square className="w-4 h-4" aria-hidden="true" />
                  <span className="font-mono text-sm hidden sm:inline">停止服务器</span>
                  <span className="font-mono text-sm sm:hidden">停止</span>
                </button>
              </>
            )}
          </div>
        </header>
        <div className="flex-1 overflow-auto p-4 lg:p-6">
          {children}
        </div>
      </main>
    </div>
  );
}
