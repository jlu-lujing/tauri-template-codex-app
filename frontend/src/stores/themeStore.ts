import { create } from 'zustand';

export type ThemeMode = 'light' | 'dark';

interface ThemeState {
  mode: ThemeMode;
  toggle: () => void;
  setMode: (mode: ThemeMode) => void;
}

const STORAGE_KEY = 'tauri-template-theme';

function getStoredTheme(): ThemeMode {
  if (typeof localStorage === 'undefined') return 'light';
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

export const useThemeStore = create<ThemeState>((set) => ({
  mode: getStoredTheme(),
  toggle: () =>
    set((s) => {
      const next = s.mode === 'light' ? 'dark' : 'light';
      setStoredTheme(next);
      return { mode: next };
    }),
  setMode: (mode) => {
    setStoredTheme(mode);
    set({ mode });
  },
}));
