import { type ButtonHTMLAttributes, forwardRef } from "react";

const VARIANTS = {
  primary:
    "bg-primary text-on-accent hover:bg-primary-hover focus-visible:ring-primary",
  secondary:
    "border-2 border-[var(--oaec-border)] bg-[var(--oaec-hover)] text-on-surface hover:bg-[var(--oaec-hover-strong)] hover:text-on-surface focus-visible:ring-primary",
  ghost:
    "border-2 border-primary text-primary hover:bg-primary hover:text-on-accent focus-visible:ring-primary",
  danger:
    "bg-red-600/20 text-red-400 border border-red-600/30 hover:bg-red-600/30 focus-visible:ring-red-500",
} as const;

const SIZES = {
  sm: "px-3 py-1.5 text-xs",
  md: "px-4 py-2 text-sm",
  lg: "px-5 py-2.5 text-base",
} as const;

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: keyof typeof VARIANTS;
  size?: keyof typeof SIZES;
}

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  ({ variant = "primary", size = "md", className = "", children, ...props }, ref) => (
    <button
      ref={ref}
      className={`inline-flex items-center justify-center gap-1.5 rounded-md font-medium
        transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-1
        disabled:pointer-events-none disabled:opacity-50
        ${VARIANTS[variant]} ${SIZES[size]} ${className}`}
      {...props}
    >
      {children}
    </button>
  ),
);

Button.displayName = "Button";
