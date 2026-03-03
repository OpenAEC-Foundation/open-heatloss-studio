import { type ButtonHTMLAttributes, forwardRef } from "react";

const VARIANTS = {
  primary:
    "bg-primary text-white hover:bg-primary-hover focus-visible:ring-primary",
  secondary:
    "border border-stone-300 bg-white text-stone-700 hover:bg-stone-50 focus-visible:ring-stone-400",
  ghost:
    "text-stone-600 hover:bg-stone-100 hover:text-stone-900 focus-visible:ring-stone-400",
  danger:
    "bg-red-600 text-white hover:bg-red-700 focus-visible:ring-red-500",
} as const;

const SIZES = {
  sm: "px-2.5 py-1 text-xs",
  md: "px-3.5 py-1.5 text-sm",
  lg: "px-5 py-2 text-base",
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
