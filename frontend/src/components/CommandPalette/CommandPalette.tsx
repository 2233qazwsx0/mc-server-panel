import React, { useState, useEffect, useCallback, useRef } from 'react';

interface Command {
  id: string;
  name: string;
  description: string;
  category: string;
  command: string;
  icon?: React.ReactNode;
}

const commands: Command[] = [
  {
    id: 'start',
    name: 'Start Server',
    description: 'Start the Minecraft server',
    category: 'Server',
    command: '/start',
  },
  {
    id: 'stop',
    name: 'Stop Server',
    description: 'Stop the Minecraft server gracefully',
    category: 'Server',
    command: '/stop',
  },
  {
    id: 'restart',
    name: 'Restart Server',
    description: 'Restart the Minecraft server',
    category: 'Server',
    command: '/restart',
  },
  {
    id: 'status',
    name: 'Server Status',
    description: 'Check current server status',
    category: 'Info',
    command: '/status',
  },
  {
    id: 'players',
    name: 'List Players',
    description: 'List all connected players',
    category: 'Players',
    command: '/list',
  },
  {
    id: 'kick',
    name: 'Kick Player',
    description: 'Kick a player from the server',
    category: 'Players',
    command: '/kick',
  },
  {
    id: 'ban',
    name: 'Ban Player',
    description: 'Ban a player from the server',
    category: 'Players',
    command: '/ban',
  },
  {
    id: 'whitelist',
    name: 'Whitelist Management',
    description: 'Manage server whitelist',
    category: 'Players',
    command: '/whitelist',
  },
  {
    id: 'op',
    name: 'Make Operator',
    description: 'Give a player operator status',
    category: 'Players',
    command: '/op',
  },
  {
    id: 'deop',
    name: 'Remove Operator',
    description: 'Remove player operator status',
    category: 'Players',
    command: '/deop',
  },
  {
    id: 'tp',
    name: 'Teleport',
    description: 'Teleport a player',
    category: 'World',
    command: '/tp',
  },
  {
    id: 'gamemode',
    name: 'Change Gamemode',
    description: 'Change player gamemode',
    category: 'World',
    command: '/gamemode',
  },
  {
    id: 'give',
    name: 'Give Item',
    description: 'Give an item to a player',
    category: 'Items',
    command: '/give',
  },
  {
    id: 'weather',
    name: 'Set Weather',
    description: 'Change server weather',
    category: 'World',
    command: '/weather',
  },
  {
    id: 'time',
    name: 'Set Time',
    description: 'Set world time',
    category: 'World',
    command: '/time',
  },
  {
    id: 'say',
    name: 'Broadcast Message',
    description: 'Send a message to all players',
    category: 'Chat',
    command: '/say',
  },
  {
    id: 'title',
    name: 'Display Title',
    description: 'Display a title to players',
    category: 'Chat',
    command: '/title',
  },
  {
    id: 'difficulty',
    name: 'Set Difficulty',
    description: 'Change server difficulty',
    category: 'Server',
    command: '/difficulty',
  },
  {
    id: 'whitelist-on',
    name: 'Enable Whitelist',
    description: 'Enable server whitelist',
    category: 'Server',
    command: '/whitelist on',
  },
  {
    id: 'whitelist-off',
    name: 'Disable Whitelist',
    description: 'Disable server whitelist',
    category: 'Server',
    command: '/whitelist off',
  },
];

interface CommandPaletteProps {
  isOpen: boolean;
  onClose: () => void;
  onExecute: (command: string) => void;
}

export const CommandPalette: React.FC<CommandPaletteProps> = ({
  isOpen,
  onClose,
  onExecute,
}) => {
  const [search, setSearch] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  const filteredCommands = commands.filter(cmd =>
    cmd.name.toLowerCase().includes(search.toLowerCase()) ||
    cmd.description.toLowerCase().includes(search.toLowerCase()) ||
    cmd.command.toLowerCase().includes(search.toLowerCase())
  );

  const groupedCommands = filteredCommands.reduce((acc, cmd) => {
    if (!acc[cmd.category]) {
      acc[cmd.category] = [];
    }
    acc[cmd.category].push(cmd);
    return acc;
  }, {} as Record<string, Command[]>);

  useEffect(() => {
    if (isOpen && inputRef.current) {
      inputRef.current.focus();
      setSearch('');
      setSelectedIndex(0);
    }
  }, [isOpen]);

  useEffect(() => {
    setSelectedIndex(0);
  }, [search]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedIndex(prev =>
          prev < filteredCommands.length - 1 ? prev + 1 : prev
        );
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex(prev => prev > 0 ? prev - 1 : prev);
        break;
      case 'Enter':
        e.preventDefault();
        if (filteredCommands[selectedIndex]) {
          onExecute(filteredCommands[selectedIndex].command);
          onClose();
        }
        break;
      case 'Escape':
        e.preventDefault();
        onClose();
        break;
    }
  }, [filteredCommands, selectedIndex, onExecute, onClose]);

  useEffect(() => {
    const handleGlobalKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
      }
    };

    if (isOpen) {
      document.addEventListener('keydown', handleGlobalKeyDown);
      return () => document.removeEventListener('keydown', handleGlobalKeyDown);
    }
  }, [isOpen]);

  if (!isOpen) return null;

  let currentIndex = 0;

  return (
    <div
      className="fixed inset-0 z-50 overflow-y-auto"
      role="dialog"
      aria-modal="true"
      aria-label="Command Palette"
    >
      <div
        className="fixed inset-0 bg-black/60 backdrop-blur-sm"
        onClick={onClose}
      />

      <div className="relative min-h-full flex items-start justify-center pt-[15vh] px-4">
        <div className="relative w-full max-w-xl bg-nether-800 rounded-xl shadow-2xl border border-nether-600 overflow-hidden">
          <div className="p-4 border-b border-nether-600">
            <div className="flex items-center gap-3">
              <svg
                className="w-5 h-5 text-text-secondary"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
                />
              </svg>
              <input
                ref={inputRef}
                type="text"
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder="Type a command..."
                className="flex-1 bg-transparent text-text-primary placeholder-text-muted outline-none text-lg"
                aria-label="Search commands"
              />
              <kbd className="px-2 py-1 text-xs text-text-muted bg-nether-700 rounded border border-nether-600">
                ESC
              </kbd>
            </div>
          </div>

          <div
            ref={listRef}
            className="max-h-96 overflow-y-auto p-2"
            role="listbox"
            aria-label="Available commands"
          >
            {Object.entries(groupedCommands).map(([category, cmds]) => (
              <div key={category} className="mb-2">
                <div className="px-3 py-1.5 text-xs font-semibold text-text-muted uppercase tracking-wider">
                  {category}
                </div>
                {cmds.map((cmd) => {
                  const itemIndex = currentIndex++;
                  const isSelected = itemIndex === selectedIndex;

                  return (
                    <button
                      key={cmd.id}
                      onClick={() => {
                        onExecute(cmd.command);
                        onClose();
                      }}
                      className={`w-full flex items-center gap-3 px-3 py-2.5 rounded-lg transition-colors ${
                        isSelected
                          ? 'bg-mc-green/20 text-mc-green'
                          : 'text-text-primary hover:bg-nether-700'
                      }`}
                      role="option"
                      aria-selected={isSelected}
                    >
                      <div className={`p-1.5 rounded ${
                        isSelected ? 'bg-mc-green/20' : 'bg-nether-700'
                      }`}>
                        <svg
                          className="w-4 h-4"
                          fill="none"
                          stroke="currentColor"
                          viewBox="0 0 24 24"
                        >
                          <path
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth={2}
                            d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
                          />
                        </svg>
                      </div>
                      <div className="flex-1 text-left">
                        <div className="font-medium">{cmd.name}</div>
                        <div className="text-xs text-text-secondary">{cmd.description}</div>
                      </div>
                      <code className="text-xs text-mc-green bg-nether-700/50 px-2 py-1 rounded">
                        {cmd.command}
                      </code>
                    </button>
                  );
                })}
              </div>
            ))}

            {filteredCommands.length === 0 && (
              <div className="px-3 py-8 text-center text-text-muted">
                No commands found
              </div>
            )}
          </div>

          <div className="px-4 py-3 border-t border-nether-600 bg-nether-800/50 flex items-center gap-4 text-xs text-text-muted">
            <span className="flex items-center gap-1">
              <kbd className="px-1.5 py-0.5 bg-nether-700 rounded border border-nether-600">↑</kbd>
              <kbd className="px-1.5 py-0.5 bg-nether-700 rounded border border-nether-600">↓</kbd>
              Navigate
            </span>
            <span className="flex items-center gap-1">
              <kbd className="px-1.5 py-0.5 bg-nether-700 rounded border border-nether-600">Enter</kbd>
              Execute
            </span>
          </div>
        </div>
      </div>
    </div>
  );
};

interface CommandPaletteTriggerProps {
  onClick: () => void;
  className?: string;
}

export const CommandPaletteTrigger: React.FC<CommandPaletteTriggerProps> = ({
  onClick,
  className = '',
}) => {
  return (
    <button
      onClick={onClick}
      className={`flex items-center gap-2 px-3 py-2 bg-nether-700 hover:bg-nether-600 border border-nether-600 rounded-lg transition-colors ${className}`}
      aria-label="Open command palette"
    >
      <svg
        className="w-4 h-4 text-text-secondary"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={2}
          d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
        />
      </svg>
      <span className="text-sm text-text-secondary">Commands</span>
      <kbd className="ml-2 px-1.5 py-0.5 text-xs text-text-muted bg-nether-800 rounded border border-nether-600">
        Ctrl+K
      </kbd>
    </button>
  );
};
