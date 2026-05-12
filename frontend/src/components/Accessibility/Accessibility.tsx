import React, { useEffect, useState } from 'react';

interface SkipLinkProps {
  targetId?: string;
}

export const SkipLink: React.FC<SkipLinkProps> = ({ targetId = 'main-content' }) => {
  return (
    <a
      href={`#${targetId}`}
      className="sr-only focus:not-sr-only focus:absolute focus:top-4 focus:left-4 focus:z-50 focus:px-4 focus:py-2 focus:bg-mc-green focus:text-white focus:rounded-lg focus:shadow-lg focus:outline-none focus:ring-2 focus:ring-mc-green-light"
    >
      Skip to main content
    </a>
  );
};

interface AriaLiveRegionProps {
  message: string;
  politeness?: 'polite' | 'assertive';
}

export const AriaLiveRegion: React.FC<AriaLiveRegionProps> = ({
  message,
  politeness = 'polite',
}) => {
  return (
    <div
      role="status"
      aria-live={politeness}
      aria-atomic="true"
      className="sr-only"
    >
      {message}
    </div>
  );
};

interface FocusTrapProps {
  children: React.ReactNode;
  isActive: boolean;
}

export const FocusTrap: React.FC<FocusTrapProps> = ({ children, isActive }) => {
  const containerRef = React.useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!isActive) return;

    const container = containerRef.current;
    if (!container) return;

    const focusableElements = container.querySelectorAll<HTMLElement>(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    );

    const firstElement = focusableElements[0];
    const lastElement = focusableElements[focusableElements.length - 1];

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key !== 'Tab') return;

      if (e.shiftKey) {
        if (document.activeElement === firstElement) {
          e.preventDefault();
          lastElement?.focus();
        }
      } else {
        if (document.activeElement === lastElement) {
          e.preventDefault();
          firstElement?.focus();
        }
      }
    };

    container.addEventListener('keydown', handleKeyDown);
    firstElement?.focus();

    return () => {
      container.removeEventListener('keydown', handleKeyDown);
    };
  }, [isActive]);

  return (
    <div ref={containerRef} style={{ outline: 'none' }}>
      {children}
    </div>
  );
};

interface AnnouncerProps {
  message: string;
  clearDelay?: number;
}

export const Announcer: React.FC<AnnouncerProps> = ({
  message,
  clearDelay = 1000,
}) => {
  const [announcement, setAnnouncement] = useState('');

  useEffect(() => {
    if (message) {
      setAnnouncement(message);
      const timer = setTimeout(() => setAnnouncement(''), clearDelay);
      return () => clearTimeout(timer);
    }
  }, [message, clearDelay]);

  return (
    <div
      role="status"
      aria-live="polite"
      aria-atomic="true"
      className="sr-only"
    >
      {announcement}
    </div>
  );
};

interface ColorContrastCheckerProps {
  foreground: string;
  background: string;
  minRatio?: number;
}

interface ContrastResult {
  ratio: number;
  passes: boolean;
  level: 'AA' | 'AAA' | 'Fail';
}

export const useColorContrast = ({
  foreground,
  background,
  minRatio = 4.5,
}: ColorContrastCheckerProps): ContrastResult => {
  const [contrast, setContrast] = useState<ContrastResult>({
    ratio: 0,
    passes: false,
    level: 'Fail'
  });

  useEffect(() => {
    const calculateContrast = () => {
      const getLuminance = (hex: string): number => {
        const rgb = hex.replace('#', '').match(/.{2}/g);
        if (!rgb) return 0;

        const [r, g, b] = rgb.map(x => {
          const val = parseInt(x, 16) / 255;
          return val <= 0.03928 ? val / 12.92 : Math.pow((val + 0.055) / 1.055, 2.4);
        });

        return 0.2126 * r + 0.7152 * g + 0.0722 * b;
      };

      const l1 = Math.max(getLuminance(foreground), getLuminance(background));
      const l2 = Math.min(getLuminance(foreground), getLuminance(background));
      const ratio = (l1 + 0.05) / (l2 + 0.05);

      let level: 'AA' | 'AAA' | 'Fail' = 'Fail';
      if (ratio >= 7) {
        level = 'AAA';
      } else if (ratio >= 4.5) {
        level = 'AA';
      }

      setContrast({
        ratio: Math.round(ratio * 100) / 100,
        passes: ratio >= minRatio,
        level,
      });
    };

    calculateContrast();
  }, [foreground, background, minRatio]);

  return contrast;
};

interface ReducedMotionProviderProps {
  children: React.ReactNode;
}

export const ReducedMotionProvider: React.FC<ReducedMotionProviderProps> = ({
  children,
}) => {
  const [prefersReducedMotion, setPrefersReducedMotion] = useState(false);

  useEffect(() => {
    const mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
    setPrefersReducedMotion(mediaQuery.matches);

    const handler = (e: MediaQueryListEvent) => setPrefersReducedMotion(e.matches);
    mediaQuery.addEventListener('change', handler);
    return () => mediaQuery.removeEventListener('change', handler);
  }, []);

  return (
    <div data-reduced-motion={prefersReducedMotion}>
      {children}
    </div>
  );
};

export const useReducedMotion = (): boolean => {
  const [prefersReducedMotion, setPrefersReducedMotion] = useState(false);

  useEffect(() => {
    const mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
    setPrefersReducedMotion(mediaQuery.matches);

    const handler = (e: MediaQueryListEvent) => setPrefersReducedMotion(e.matches);
    mediaQuery.addEventListener('change', handler);
    return () => mediaQuery.removeEventListener('change', handler);
  }, []);

  return prefersReducedMotion;
};

interface ScreenReaderOnlyProps {
  children: React.ReactNode;
}

export const ScreenReaderOnly: React.FC<ScreenReaderOnlyProps> = ({ children }) => {
  return <span className="sr-only">{children}</span>;
};

interface LandmarkRegionProps {
  children: React.ReactNode;
  label: string;
  id?: string;
  as?: 'section' | 'aside' | 'nav' | 'main' | 'header' | 'footer';
}

export const LandmarkRegion: React.FC<LandmarkRegionProps> = ({
  children,
  label,
  id,
  as: Component = 'section',
}) => {
  return (
    <Component aria-label={label} id={id}>
      {children}
    </Component>
  );
};
