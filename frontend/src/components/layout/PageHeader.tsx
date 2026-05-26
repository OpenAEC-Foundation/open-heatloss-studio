import type { ReactNode } from "react";

interface PageHeaderProps {
  title: string;
  subtitle?: string;
  actions?: ReactNode;
}

export function PageHeader({ title, subtitle, actions }: PageHeaderProps) {
  return (
    <header className="sticky top-0 z-20 border-b border-[var(--oaec-border-subtle)] bg-surface">
      {/* Title bar */}
      <div className="flex h-header items-center justify-between px-6">
        <div className="flex items-baseline gap-3">
          <h1 className="font-heading text-lg font-bold text-on-surface">{title}</h1>
          {subtitle && <span className="text-xs text-on-surface-muted">{subtitle}</span>}
        </div>
        {actions && <div className="flex items-center gap-2">{actions}</div>}
      </div>
    </header>
  );
}
