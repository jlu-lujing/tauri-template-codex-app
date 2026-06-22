import type { CSSProperties } from 'react';

/** 共享表单输入样式 */
export const inputStyle: CSSProperties = {
  width: '100%',
  padding: '6px 10px',
  border: '1px solid var(--color-border)',
  borderRadius: 6,
  fontSize: 13,
  background: 'var(--color-surface-alt)',
  color: 'var(--color-text)',
  outline: 'none',
  boxSizing: 'border-box',
  fontFamily: "'IBM Plex Mono', monospace",
};

/** 共享表单标签样式 */
export const labelStyle: CSSProperties = {
  fontSize: 11,
  fontWeight: 500,
  color: 'var(--color-text-muted)',
  marginBottom: 4,
  display: 'block',
  textTransform: 'uppercase',
  letterSpacing: '0.5px',
};

/** 共享卡片容器样式 */
export const cardStyle: CSSProperties = {
  background: 'var(--color-surface)',
  borderRadius: 8,
  border: '1px solid var(--color-border)',
  padding: 14,
};

/** 共享错误提示样式 */
export const errorStyle: CSSProperties = {
  background: 'var(--color-danger-soft)',
  border: '1px solid var(--color-danger)',
  borderRadius: 6,
  padding: '6px 10px',
  fontSize: 12,
  color: 'var(--color-danger)',
};
