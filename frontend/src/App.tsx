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
        gap: 8,
        padding: '0 8px',
        height: 36,
        borderRadius: 'var(--radius-sm)',
        fontSize: 13,
        fontWeight: active ? 500 : 400,
        background: active
          ? 'var(--sidebar-primary)'
          : hover
          ? 'var(--sidebar-accent)'
          : 'transparent',
        color: active ? 'var(--sidebar-primary-foreground)' : 'var(--sidebar-foreground)',
        cursor: 'pointer',
        transition: 'background 150ms ease, color 150ms ease',
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
        gap: 8,
        padding: '0 8px',
        height: 36,
        borderRadius: 'var(--radius-sm)',
        fontSize: 13,
        fontWeight: 400,
        background: hover ? 'var(--sidebar-accent)' : 'transparent',
        color: 'var(--sidebar-foreground)',
        cursor: 'pointer',
        border: 'none',
        width: '100%',
        textAlign: 'left',
        transition: 'background 150ms ease, color 150ms ease',
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
      data-slot="root"
      style={{
        background: 'transparent',
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
        data-slot="sidebar"
        style={{
          width: 'var(--sidebar-width)',
          display: 'flex',
          flexDirection: 'column',
          flexShrink: 0,
          background: 'var(--sidebar)',
          color: 'var(--sidebar-foreground)',
        }}
      >
        {/* Traffic light spacer — avoid overlapping with native overlay buttons */}
        <div style={{ height: 36, flexShrink: 0 }} />

        {/* Navigation */}
        <nav style={{ flex: 1, padding: '12px 8px 0', display: 'flex', flexDirection: 'column', gap: 1 }}>
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
            padding: '0 8px 12px',
            display: 'flex',
            flexDirection: 'column',
            gap: 1,
          }}
        >
          <ActionButton
            icon={mode === 'light' ? <Moon size={16} strokeWidth={2} /> : <Sun size={16} strokeWidth={2} />}
            label={mode === 'light' ? 'Dark' : 'Light'}
            onClick={toggle}
          />
        </div>
      </aside>

      {/* Main Content Area */}
      <main data-slot="content" style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
        {/* Content card — rounded left, subtle shadow */}
        <div
          style={{
            flex: 1,
            overflow: 'auto',
            padding: '16px 24px',
            background: 'var(--background)',
            borderRadius: 'var(--radius-xl)',
            margin: '8px 8px 8px 0',
            boxShadow: '-1px 0 3px -1px rgba(0,0,0,0.08)',
            minHeight: 0,
          }}
        >
          {activeTab === 'home' && <HomePage />}
          {activeTab === 'settings' && <SettingsPage />}
        </div>
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
          color: 'var(--foreground)',
          margin: 0,
          marginBottom: 12,
        }}
      >
        Tauri Template
      </h1>
      <p style={{ fontSize: 13, color: 'var(--muted-foreground)', lineHeight: 1.6 }}>
        A minimal Tauri template with borderless window, sidebar navigation, and dark/light theme.
      </p>

      <div data-slot="card"
        style={{
          marginTop: 24,
          padding: 16,
          background: 'var(--card)',
          borderRadius: 'var(--radius)',
          border: '1px solid var(--border)',
        }}
      >
        <h2 style={{ fontSize: 14, fontWeight: 600, margin: 0, marginBottom: 8, color: 'var(--card-foreground)' }}>Features</h2>
        <ul style={{ fontSize: 13, color: 'var(--muted-foreground)', margin: 0, paddingLeft: 18, lineHeight: 1.8 }}>
          <li>Borderless window with transparent background and rounded corners</li>
          <li>macOS overlay title bar with custom traffic light position</li>
          <li>Drag region and double-click to maximize</li>
          <li>Sidebar navigation with theme toggle</li>
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
          color: 'var(--foreground)',
          margin: 0,
          marginBottom: 12,
        }}
      >
        Settings
      </h1>

      <div
        data-slot="card"
        style={{
          padding: 16,
          background: 'var(--card)',
          borderRadius: 'var(--radius)',
          border: '1px solid var(--border)',
        }}
      >
        <h2 style={{ fontSize: 14, fontWeight: 600, margin: 0, marginBottom: 12, color: 'var(--card-foreground)' }}>Appearance</h2>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <span style={{ fontSize: 13, color: 'var(--muted-foreground)' }}>Theme</span>
          <button
            onClick={toggle}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 6,
              padding: '4px 12px',
              height: 32,
              borderRadius: 'var(--radius-sm)',
              fontSize: 13,
              fontWeight: 500,
              background: 'var(--primary)',
              color: 'var(--primary-foreground)',
              border: 'none',
              cursor: 'pointer',
              transition: 'background 150ms ease',
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
