import { create } from 'zustand';

export type ThemeMode = 'light' | 'dark';

interface ThemeState {
  mode: ThemeMode;
  toggle: () => void;
  setMode: (mode: ThemeMode) => void;
}

const STORAGE_KEY = 'tauri-template-theme';

function getStoredTheme(): ThemeMode {
  if (typeof localStorage === 'undefined') {
    // Fallback to system preference
    if (typeof window !== 'undefined' && window.matchMedia) {
      return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    }
    return 'light';
  }
  const stored = localStorage.getItem(STORAGE_KEY);
  return stored === 'light' || stored === 'dark' ? stored : 'light';
}

function setStoredTheme(mode: ThemeMode) {
  try {
    localStorage.setItem(STORAGE_KEY, mode);
  } catch {
    // ignore (e.g., private browsing)
  }
}

function applyTheme(mode: ThemeMode) {
  const root = document.documentElement;
  if (mode === 'dark') {
    root.classList.add('dark');
  } else {
    root.classList.remove('dark');
  }
}

export const useThemeStore = create<ThemeState>((set) => ({
  mode: getStoredTheme(),
  toggle: () =>
    set((s) => {
      const next = s.mode === 'light' ? 'dark' : 'light';
      setStoredTheme(next);
      applyTheme(next);
      return { mode: next };
    }),
  setMode: (mode) => {
    setStoredTheme(mode);
    applyTheme(mode);
    set({ mode });
  },
}));
