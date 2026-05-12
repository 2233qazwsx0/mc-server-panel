import { createContext, useContext, ReactNode } from 'react';
import { useServerStatus } from '@/hooks/useServerStatus';

interface ServerContextType {
  serverStatus: ReturnType<typeof useServerStatus>['serverStatus'];
  cpuMetrics: ReturnType<typeof useServerStatus>['cpuMetrics'];
  memoryMetrics: ReturnType<typeof useServerStatus>['memoryMetrics'];
  terminalLines: ReturnType<typeof useServerStatus>['terminalLines'];
  isConnected: ReturnType<typeof useServerStatus>['isConnected'];
  sendCommand: ReturnType<typeof useServerStatus>['sendCommand'];
  startServer: ReturnType<typeof useServerStatus>['startServer'];
  stopServer: ReturnType<typeof useServerStatus>['stopServer'];
  restartServer: ReturnType<typeof useServerStatus>['restartServer'];
  reconnect: ReturnType<typeof useServerStatus>['reconnect'];
  setServerStatus: ReturnType<typeof useServerStatus>['setServerStatus'];
}

const ServerContext = createContext<ServerContextType | undefined>(undefined);

export function ServerProvider({ children }: { children: ReactNode }) {
  const serverStatusData = useServerStatus();

  return (
    <ServerContext.Provider value={serverStatusData}>
      {children}
    </ServerContext.Provider>
  );
}

export function useServer() {
  const context = useContext(ServerContext);
  if (context === undefined) {
    throw new Error('useServer must be used within a ServerProvider');
  }
  return context;
}
