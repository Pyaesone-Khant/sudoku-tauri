import { createContext, use, useCallback, useEffect, useMemo, useState, type ReactNode } from 'react';

/** Supported appearance modes matching macOS desktop conventions. */
export type ThemePreference = 'system' | 'light' | 'dark';

interface StoredPreferences {
  theme: ThemePreference;
  muted: boolean;
}

const STORAGE_KEY = 'sudoku.preferences.v1';

function loadStoredPreferences(): StoredPreferences {
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      return { theme: 'system', muted: false };
    }
    const parsed = JSON.parse(raw) as Partial<StoredPreferences>;
    const theme: ThemePreference =
      parsed.theme === 'light' || parsed.theme === 'dark' || parsed.theme === 'system'
        ? parsed.theme
        : 'system';
    return { theme, muted: Boolean(parsed.muted) };
  } catch {
    return { theme: 'system', muted: false };
  }
}

interface PreferencesContextValue {
  theme: ThemePreference;
  resolvedTheme: 'light' | 'dark';
  muted: boolean;
  setTheme: (theme: ThemePreference) => void;
  toggleMuted: () => void;
}

const PreferencesContext = createContext<PreferencesContextValue | null>(null);

/**
 * Manages appearance preference (`system` / `light` / `dark`) and synthesized audio mute state.
 */
export function PreferencesProvider({ children }: { children: ReactNode }) {
  const [prefs, setPrefs] = useState<StoredPreferences>(loadStoredPreferences);
  const [systemDark, setSystemDark] = useState<boolean>(() => {
    if (typeof window === 'undefined' || !window.matchMedia) {
      return false;
    }
    return window.matchMedia('(prefers-color-scheme: dark)').matches;
  });

  useEffect(() => {
    if (typeof window === 'undefined' || !window.matchMedia) {
      return;
    }
    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    const handler = (event: MediaQueryListEvent) => setSystemDark(event.matches);
    mq.addEventListener('change', handler);
    return () => mq.removeEventListener('change', handler);
  }, []);

  const resolvedTheme: 'light' | 'dark' =
    prefs.theme === 'system' ? (systemDark ? 'dark' : 'light') : prefs.theme;

  useEffect(() => {
    document.documentElement.dataset.theme = resolvedTheme;
  }, [resolvedTheme]);

  useEffect(() => {
    try {
      window.localStorage.setItem(STORAGE_KEY, JSON.stringify(prefs));
    } catch {
      // Ignore storage quota errors in restricted environments.
    }
  }, [prefs]);

  const setTheme = useCallback((theme: ThemePreference) => {
    setPrefs((prev) => ({ ...prev, theme }));
  }, []);

  const toggleMuted = useCallback(() => {
    setPrefs((prev) => ({ ...prev, muted: !prev.muted }));
  }, []);

  const value = useMemo<PreferencesContextValue>(
    () => ({
      theme: prefs.theme,
      resolvedTheme,
      muted: prefs.muted,
      setTheme,
      toggleMuted,
    }),
    [prefs.theme, prefs.muted, resolvedTheme, setTheme, toggleMuted],
  );

  return <PreferencesContext value={value}>{children}</PreferencesContext>;
}

/** Reads appearance and sound preferences from `PreferencesProvider`. */
export function usePreferences(): PreferencesContextValue {
  const ctx = use(PreferencesContext);
  if (!ctx) {
    throw new Error('usePreferences must be used within PreferencesProvider');
  }
  return ctx;
}
