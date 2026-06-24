import { StrictMode, useEffect } from 'react';
import { createRoot } from 'react-dom/client';
import './index.css';
import App from './App';
import { useThemeStore } from './stores/themeStore';

// Apply stored theme immediately to avoid flash on first paint
const storedTheme = localStorage.getItem('tauri-template-theme');
if (storedTheme === 'dark') {
  document.documentElement.classList.add('dark');
} else if (storedTheme !== 'light' && window.matchMedia('(prefers-color-scheme: dark)').matches) {
  document.documentElement.classList.add('dark');
}

// Apply glass effect on macOS by default (Tauri transparent window looks best with it)
// 临时调试：所有平台默认开启，方便看到效果
document.documentElement.classList.add('glass');
const isMac = (() => {
  if (typeof navigator === 'undefined') return false;
  const ua = navigator.userAgent;
  return /Mac/.test(ua) || /Mac/.test(navigator.platform || '');
})();
const glassDisabled = localStorage.getItem('tauri-template-glass') === 'off';
if (isMac && !glassDisabled) {
  document.documentElement.classList.add('glass');
}

/**
 * Theme provider — applies .dark class to <html>
 */
function ThemeProvider({ children }: { children: React.ReactNode }) {
  const mode = useThemeStore((s) => s.mode);

  useEffect(() => {
    if (mode === 'dark') {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  }, [mode]);

  return <>{children}</>;
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <ThemeProvider>
      <App />
    </ThemeProvider>
  </StrictMode>,
);
