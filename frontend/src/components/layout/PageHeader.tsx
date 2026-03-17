import type { ReactNode } from "react";
import { Link } from "react-router-dom";

interface Breadcrumb {
  label: string;
  to?: string;
}

interface PageHeaderProps {
  title: string;
  subtitle?: string;
  actions?: ReactNode;
  breadcrumbs?: Breadcrumb[];
}

export function PageHeader({ title, subtitle, actions, breadcrumbs }: PageHeaderProps) {
  return (
    <header className="sticky top-0 z-20 border-b border-stone-200 bg-white">
      {/* Breadcrumbs */}
      {breadcrumbs && breadcrumbs.length > 0 && (
        <div className="flex items-center gap-1.5 border-b border-stone-100 px-6 py-1.5 text-xs text-scaffold-gray">
          {breadcrumbs.map((crumb, i) => (
            <span key={crumb.label} className="flex items-center gap-1.5">
              {i > 0 && (
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
                  <polyline points="9 18 15 12 9 6" />
                </svg>
              )}
              {crumb.to ? (
                <Link to={crumb.to} className="transition-colors hover:text-primary">
                  {crumb.label}
                </Link>
              ) : (
                <span className="text-stone-500">{crumb.label}</span>
              )}
            </span>
          ))}
        </div>
      )}

      {/* Title bar */}
      <div className="flex h-header items-center justify-between px-6">
        <div className="flex items-baseline gap-3">
          <h1 className="font-heading text-lg font-bold text-stone-900">{title}</h1>
          {subtitle && <span className="text-xs text-stone-400">{subtitle}</span>}
        </div>
        {actions && <div className="flex items-center gap-2">{actions}</div>}
      </div>
    </header>
  );
}
