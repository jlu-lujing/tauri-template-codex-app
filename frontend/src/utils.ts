/** 格式化纳秒时间戳为短日期 (M-D) */
export function fmtDate(ns: number): string {
  const d = new Date(Math.floor(ns / 1_000_000));
  const m = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${m}-${day}`;
}

/** 格式化纳秒时间戳为完整日期 */
export function fmtFullDate(ns: number): string {
  const d = new Date(Math.floor(ns / 1_000_000));
  return d.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  });
}
