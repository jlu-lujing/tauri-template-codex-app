import type { CSSProperties } from 'react';

/** 共享表单输入样式 */
export const inputStyle: CSSProperties = {
  width: '100%',
  padding: '6px 10px',
  border: '1px solid var(--border)',
  borderRadius: 'var(--radius-sm)',
  fontSize: 13,
  background: 'var(--background)',
  color: 'var(--foreground)',
  outline: 'none',
  boxSizing: 'border-box',
  fontFamily: "'IBM Plex Mono', 'SF Mono', 'Fira Code', monospace",
};

/** 共享表单标签样式 */
export const labelStyle: CSSProperties = {
  fontSize: 11,
  fontWeight: 500,
  color: 'var(--muted-foreground)',
  marginBottom: 4,
  display: 'block',
  textTransform: 'uppercase',
  letterSpacing: '0.5px',
};

/** 共享卡片容器样式 */
export const cardStyle: CSSProperties = {
  background: 'var(--card)',
  borderRadius: 'var(--radius)',
  border: '1px solid var(--border)',
  padding: 16,
};

/** 共享错误提示样式 */
export const errorStyle: CSSProperties = {
  background: 'var(--destructive)',
  border: '1px solid var(--destructive)',
  borderRadius: 'var(--radius-sm)',
  padding: '6px 10px',
  fontSize: 12,
  color: 'var(--destructive-foreground)',
};
