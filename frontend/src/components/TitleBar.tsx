import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

/**
 * macOS 原生红绿灯窗口控制按钮
 *
 * 14x14px 圆形，经典三色
 * 联动 hover：鼠标进入任意按钮，三个同时显示图标（带淡入动画）
 */

interface TrafficLightProps {
  color: string;
  icon: React.ReactNode;
  onClick: () => void;
  showIcon: boolean;
}

function TrafficLight({ color, icon, onClick, showIcon }: TrafficLightProps) {
  return (
    <button
      onClick={onClick}
      onMouseDown={(e) => e.stopPropagation()}
      style={{
        width: 14,
        height: 14,
        minWidth: 14,
        minHeight: 14,
        borderRadius: '50%',
        background: color,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        cursor: 'pointer',
        border: 'none',
        padding: 0,
        outline: 'none',
        color: showIcon ? '#4a4a4a' : 'transparent',
        transition: 'color 0.15s ease',
      }}
    >
      <span
        style={{
          opacity: showIcon ? 1 : 0,
          transform: showIcon ? 'scale(1)' : 'scale(0.6)',
          transition: 'opacity 0.15s ease, transform 0.15s ease',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
        }}
      >
        {icon}
      </span>
    </button>
  );
}

export default function SidebarControls() {
  const [isTauri, setIsTauri] = useState(false);
  const [hovered, setHovered] = useState(false);
  const controlsRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    // @ts-expect-error __TAURI_INTERNALS__ is injected by Tauri runtime
    const ok = typeof window !== 'undefined' && !!window.__TAURI_INTERNALS__;
    setIsTauri(ok);
  }, []);

  useEffect(() => {
    if (!isTauri) return;
    const el = document.getElementById('jquant-sidebar');
    if (!el) return;

    const handler = (e: MouseEvent) => {
      if (controlsRef.current && controlsRef.current.contains(e.target as Node)) return;
      invoke('win_start_drag').catch(() => {});
    };

    const dblHandler = (e: MouseEvent) => {
      if (controlsRef.current && controlsRef.current.contains(e.target as Node)) return;
      invoke('win_maximize').catch(() => {});
    };

    el.addEventListener('mousedown', handler);
    el.addEventListener('dblclick', dblHandler);
    return () => {
      el.removeEventListener('mousedown', handler);
      el.removeEventListener('dblclick', dblHandler);
    };
  }, [isTauri]);

  const handleClose = () => invoke('win_close').catch(() => {});
  const handleMinimize = () => invoke('win_minimize').catch(() => {});
  const handleMaximize = () => invoke('win_maximize').catch(() => {});

  return (
    <div
      ref={controlsRef}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        display: 'flex',
        gap: 8,
        padding: '14px 16px 10px',
      }}
    >
      {/* Close — Red */}
      <TrafficLight
        color="#ff5f57"
        onClick={handleClose}
        showIcon={hovered}
        icon={
          <svg width="9" height="9" viewBox="0 0 12 12">
            <path
              d="M2.5 2.5l7 7M9.5 2.5l-7 7"
              stroke="currentColor"
              strokeWidth="1.6"
              strokeLinecap="round"
              fill="none"
            />
          </svg>
        }
      />
      {/* Minimize — Yellow */}
      <TrafficLight
        color="#febc2e"
        onClick={handleMinimize}
        showIcon={hovered}
        icon={
          <svg width="9" height="9" viewBox="0 0 12 12">
            <path
              d="M2 6h8"
              stroke="currentColor"
              strokeWidth="1.6"
              strokeLinecap="round"
              fill="none"
            />
          </svg>
        }
      />
      {/* Maximize — Green */}
      <TrafficLight
        color="#28c840"
        onClick={handleMaximize}
        showIcon={hovered}
        icon={
          <svg width="9" height="9" viewBox="0 0 12 12">
            <path
              d="M2 6L6 2h4v4M10 6L6 10H2v-4"
              stroke="currentColor"
              strokeWidth="1.4"
              strokeLinecap="round"
              strokeLinejoin="round"
              fill="none"
            />
          </svg>
        }
      />
    </div>
  );
}
