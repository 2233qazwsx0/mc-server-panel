import { useRef, useEffect, useState } from 'react';
import { useServer } from '@/contexts/ServerContext';
import { Terminal as TerminalIcon, Send, WifiOff } from 'lucide-react';
import { clsx } from 'clsx';
import { TerminalSkeleton } from '@/components/LoadingSkeleton';
import { ConnectionLostState } from '@/components/EmptyState';

export function Terminal() {
  const { terminalLines, sendCommand, isConnected, reconnect } = useServer();
  const [input, setInput] = useState('');
  const [isLoading, setIsLoading] = useState(true);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const timer = setTimeout(() => {
      setIsLoading(false);
    }, 1000);
    return () => clearTimeout(timer);
  }, []);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [terminalLines]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (input.trim()) {
      sendCommand(input.trim());
      setInput('');
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      handleSubmit(e);
    }
  };

  const getLineClass = (type: string) => {
    switch (type) {
      case 'error':
        return 'terminal-line-error';
      case 'info':
        return 'terminal-line-info';
      case 'command':
        return 'terminal-line-command';
      default:
        return 'terminal-line-log';
    }
  };

  if (isLoading) {
    return (
      <div className="h-full flex flex-col">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between mb-4 gap-2">
          <div className="flex items-center gap-3">
            <TerminalIcon className="w-6 h-6 text-mc-green" aria-hidden="true" />
            <h2 className="font-display text-xl text-text-primary">服务器控制台</h2>
          </div>
        </div>
        <TerminalSkeleton />
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between mb-4 gap-2">
        <div className="flex items-center gap-3">
          <TerminalIcon className="w-6 h-6 text-mc-green" aria-hidden="true" />
          <h2 className="font-display text-xl text-text-primary">服务器控制台</h2>
        </div>
        <div className="flex items-center gap-2 text-xs text-text-muted font-mono">
          <span>{terminalLines.length} 行</span>
          {!isConnected && (
            <button
              onClick={reconnect}
              className="game-button p-1 text-xs flex items-center gap-1"
              aria-label="重新连接到服务器"
            >
              <WifiOff className="w-3 h-3" aria-hidden="true" />
              <span>重新连接</span>
            </button>
          )}
        </div>
      </div>

      {!isConnected && (
        <div className="mb-4">
          <ConnectionLostState onRetry={reconnect} />
        </div>
      )}

      <div className="terminal-container flex flex-col flex-1">
        <div className="terminal-header">
          <div className="w-3 h-3 rounded-full bg-status-error" aria-hidden="true" />
          <div className="w-3 h-3 rounded-full bg-rust" aria-hidden="true" />
          <div className="w-3 h-3 rounded-full bg-mc-green" aria-hidden="true" />
          <span className="terminal-title">minecraft-server.log</span>
        </div>

        <div 
          className="terminal-body"
          role="log"
          aria-live="polite"
          aria-label="服务器控制台输出"
        >
          {terminalLines.length === 0 ? (
            <div className="text-text-muted text-center py-8">
              <p className="terminal-line">等待服务器输出…</p>
            </div>
          ) : (
            <div className="space-y-1">
              {terminalLines.map((line) => (
                <div 
                  key={line.id} 
                  className={clsx('terminal-line', getLineClass(line.type))}
                  role="article"
                >
                  <span className="text-text-muted mr-2">[{line.timestamp.toLocaleTimeString()}]</span>
                  {line.content}
                </div>
              ))}
            </div>
          )}
          <div ref={messagesEndRef} />
        </div>

        <form onSubmit={handleSubmit} className="terminal-input-wrapper">
          <label htmlFor="command-input" className="sr-only">服务器命令</label>
          <span className="terminal-prompt" aria-hidden="true">$</span>
          <input
            id="command-input"
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={isConnected ? "输入命令…" : "连接已断开…"}
            className="terminal-input"
            autoComplete="off"
            spellCheck={false}
            disabled={!isConnected}
            aria-describedby="command-help"
          />
          <button
            type="submit"
            disabled={!input.trim() || !isConnected}
            className="game-button p-2 text-mc-green hover:bg-mc-green/10 disabled:opacity-50 disabled:cursor-not-allowed"
            aria-label="发送命令"
          >
            <Send className="w-4 h-4" aria-hidden="true" />
          </button>
        </form>
      </div>
    </div>
  );
}
