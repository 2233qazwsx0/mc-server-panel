export interface ServerStatus {
  online: boolean;
  cpu: number;
  memory: number;
  tps: number;
  players: number;
  maxPlayers: number;
}

export interface MetricPoint {
  time: string;
  value: number;
}

export interface TerminalLine {
  id: string;
  timestamp: Date;
  content: string;
  type: 'log' | 'error' | 'info' | 'command';
}

export interface FileItem {
  name: string;
  type: 'file' | 'directory';
  size: number;
  modified: Date;
}

export interface WebSocketMessage {
  type: 'status' | 'metrics' | 'log' | 'error';
  data: any;
}
