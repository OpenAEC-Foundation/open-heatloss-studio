import { type SelectHTMLAttributes, forwardRef } from "react";

interface SelectOption {
  value: string;
  label: string;
}

interface SelectProps extends SelectHTMLAttributes<HTMLSelectElement> {
  label?: string;
  options: SelectOption[];
  error?: string;
}

export const Select = forwardRef<HTMLSelectElement, SelectProps>(
  ({ label, options, error, id, className = "", ...props }, ref) => (
    <div className="flex flex-col gap-1">
      {label && (
        <label htmlFor={id} className="text-xs font-medium text-on-surface-secondary">
          {label}
        </label>
      )}
      <select
        ref={ref}
        id={id}
        className={`w-full rounded-md border-[1.5px] bg-[var(--oaec-bg-input)] px-3 py-2 text-sm text-on-surface
          transition-colors
          focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/20
          disabled:opacity-50 disabled:cursor-not-allowed
          ${error ? "border-red-400" : "border-[var(--oaec-border)]"}
          ${className}`}
        {...props}
      >
        {options.map((opt) => (
          <option key={opt.value} value={opt.value}>
            {opt.label}
          </option>
        ))}
      </select>
      {error && <p className="text-xs text-red-400">{error}</p>}
    </div>
  ),
);

Select.displayName = "Select";
