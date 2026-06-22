import { useEffect, useState } from 'react';

/**
 * 底部状态栏
 *
 * 显示应用状态、时间等信息
 */
export default function StatusBar() {
  const [time, setTime] = useState(new Date());
  const [status, setStatus] = useState<'ready' | 'loading'>('loading');

  useEffect(() => {
    // Simulate app initialization
    const timer = setTimeout(() => setStatus('ready'), 800);
    return () => clearTimeout(timer);
  }, []);

  useEffect(() => {
    const interval = setInterval(() => setTime(new Date()), 1000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div
      style={{
        height: 24,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: '0 12px',
        background: 'var(--surface)',
        borderTop: '1px solid var(--border)',
        fontSize: 11,
        color: 'var(--text-muted)',
        flexShrink: 0,
      }}
    >
      {/* Left: status indicator */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
        <span
          style={{
            width: 6,
            height: 6,
            borderRadius: '50%',
            background: status === 'ready' ? 'var(--success)' : 'var(--warning)',
            transition: 'background 0.3s ease',
          }}
        />
        <span>{status === 'ready' ? 'Ready' : 'Loading...'}</span>
      </div>

      {/* Right: time */}
      <span>
        {time.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit', second: '2-digit' })}
      </span>
    </div>
  );
}
