import { type InputHTMLAttributes, forwardRef } from "react";

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  unit?: string;
  error?: string;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ label, unit, error, id, className = "", ...props }, ref) => (
    <div className="flex flex-col gap-1">
      {label && (
        <label htmlFor={id} className="text-xs font-medium text-on-surface-secondary">
          {label}
        </label>
      )}
      <div className="relative">
        <input
          ref={ref}
          id={id}
          className={`w-full rounded-md border-[1.5px] bg-[var(--oaec-bg-input)] px-3 py-2 text-sm text-on-surface
            transition-colors placeholder:text-on-surface-muted
            focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/20
            disabled:opacity-50 disabled:cursor-not-allowed
            ${unit ? "pr-12" : ""}
            ${error ? "border-red-400" : "border-[var(--oaec-border)]"}
            ${props.type === "number" ? "font-mono text-right" : ""}
            ${className}`}
          {...props}
        />
        {unit && <span className="input-unit">{unit}</span>}
      </div>
      {error && <p className="text-xs text-red-400">{error}</p>}
    </div>
  ),
);

Input.displayName = "Input";
