import type { HTMLAttributes, ReactNode } from "react";

interface CardProps extends HTMLAttributes<HTMLDivElement> {
  title?: string;
  children: ReactNode;
}

export function Card({ title, children, className = "", ...props }: CardProps) {
  return (
    <div
      className={`rounded-lg border border-[var(--oaec-border-subtle)] bg-[var(--oaec-bg-lighter)] shadow-sm transition-shadow hover:shadow-md ${className}`}
      {...props}
    >
      {title && (
        <div className="border-b border-[var(--oaec-border-subtle)] px-5 py-3">
          <h3 className="font-heading text-sm font-medium text-on-surface">{title}</h3>
        </div>
      )}
      <div className="p-5">{children}</div>
    </div>
  );
}
