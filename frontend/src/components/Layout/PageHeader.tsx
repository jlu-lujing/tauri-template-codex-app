interface Props {
  title: string;
  subtitle?: string;
  action?: React.ReactNode;
}

/**
 * 页面标题栏 — 统一各页面顶部样式
 */
export default function PageHeader({ title, subtitle, action }: Props) {
  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'flex-start',
        justifyContent: 'space-between',
        marginBottom: 12,
      }}
    >
      <div>
        <h1
          style={{
            fontSize: 16,
            fontWeight: 700,
            color: 'var(--color-text)',
            margin: 0,
          }}
        >
          {title}
        </h1>
        {subtitle && (
          <p
            style={{
              fontSize: 12,
              color: 'var(--color-text-muted)',
              margin: '2px 0 0',
            }}
          >
            {subtitle}
          </p>
        )}
      </div>
      {action && <div>{action}</div>}
    </div>
  );
}
