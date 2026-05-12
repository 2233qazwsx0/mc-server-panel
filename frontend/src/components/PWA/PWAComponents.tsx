import { useState, useEffect } from 'react';

export interface PWAInstallPrompt {
  isStandalone: boolean;
  isInStandaloneMode: boolean;
  platform: string;
  canInstall: boolean;
  deferredPrompt: any | null;
  dismissPrompt: () => void;
  install: () => Promise<boolean>;
}

export const usePWAInstall = (): PWAInstallPrompt => {
  const [deferredPrompt, setDeferredPrompt] = useState<any | null>(null);
  const [isStandalone, setIsStandalone] = useState(false);

  useEffect(() => {
    const checkStandalone = () => {
      setIsStandalone(
        window.matchMedia('(display-mode: standalone)').matches ||
        (window.navigator as any).standalone ||
        document.referrer.includes('android-app://')
      );
    };

    checkStandalone();

    const mediaQuery = window.matchMedia('(display-mode: standalone)');
    mediaQuery.addEventListener('change', checkStandalone);

    return () => mediaQuery.removeEventListener('change', checkStandalone);
  }, []);

  useEffect(() => {
    const handleBeforeInstallPrompt = (e: Event) => {
      e.preventDefault();
      setDeferredPrompt(e);
    };

    window.addEventListener('beforeinstallprompt', handleBeforeInstallPrompt);

    return () => {
      window.removeEventListener('beforeinstallprompt', handleBeforeInstallPrompt);
    };
  }, []);

  const dismissPrompt = () => {
    setDeferredPrompt(null);
  };

  const install = async (): Promise<boolean> => {
    if (!deferredPrompt) {
      return false;
    }

    deferredPrompt.prompt();
    const { outcome } = await deferredPrompt.userChoice;

    if (outcome === 'accepted') {
      setDeferredPrompt(null);
      return true;
    }

    return false;
  };

  return {
    isStandalone,
    isInStandaloneMode: isStandalone,
    platform: (navigator as any).userAgentData?.platform || navigator.platform,
    canInstall: !!deferredPrompt,
    deferredPrompt,
    dismissPrompt,
    install,
  };
};

interface PWAInstallButtonProps {
  className?: string;
  children?: React.ReactNode;
}

export const PWAInstallButton: React.FC<PWAInstallButtonProps> = ({
  className = '',
  children,
}) => {
  const { canInstall, install, isStandalone } = usePWAInstall();

  if (isStandalone || !canInstall) {
    return null;
  }

  return (
    <button
      onClick={install}
      className={`flex items-center gap-2 px-4 py-2 bg-mc-green hover:bg-mc-green-light text-white rounded-lg transition-colors ${className}`}
    >
      <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={2}
          d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
        />
      </svg>
      {children || 'Install App'}
    </button>
  );
};

interface PWANotificationPermissionProps {
  onGranted?: () => void;
  onDenied?: () => void;
}

export const PWANotificationPermission: React.FC<PWANotificationPermissionProps> = ({
  onGranted,
  onDenied,
}) => {
  const [permission, setPermission] = useState<NotificationPermission>('default');

  useEffect(() => {
    if ('Notification' in window) {
      setPermission(Notification.permission);
    }
  }, []);

  const requestPermission = async () => {
    if (!('Notification' in window)) {
      console.warn('This browser does not support notifications');
      return;
    }

    const result = await Notification.requestPermission();
    setPermission(result);

    if (result === 'granted') {
      onGranted?.();
    } else {
      onDenied?.();
    }
  };

  if (permission === 'granted') {
    return (
      <div className="flex items-center gap-2 text-mc-green text-sm">
        <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
        </svg>
        Notifications enabled
      </div>
    );
  }

  if (permission === 'denied') {
    return (
      <div className="text-text-muted text-sm">
        Notifications blocked
      </div>
    );
  }

  return (
    <button
      onClick={requestPermission}
      className="flex items-center gap-2 px-4 py-2 bg-nether-700 hover:bg-nether-600 text-text-primary rounded-lg transition-colors text-sm"
    >
      <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={2}
          d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"
        />
      </svg>
      Enable Notifications
    </button>
  );
};

interface PWAOfflineIndicatorProps {
  className?: string;
}

export const PWAOfflineIndicator: React.FC<PWAOfflineIndicatorProps> = ({
  className = '',
}) => {
  const [isOnline, setIsOnline] = useState(navigator.onLine);

  useEffect(() => {
    const handleOnline = () => setIsOnline(true);
    const handleOffline = () => setIsOnline(false);

    window.addEventListener('online', handleOnline);
    window.addEventListener('offline', handleOffline);

    return () => {
      window.removeEventListener('online', handleOnline);
      window.removeEventListener('offline', handleOffline);
    };
  }, []);

  if (isOnline) {
    return null;
  }

  return (
    <div
      className={`fixed top-0 left-0 right-0 bg-rust px-4 py-2 text-center text-white text-sm font-medium z-50 ${className}`}
      role="alert"
    >
      <div className="flex items-center justify-center gap-2">
        <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M18.364 5.636a9 9 0 010 12.728m0 0l-2.829-2.829m2.829 2.829L21 21M15.536 8.464a5 5 0 010 7.072m0 0l-2.829-2.829m-4.243 2.829a4.978 4.978 0 01-1.414-2.83m-1.414 5.658a9 9 0 01-2.167-9.238m7.824 2.167a1 1 0 111.414 1.414m-1.414-1.414L3 3"
          />
        </svg>
        You are offline. Some features may be unavailable.
      </div>
    </div>
  );
};
