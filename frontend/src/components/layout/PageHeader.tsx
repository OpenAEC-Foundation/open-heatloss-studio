import type { ReactNode } from "react";

interface PageHeaderProps {
  title: string;
  subtitle?: string;
  actions?: ReactNode;
}

export function PageHeader({ title, subtitle, actions }: PageHeaderProps) {
  return (
    <header className="flex h-header items-center justify-between border-b border-stone-200 bg-white px-6">
      <div className="flex items-baseline gap-3">
        <h1 className="font-heading text-lg font-bold text-stone-900">{title}</h1>
        {subtitle && <span className="text-xs text-stone-400">{subtitle}</span>}
      </div>
      {actions && <div className="flex items-center gap-2">{actions}</div>}
    </header>
  );
}
