import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './App.css';
import type { Tab } from './types';
import { useThemeStore } from './stores/themeStore';
import {
  Home,
  Settings,
  Sun,
  Moon,
} from 'lucide-react';
import StatusBar from './components/StatusBar';

/* ── Sidebar Nav Button ────────────────────────────────── */

interface NavButtonProps {
  icon: React.ComponentType<{ size?: number; strokeWidth?: number }>;
  label: string;
  active?: boolean;
  onClick?: () => void;
}

function NavButton({ icon: Icon, label, active, onClick }: NavButtonProps) {
  const [hover, setHover] = useState(false);

  return (
    <button
      onClick={onClick}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: 10,
        padding: '8px 12px',
        borderRadius: 6,
        fontSize: 13,
        fontWeight: active ? 600 : 400,
        background: active
          ? 'var(--accent-soft)'
          : hover
          ? 'var(--bg-alt)'
          : 'transparent',
        color: active ? 'var(--accent)' : 'var(--text-secondary)',
        cursor: 'pointer',
        transition: 'all 0.12s ease',
        border: 'none',
        width: '100%',
        textAlign: 'left',
      }}
    >
      <Icon size={16} strokeWidth={2} />
      <span>{label}</span>
    </button>
  );
}

/* ── Sidebar Action Button ─────────────────────────────── */

interface ActionButtonProps {
  icon: React.ReactNode;
  label: string;
  onClick?: () => void;
}

function ActionButton({ icon, label, onClick }: ActionButtonProps) {
  const [hover, setHover] = useState(false);

  return (
    <button
      onClick={onClick}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: 10,
        padding: '8px 12px',
        borderRadius: 6,
        fontSize: 13,
        fontWeight: 400,
        background: hover ? 'var(--bg-alt)' : 'transparent',
        color: 'var(--text-secondary)',
        cursor: 'pointer',
        border: 'none',
        width: '100%',
        textAlign: 'left',
        transition: 'all 0.12s ease',
      }}
    >
      {icon}
      <span>{label}</span>
    </button>
  );
}

function App() {
  const [activeTab, setActiveTab] = useState<Tab>('home');
  const { mode, toggle } = useThemeStore();

  const tabs: { key: Tab; label: string; icon: React.ComponentType<{ size?: number; strokeWidth?: number }> }[] = [
    { key: 'home', label: 'Home', icon: Home },
    { key: 'settings', label: 'Settings', icon: Settings },
  ];

  return (
    <div
      className="flex h-screen"
      style={{
        background: 'var(--sidebar-bg)',
        borderRadius: 12,
        overflow: 'hidden',
        position: 'relative',
      }}
    >
      {/* Drag bar — top area for dragging and double-click to maximize */}
      <div
        style={{
          position: 'absolute',
          top: 0,
          left: 0,
          right: 0,
          height: 46,
          zIndex: 1000,
          userSelect: 'none',
          WebkitUserSelect: 'none',
        }}
        onMouseDown={() => {
          invoke('win_start_drag').catch(() => {});
        }}
        onDoubleClick={() => {
          invoke('win_maximize').catch(() => {});
        }}
      />
      {/* Sidebar */}
      <aside
        id="tauri-sidebar"
        style={{
          width: 'var(--sidebar-width)',
          display: 'flex',
          flexDirection: 'column',
          flexShrink: 0,
        }}
      >
        {/* Traffic light spacer — avoid overlapping with native overlay buttons */}
        <div style={{ height: 36, flexShrink: 0 }} />

        {/* Navigation */}
        <nav style={{ flex: 1, padding: '8px 8px', display: 'flex', flexDirection: 'column', gap: 2 }}>
          {tabs.map((tab) => (
            <NavButton
              key={tab.key}
              icon={tab.icon}
              label={tab.label}
              active={activeTab === tab.key}
              onClick={() => setActiveTab(tab.key)}
            />
          ))}
        </nav>

        {/* Footer: theme toggle */}
        <div
          style={{
            padding: '8px',
            display: 'flex',
            flexDirection: 'column',
            gap: 2,
          }}
        >
          <ActionButton
            icon={mode === 'light' ? <Moon size={16} strokeWidth={2} /> : <Sun size={16} strokeWidth={2} />}
            label={mode === 'light' ? 'Dark' : 'Light'}
            onClick={toggle}
          />
        </div>
      </aside>

      {/* Main Content — rounded container */}
      <main style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
        <div
          style={{
            flex: 1,
            overflow: 'auto',
            padding: '12px 16px',
            background: 'var(--bg)',
            borderRadius: 10,
            margin: 8,
          }}
        >
          {activeTab === 'home' && <HomePage />}

          {activeTab === 'settings' && <SettingsPage />}
        </div>

        {/* Status Bar */}
        <StatusBar />
      </main>
    </div>
  );
}

/* ── Home Page ─────────────────────────────────────────── */

function HomePage() {
  return (
    <div>
      <h1
        style={{
          fontSize: 16,
          fontWeight: 700,
          color: 'var(--text)',
          margin: 0,
          marginBottom: 12,
        }}
      >
        Tauri Template
      </h1>
      <p style={{ fontSize: 13, color: 'var(--text-secondary)', lineHeight: 1.6 }}>
        A minimal Tauri template with borderless window, sidebar navigation, status bar, and dark/light theme.
      </p>

      <div
        style={{
          marginTop: 24,
          padding: 16,
          background: 'var(--surface)',
          borderRadius: 8,
          border: '1px solid var(--border)',
        }}
      >
        <h2 style={{ fontSize: 14, fontWeight: 600, margin: 0, marginBottom: 8 }}>Features</h2>
        <ul style={{ fontSize: 13, color: 'var(--text-secondary)', margin: 0, paddingLeft: 18, lineHeight: 1.8 }}>
          <li>Borderless window with transparent background and rounded corners</li>
          <li>macOS overlay title bar with custom traffic light position</li>
          <li>Drag region and double-click to maximize</li>
          <li>Sidebar navigation with theme toggle</li>
          <li>Bottom status bar</li>
          <li>Light / Dark theme with CSS variables</li>
          <li>Cross-monitor resize and centering</li>
        </ul>
      </div>
    </div>
  );
}

/* ── Settings Page ─────────────────────────────────────── */

function SettingsPage() {
  const { mode, toggle } = useThemeStore();

  return (
    <div>
      <h1
        style={{
          fontSize: 16,
          fontWeight: 700,
          color: 'var(--text)',
          margin: 0,
          marginBottom: 12,
        }}
      >
        Settings
      </h1>

      <div
        style={{
          padding: 16,
          background: 'var(--surface)',
          borderRadius: 8,
          border: '1px solid var(--border)',
        }}
      >
        <h2 style={{ fontSize: 14, fontWeight: 600, margin: 0, marginBottom: 12 }}>Appearance</h2>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <span style={{ fontSize: 13, color: 'var(--text-secondary)' }}>Theme</span>
          <button
            onClick={toggle}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 6,
              padding: '6px 12px',
              borderRadius: 6,
              fontSize: 13,
              background: 'var(--accent-soft)',
              color: 'var(--accent)',
              border: 'none',
              cursor: 'pointer',
            }}
          >
            {mode === 'light' ? <Moon size={14} /> : <Sun size={14} />}
            {mode === 'light' ? 'Switch to Dark' : 'Switch to Light'}
          </button>
        </div>
      </div>
    </div>
  );
}

export default App;
