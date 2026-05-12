import { useEffect, useCallback, useState } from 'react';

export interface KeyboardShortcut {
  key: string;
  ctrl?: boolean;
  meta?: boolean;
  shift?: boolean;
  alt?: boolean;
  description: string;
  action: () => void;
  scope?: string;
  enabled?: boolean;
}

interface UseKeyboardShortcutsOptions {
  shortcuts: KeyboardShortcut[];
  enable?: boolean;
  scope?: string;
}

export const useKeyboardShortcuts = ({
  shortcuts,
  enable = true,
  scope,
}: UseKeyboardShortcutsOptions) => {
  const [enabledShortcuts, setEnabledShortcuts] = useState<KeyboardShortcut[]>(shortcuts);

  useEffect(() => {
    setEnabledShortcuts(shortcuts);
  }, [shortcuts]);

  useEffect(() => {
    if (!enable) return;

    const handleKeyDown = (event: KeyboardEvent) => {
      for (const shortcut of enabledShortcuts) {
        if (shortcut.enabled === false) continue;
        if (scope && shortcut.scope && shortcut.scope !== scope) continue;

        const keyMatch = event.key.toLowerCase() === shortcut.key.toLowerCase() ||
                        event.code.toLowerCase() === shortcut.key.toLowerCase();
        const ctrlMatch = shortcut.ctrl ? (event.ctrlKey || event.metaKey) : !event.ctrlKey && !event.metaKey;
        const metaMatch = shortcut.meta ? event.metaKey : true;
        const shiftMatch = shortcut.shift ? event.shiftKey : !event.shiftKey;
        const altMatch = shortcut.alt ? event.altKey : !event.altKey;

        if (keyMatch && ctrlMatch && metaMatch && shiftMatch && altMatch) {
          event.preventDefault();
          event.stopPropagation();
          shortcut.action();
          return;
        }
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [enabledShortcuts, enable, scope]);

  const updateShortcut = useCallback((key: string, updates: Partial<KeyboardShortcut>) => {
    setEnabledShortcuts(prev =>
      prev.map(shortcut =>
        shortcut.key.toLowerCase() === key.toLowerCase()
          ? { ...shortcut, ...updates }
          : shortcut
      )
    );
  }, []);

  const disableShortcut = useCallback((key: string) => {
    updateShortcut(key, { enabled: false });
  }, [updateShortcut]);

  const enableShortcut = useCallback((key: string) => {
    updateShortcut(key, { enabled: true });
  }, [updateShortcut]);

  return {
    shortcuts: enabledShortcuts,
    updateShortcut,
    disableShortcut,
    enableShortcut,
  };
};

export interface ShortcutCategory {
  name: string;
  shortcuts: KeyboardShortcut[];
}

export const KeyboardShortcutsHelp: React.FC<{
  categories: ShortcutCategory[];
  isOpen: boolean;
  onClose: () => void;
}> = ({ categories, isOpen, onClose }) => {
  if (!isOpen) return null;

  const formatShortcut = (shortcut: KeyboardShortcut) => {
    const keys: string[] = [];
    if (shortcut.ctrl) keys.push('Ctrl');
    if (shortcut.meta) keys.push('⌘');
    if (shortcut.shift) keys.push('⇧');
    if (shortcut.alt) keys.push('⌥');
    keys.push(shortcut.key.toUpperCase());
    return keys;
  };

  return (
    <div
      className="fixed inset-0 z-50 overflow-y-auto"
      role="dialog"
      aria-modal="true"
      aria-labelledby="shortcuts-title"
    >
      <div
        className="fixed inset-0 bg-black/60 backdrop-blur-sm"
        onClick={onClose}
      />

      <div className="relative min-h-full flex items-start justify-center pt-[10vh] px-4">
        <div className="relative w-full max-w-2xl bg-nether-800 rounded-xl shadow-2xl border border-nether-600 overflow-hidden max-h-[80vh]">
          <div className="sticky top-0 px-6 py-4 border-b border-nether-600 bg-nether-800 flex items-center justify-between">
            <h2 id="shortcuts-title" className="text-xl font-semibold text-text-primary">
              Keyboard Shortcuts
            </h2>
            <button
              onClick={onClose}
              className="p-2 hover:bg-nether-700 rounded-lg transition-colors"
              aria-label="Close"
            >
              <svg className="w-5 h-5 text-text-secondary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <div className="p-6 overflow-y-auto max-h-[calc(80vh-80px)]">
            <div className="space-y-6">
              {categories.map((category) => (
                <div key={category.name}>
                  <h3 className="text-sm font-semibold text-text-muted uppercase tracking-wider mb-3">
                    {category.name}
                  </h3>
                  <div className="space-y-2">
                    {category.shortcuts.map((shortcut, index) => (
                      <div
                        key={index}
                        className="flex items-center justify-between p-3 bg-nether-700 rounded-lg"
                      >
                        <span className="text-text-primary">{shortcut.description}</span>
                        <div className="flex items-center gap-1">
                          {formatShortcut(shortcut).map((key, keyIndex) => (
                            <span key={keyIndex}>
                              <kbd className="px-2 py-1 text-xs font-medium bg-nether-800 border border-nether-600 rounded text-text-secondary">
                                {key}
                              </kbd>
                              {keyIndex < formatShortcut(shortcut).length - 1 && (
                                <span className="mx-0.5 text-text-muted">+</span>
                              )}
                            </span>
                          ))}
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export const ShortcutProvider: React.FC<{
  children: React.ReactNode;
}> = ({ children }) => {
  const [helpOpen, setHelpOpen] = useState(false);

  const defaultShortcuts: KeyboardShortcut[] = [
    {
      key: 'k',
      ctrl: true,
      description: 'Open command palette',
      action: () => {},
      scope: 'global',
    },
    {
      key: 'n',
      ctrl: true,
      description: 'New server',
      action: () => {},
      scope: 'dashboard',
    },
    {
      key: '?',
      shift: true,
      description: 'Show keyboard shortcuts',
      action: () => setHelpOpen(true),
      scope: 'global',
    },
    {
      key: 'Escape',
      description: 'Close dialog/panel',
      action: () => setHelpOpen(false),
      scope: 'global',
    },
    {
      key: '1',
      description: 'Navigate to Dashboard',
      action: () => {},
      scope: 'navigation',
    },
    {
      key: '2',
      description: 'Navigate to Terminal',
      action: () => {},
      scope: 'navigation',
    },
    {
      key: '3',
      description: 'Navigate to Files',
      action: () => {},
      scope: 'navigation',
    },
  ];

  useKeyboardShortcuts({
    shortcuts: defaultShortcuts,
    enable: true,
  });

  return (
    <>
      {children}
      <KeyboardShortcutsHelp
        categories={[
          {
            name: 'Global',
            shortcuts: defaultShortcuts.filter(s => s.scope === 'global'),
          },
          {
            name: 'Navigation',
            shortcuts: defaultShortcuts.filter(s => s.scope === 'navigation'),
          },
        ]}
        isOpen={helpOpen}
        onClose={() => setHelpOpen(false)}
      />
    </>
  );
};
