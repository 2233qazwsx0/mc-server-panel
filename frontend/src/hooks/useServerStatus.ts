import { useState, useCallback, useEffect, useRef } from 'react';
import { ServerStatus, MetricPoint, TerminalLine } from '@/types';
import { useWebSocket } from './useWebSocket';

const MAX_METRICS = 50;
const MAX_TERMINAL_LINES = 500;

export function useServerStatus() {
  const [serverStatus, setServerStatus] = useState<ServerStatus>({
    online: false,
    cpu: 0,
    memory: 0,
    tps: 20,
    players: 0,
    maxPlayers: 100,
  });

  const [cpuMetrics, setCpuMetrics] = useState<MetricPoint[]>([]);
  const [memoryMetrics, setMemoryMetrics] = useState<MetricPoint[]>([]);
  const [terminalLines, setTerminalLines] = useState<TerminalLine[]>([]);
  
  // Use ref for serverStatus to avoid re-running interval effect on every change
  const serverStatusRef = useRef(serverStatus);
  useEffect(() => {
    serverStatusRef.current = serverStatus;
  }, [serverStatus]);

  const addTerminalLine = useCallback((content: string, type: TerminalLine['type'] = 'log') => {
    const line: TerminalLine = {
      id: Date.now().toString() + Math.random().toString(36).substr(2, 9),
      timestamp: new Date(),
      content,
      type,
    };

    setTerminalLines(prev => {
      if (prev.length >= MAX_TERMINAL_LINES) {
        return [...prev.slice(1), line];
      }
      return [...prev, line];
    });
  }, []);

  const addMetricPoint = useCallback((value: number, metricSetter: React.Dispatch<React.SetStateAction<MetricPoint[]>>) => {
    const now = new Date().toLocaleTimeString();
    const newPoint: MetricPoint = { time: now, value };
    metricSetter(prev => {
      if (prev.length >= MAX_METRICS) {
        return [...prev.slice(1), newPoint];
      }
      return [...prev, newPoint];
    });
  }, []);

  const handleMessage = useCallback((data: any) => {
    switch (data.type) {
      case 'status':
        setServerStatus(prev => ({ ...prev, ...data.data }));
        break;
      case 'metrics':
        if (data.data.cpu !== undefined) {
          addMetricPoint(data.data.cpu, setCpuMetrics);
        }
        if (data.data.memory !== undefined) {
          addMetricPoint(data.data.memory, setMemoryMetrics);
        }
        break;
      case 'log':
        addTerminalLine(data.data.message || data.data, 'log');
        break;
      case 'error':
        addTerminalLine(data.data.message || data.data, 'error');
        break;
    }
  }, [addMetricPoint, addTerminalLine]);

  const { isConnected, send, reconnect } = useWebSocket({
    url: 'ws://localhost:8080/ws',
    onMessage: handleMessage,
    onOpen: () => {
      addTerminalLine('[系统] 已连接到服务器', 'info');
    },
    onClose: () => {
      addTerminalLine('[系统] 连接已断开', 'error');
    },
    onError: () => {
      addTerminalLine('[系统] 连接错误', 'error');
    },
  });

  const sendCommand = useCallback((command: string) => {
    addTerminalLine(`> ${command}`, 'command');
    send({ type: 'command', data: command });
  }, [send, addTerminalLine]);

  const startServer = useCallback(() => {
    send({ type: 'server_start' });
  }, [send]);

  const stopServer = useCallback(() => {
    send({ type: 'server_stop' });
  }, [send]);

  const restartServer = useCallback(() => {
    send({ type: 'server_restart' });
  }, [send]);

  useEffect(() => {
    const interval = setInterval(() => {
      const currentStatus = serverStatusRef.current;
      if (!currentStatus.online) return;
      
      const now = new Date().toLocaleTimeString();
      
      // Update CPU metrics
      const newCpuValue = Math.min(100, Math.max(0, currentStatus.cpu + (Math.random() - 0.5) * 5));
      setCpuMetrics(prev => {
        const newPoint: MetricPoint = { time: now, value: newCpuValue };
        if (prev.length >= MAX_METRICS) {
          return [...prev.slice(1), newPoint];
        }
        return [...prev, newPoint];
      });

      // Update memory metrics
      const newMemoryValue = Math.min(100, Math.max(0, currentStatus.memory + (Math.random() - 0.5) * 2));
      setMemoryMetrics(prev => {
        const newPoint: MetricPoint = { time: now, value: newMemoryValue };
        if (prev.length >= MAX_METRICS) {
          return [...prev.slice(1), newPoint];
        }
        return [...prev, newPoint];
      });
    }, 2000);

    return () => clearInterval(interval);
  }, []); // Empty dependency array - use ref instead

  return {
    serverStatus,
    cpuMetrics,
    memoryMetrics,
    terminalLines,
    isConnected,
    sendCommand,
    startServer,
    stopServer,
    restartServer,
    reconnect,
    setServerStatus,
  };
}
