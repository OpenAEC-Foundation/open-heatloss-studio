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
        <label htmlFor={id} className="text-xs font-medium text-stone-600">
          {label}
        </label>
      )}
      <select
        ref={ref}
        id={id}
        className={`w-full rounded-md border bg-white px-3 py-1.5 text-sm
          transition-colors
          focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary
          disabled:bg-stone-100 disabled:text-stone-400
          ${error ? "border-red-400" : "border-stone-300"}
          ${className}`}
        {...props}
      >
        {options.map((opt) => (
          <option key={opt.value} value={opt.value}>
            {opt.label}
          </option>
        ))}
      </select>
      {error && <p className="text-xs text-red-600">{error}</p>}
    </div>
  ),
);

Select.displayName = "Select";
