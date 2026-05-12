import React from 'react';
import { Sun, Moon, Monitor } from 'lucide-react';

interface ThemeToggleProps {
  variant?: 'icon' | 'button' | 'dropdown';
  className?: string;
  showLabel?: boolean;
}

export const ThemeToggle: React.FC<ThemeToggleProps> = ({
  variant = 'icon',
  className = '',
  showLabel = false,
}) => {
  const [theme, setTheme] = React.useState<'light' | 'dark' | 'system'>('dark');
  const [showDropdown, setShowDropdown] = React.useState(false);

  React.useEffect(() => {
    const savedTheme = localStorage.getItem('theme');
    if (savedTheme === 'light' || savedTheme === 'dark') {
      setTheme(savedTheme);
    }
  }, []);

  const handleThemeChange = (newTheme: 'light' | 'dark' | 'system') => {
    setTheme(newTheme);
    setShowDropdown(false);

    if (newTheme === 'system') {
      localStorage.removeItem('theme');
      const systemTheme = window.matchMedia('(prefers-color-scheme: light)').matches
        ? 'light'
        : 'dark';
      document.documentElement.classList.remove('light', 'dark');
      document.documentElement.classList.add(systemTheme);
    } else {
      localStorage.setItem('theme', newTheme);
      document.documentElement.classList.remove('light', 'dark');
      document.documentElement.classList.add(newTheme);
    }
  };

  const getCurrentIcon = () => {
    switch (theme) {
      case 'light':
        return <Sun className="w-5 h-5" />;
      case 'dark':
        return <Moon className="w-5 h-5" />;
      default:
        return <Monitor className="w-5 h-5" />;
    }
  };

  if (variant === 'icon') {
    return (
      <button
        onClick={() => handleThemeChange(theme === 'dark' ? 'light' : 'dark')}
        className={`p-2 rounded-lg hover:bg-nether-700 transition-colors focus:outline-none focus:ring-2 focus:ring-mc-green ${className}`}
        aria-label="Toggle theme"
        title={theme === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}
      >
        {getCurrentIcon()}
      </button>
    );
  }

  if (variant === 'dropdown') {
    return (
      <div className="relative">
        <button
          onClick={() => setShowDropdown(!showDropdown)}
          className={`flex items-center gap-2 px-3 py-2 rounded-lg hover:bg-nether-700 transition-colors focus:outline-none focus:ring-2 focus:ring-mc-green ${className}`}
          aria-label="Change theme"
          aria-expanded={showDropdown}
        >
          {getCurrentIcon()}
          {showLabel && (
            <span className="text-sm text-text-primary capitalize">{theme}</span>
          )}
          <svg
            className={`w-4 h-4 text-text-secondary transition-transform ${
              showDropdown ? 'rotate-180' : ''
            }`}
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M19 9l-7 7-7-7"
            />
          </svg>
        </button>

        {showDropdown && (
          <>
            <div
              className="fixed inset-0 z-10"
              onClick={() => setShowDropdown(false)}
            />
            <div className="absolute right-0 mt-2 w-48 bg-nether-800 border border-nether-600 rounded-lg shadow-xl z-20 overflow-hidden">
              <div className="p-2">
                <button
                  onClick={() => handleThemeChange('light')}
                  className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg transition-colors ${
                    theme === 'light'
                      ? 'bg-mc-green/20 text-mc-green'
                      : 'text-text-primary hover:bg-nether-700'
                  }`}
                >
                  <Sun className="w-4 h-4" />
                  <span className="text-sm">Light</span>
                  {theme === 'light' && (
                    <svg
                      className="w-4 h-4 ml-auto"
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M5 13l4 4L19 7"
                      />
                    </svg>
                  )}
                </button>

                <button
                  onClick={() => handleThemeChange('dark')}
                  className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg transition-colors ${
                    theme === 'dark'
                      ? 'bg-mc-green/20 text-mc-green'
                      : 'text-text-primary hover:bg-nether-700'
                  }`}
                >
                  <Moon className="w-4 h-4" />
                  <span className="text-sm">Dark</span>
                  {theme === 'dark' && (
                    <svg
                      className="w-4 h-4 ml-auto"
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M5 13l4 4L19 7"
                      />
                    </svg>
                  )}
                </button>

                <button
                  onClick={() => handleThemeChange('system')}
                  className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg transition-colors ${
                    theme === 'system'
                      ? 'bg-mc-green/20 text-mc-green'
                      : 'text-text-primary hover:bg-nether-700'
                  }`}
                >
                  <Monitor className="w-4 h-4" />
                  <span className="text-sm">System</span>
                  {theme === 'system' && (
                    <svg
                      className="w-4 h-4 ml-auto"
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M5 13l4 4L19 7"
                      />
                    </svg>
                  )}
                </button>
              </div>
            </div>
          </>
        )}
      </div>
    );
  }

  return (
    <button
      onClick={() => handleThemeChange(theme === 'dark' ? 'light' : 'dark')}
      className={`flex items-center gap-2 px-4 py-2 bg-nether-700 hover:bg-nether-600 rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-mc-green ${className}`}
    >
      {getCurrentIcon()}
      {showLabel && (
        <span className="text-sm text-text-primary">
          {theme === 'dark' ? 'Dark Mode' : 'Light Mode'}
        </span>
      )}
    </button>
  );
};

interface LanguageSelectorProps {
  variant?: 'icon' | 'button' | 'dropdown';
  className?: string;
  showLabel?: boolean;
}

export const LanguageSelector: React.FC<LanguageSelectorProps> = ({
  variant = 'icon',
  className = '',
  showLabel = false,
}) => {
  const { i18n } = React.useTranslation ? React.useTranslation() : { i18n: { language: 'en' } };
  const [showDropdown, setShowDropdown] = React.useState(false);

  const currentLanguage = i18n?.language || 'en';
  const languages = [
    { code: 'en', label: 'English', flag: '🇺🇸' },
    { code: 'zh', label: '中文', flag: '🇨🇳' },
  ];

  const currentLang = languages.find(l => l.code === currentLanguage) || languages[0];

  const handleLanguageChange = (code: string) => {
    i18n?.changeLanguage?.(code);
    setShowDropdown(false);
  };

  if (variant === 'icon') {
    return (
      <button
        onClick={() =>
          handleLanguageChange(currentLanguage === 'en' ? 'zh' : 'en')
        }
        className={`p-2 rounded-lg hover:bg-nether-700 transition-colors focus:outline-none focus:ring-2 focus:ring-mc-green ${className}`}
        aria-label="Toggle language"
        title={currentLanguage === 'en' ? '切换到中文' : 'Switch to English'}
      >
        <span className="text-lg">{currentLang.flag}</span>
      </button>
    );
  }

  if (variant === 'dropdown') {
    return (
      <div className="relative">
        <button
          onClick={() => setShowDropdown(!showDropdown)}
          className={`flex items-center gap-2 px-3 py-2 rounded-lg hover:bg-nether-700 transition-colors focus:outline-none focus:ring-2 focus:ring-mc-green ${className}`}
          aria-label="Change language"
          aria-expanded={showDropdown}
        >
          <span className="text-lg">{currentLang.flag}</span>
          {showLabel && (
            <span className="text-sm text-text-primary">{currentLang.label}</span>
          )}
          <svg
            className={`w-4 h-4 text-text-secondary transition-transform ${
              showDropdown ? 'rotate-180' : ''
            }`}
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M19 9l-7 7-7-7"
            />
          </svg>
        </button>

        {showDropdown && (
          <>
            <div
              className="fixed inset-0 z-10"
              onClick={() => setShowDropdown(false)}
            />
            <div className="absolute right-0 mt-2 w-48 bg-nether-800 border border-nether-600 rounded-lg shadow-xl z-20 overflow-hidden">
              <div className="p-2">
                {languages.map(lang => (
                  <button
                    key={lang.code}
                    onClick={() => handleLanguageChange(lang.code)}
                    className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg transition-colors ${
                      currentLanguage === lang.code
                        ? 'bg-mc-green/20 text-mc-green'
                        : 'text-text-primary hover:bg-nether-700'
                    }`}
                  >
                    <span className="text-lg">{lang.flag}</span>
                    <span className="text-sm">{lang.label}</span>
                    {currentLanguage === lang.code && (
                      <svg
                        className="w-4 h-4 ml-auto"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                      >
                        <path
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          strokeWidth={2}
                          d="M5 13l4 4L19 7"
                        />
                      </svg>
                    )}
                  </button>
                ))}
              </div>
            </div>
          </>
        )}
      </div>
    );
  }

  return (
    <button
      onClick={() =>
        handleLanguageChange(currentLanguage === 'en' ? 'zh' : 'en')
      }
      className={`flex items-center gap-2 px-4 py-2 bg-nether-700 hover:bg-nether-600 rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-mc-green ${className}`}
    >
      <span className="text-lg">{currentLang.flag}</span>
      {showLabel && (
        <span className="text-sm text-text-primary">{currentLang.label}</span>
      )}
    </button>
  );
};

interface ThemeProviderProps {
  children: React.ReactNode;
}

export const ThemeProvider: React.FC<ThemeProviderProps> = ({ children }) => {
  const [theme, setTheme] = React.useState<'light' | 'dark'>(() => {
    const saved = localStorage.getItem('theme');
    if (saved === 'light' || saved === 'dark') return saved;
    return window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
  });

  React.useEffect(() => {
    document.documentElement.classList.remove('light', 'dark');
    document.documentElement.classList.add(theme);
    localStorage.setItem('theme', theme);
  }, [theme]);

  const toggleTheme = () => {
    setTheme(prev => (prev === 'dark' ? 'light' : 'dark'));
  };

  return <>{children}</>;
};
