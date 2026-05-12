import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';

export type Theme = 'light' | 'dark';
export type Language = 'en' | 'zh';

interface ThemeContextType {
  theme: Theme;
  toggleTheme: () => void;
  setTheme: (theme: Theme) => void;
}

interface LanguageContextType {
  language: Language;
  setLanguage: (lang: Language) => void;
  t: (key: string) => string;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);
const LanguageContext = createContext<LanguageContextType | undefined>(undefined);

interface AppProvidersProps {
  children: ReactNode;
}

export const AppProviders: React.FC<AppProvidersProps> = ({ children }) => {
  const [theme, setThemeState] = useState<Theme>(() => {
    const saved = localStorage.getItem('theme');
    if (saved === 'light' || saved === 'dark') return saved;
    return window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
  });

  const [language, setLanguageState] = useState<Language>(() => {
    const saved = localStorage.getItem('language');
    if (saved === 'en' || saved === 'zh') return saved;
    const browserLang = navigator.language.split('-')[0];
    return browserLang === 'zh' ? 'zh' : 'en';
  });

  useEffect(() => {
    document.documentElement.classList.remove('light', 'dark');
    document.documentElement.classList.add(theme);
    localStorage.setItem('theme', theme);
  }, [theme]);

  useEffect(() => {
    localStorage.setItem('language', language);
  }, [language]);

  const toggleTheme = () => {
    setThemeState(prev => prev === 'dark' ? 'light' : 'dark');
  };

  const setTheme = (newTheme: Theme) => {
    setThemeState(newTheme);
  };

  const setLanguage = (lang: Language) => {
    setLanguageState(lang);
  };

  const translations: Record<Language, Record<string, string>> = {
    en: {
      'dashboard.title': 'Dashboard',
      'dashboard.settings': 'Settings',
      'server.status': 'Server Status',
      'server.online': 'Online',
      'server.offline': 'Offline',
      'server.players': 'Players',
      'server.cpu': 'CPU Usage',
      'server.memory': 'Memory Usage',
      'server.disk': 'Disk Usage',
      'server.tps': 'TPS',
      'notifications.title': 'Notifications',
      'notifications.markAllRead': 'Mark all as read',
      'notifications.empty': 'No notifications',
      'command.title': 'Command Palette',
      'command.placeholder': 'Type a command...',
      'batch.title': 'Batch Operations',
      'batch.select': 'Select items',
      'batch.confirm': 'Confirm',
      'batch.cancel': 'Cancel',
      'accessibility.skipLink': 'Skip to main content',
      'accessibility.theme': 'Toggle theme',
      'accessibility.language': 'Change language',
    },
    zh: {
      'dashboard.title': '仪表盘',
      'dashboard.settings': '设置',
      'server.status': '服务器状态',
      'server.online': '在线',
      'server.offline': '离线',
      'server.players': '玩家数',
      'server.cpu': 'CPU 使用率',
      'server.memory': '内存使用率',
      'server.disk': '磁盘使用率',
      'server.tps': 'TPS',
      'notifications.title': '通知中心',
      'notifications.markAllRead': '全部标记为已读',
      'notifications.empty': '暂无通知',
      'command.title': '命令面板',
      'command.placeholder': '输入命令...',
      'batch.title': '批量操作',
      'batch.select': '选择项目',
      'batch.confirm': '确认',
      'batch.cancel': '取消',
      'accessibility.skipLink': '跳转到主要内容',
      'accessibility.theme': '切换主题',
      'accessibility.language': '切换语言',
    },
  };

  const t = (key: string): string => {
    return translations[language][key] || key;
  };

  return (
    <ThemeContext.Provider value={{ theme, toggleTheme, setTheme }}>
      <LanguageContext.Provider value={{ language, setLanguage, t }}>
        {children}
      </LanguageContext.Provider>
    </ThemeContext.Provider>
  );
};

export const useTheme = (): ThemeContextType => {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error('useTheme must be used within AppProviders');
  }
  return context;
};

export const useLanguage = (): LanguageContextType => {
  const context = useContext(LanguageContext);
  if (!context) {
    throw new Error('useLanguage must be used within AppProviders');
  }
  return context;
};
