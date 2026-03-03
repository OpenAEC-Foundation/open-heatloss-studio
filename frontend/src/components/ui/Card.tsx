import type { HTMLAttributes, ReactNode } from "react";

interface CardProps extends HTMLAttributes<HTMLDivElement> {
  title?: string;
  children: ReactNode;
}

export function Card({ title, children, className = "", ...props }: CardProps) {
  return (
    <div
      className={`rounded-lg border border-stone-200 bg-white shadow-sm ${className}`}
      {...props}
    >
      {title && (
        <div className="border-b border-stone-200 px-4 py-2.5">
          <h3 className="font-heading text-sm font-medium text-stone-800">{title}</h3>
        </div>
      )}
      <div className="p-4">{children}</div>
    </div>
  );
}
