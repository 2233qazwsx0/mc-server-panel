import { useState, useEffect } from 'react';
import { useServer } from '@/contexts/ServerContext';
import { MetricCard } from '@/components/MetricCard';
import { LineChart } from '@/components/LineChart';
import { Cpu, HardDrive, Activity, Users, Zap } from 'lucide-react';
import { MetricCardSkeleton, ChartSkeleton } from '@/components/LoadingSkeleton';
import { ServerOfflineState } from '@/components/EmptyState';

export function Dashboard() {
  const { serverStatus, cpuMetrics, memoryMetrics, setServerStatus, startServer, isConnected } = useServer();
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const timer = setTimeout(() => {
      setIsLoading(false);
    }, 1500);
    return () => clearTimeout(timer);
  }, []);

  const handleToggle = () => {
    setServerStatus(prev => ({
      ...prev,
      online: !prev.online,
      cpu: !prev.online ? 45 : 0,
      memory: !prev.online ? 60 : 0,
      tps: !prev.online ? 20 : 0,
      players: !prev.online ? 12 : 0,
    }));
  };

  if (isLoading) {
    return (
      <div className="space-y-4 md:space-y-6">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
          <div>
            <h1 className="font-display text-2xl md:text-3xl font-bold text-text-primary">仪表盘</h1>
            <p className="text-text-secondary font-mono mt-1">实时服务器监控</p>
          </div>
        </div>
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 md:gap-6">
          {[...Array(4)].map((_, i) => (
            <MetricCardSkeleton key={i} />
          ))}
        </div>
        <div className="grid grid-cols-1 xl:grid-cols-2 gap-4 md:gap-6">
          <ChartSkeleton />
          <ChartSkeleton />
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-4 md:space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="font-display text-2xl md:text-3xl font-bold text-text-primary">仪表盘</h1>
          <p className="text-text-secondary font-mono mt-1">实时服务器监控</p>
        </div>
        <button 
          onClick={handleToggle}
          className="game-button hover-lift"
          aria-label="切换演示模式"
        >
          切换演示
        </button>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 md:gap-6">
        <MetricCard
          title="CPU 使用率"
          value={`${serverStatus.cpu.toFixed(1)}%`}
          icon={<Cpu className="w-6 h-6" aria-hidden="true" />}
          color="green"
          progress={serverStatus.cpu}
        />
        <MetricCard
          title="内存使用"
          value={`${serverStatus.memory.toFixed(1)}%`}
          icon={<HardDrive className="w-6 h-6" aria-hidden="true" />}
          color="green"
          progress={serverStatus.memory}
        />
        <MetricCard
          title="TPS"
          value={serverStatus.tps.toFixed(1)}
          icon={<Zap className="w-6 h-6" aria-hidden="true" />}
          color="yellow"
          subtitle="每秒滴答数"
        />
        <MetricCard
          title="玩家数"
          value={`${serverStatus.players}/${serverStatus.maxPlayers}`}
          icon={<Users className="w-6 h-6" aria-hidden="true" />}
          color="purple"
          subtitle="在线玩家"
        />
      </div>

      <div className="grid grid-cols-1 xl:grid-cols-2 gap-4 md:gap-6">
        {serverStatus.online && (
        <>
          <LineChart
            data={cpuMetrics}
            color="#5D7C15"
            title="CPU 使用历史"
            unit="%"
          />
          <LineChart
            data={memoryMetrics}
            color="#DEA584"
            title="内存使用历史"
            unit="%"
          />
        </>
        )}
      </div>

      {!serverStatus.online && (
        <div className="game-card">
          <ServerOfflineState onStart={startServer} />
        </div>
      )}

      {!isConnected && serverStatus.online && (
        <div className="fixed bottom-4 right-4 game-card p-4 max-w-sm" role="alert">
          <div className="flex items-start gap-3">
            <Activity className="w-5 h-5 text-rust flex-shrink-0 mt-0.5" aria-hidden="true" />
            <div>
              <p className="font-semibold text-text-primary">连接问题</p>
              <p className="text-sm text-text-secondary">WebSocket 已断开，正在尝试重新连接…</p>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
