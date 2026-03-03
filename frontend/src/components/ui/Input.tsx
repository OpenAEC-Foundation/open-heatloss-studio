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
        <label htmlFor={id} className="text-xs font-medium text-stone-600">
          {label}
        </label>
      )}
      <div className="relative">
        <input
          ref={ref}
          id={id}
          className={`w-full rounded-md border bg-white px-3 py-1.5 text-sm
            transition-colors placeholder:text-stone-400
            focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary
            disabled:bg-stone-100 disabled:text-stone-400
            ${unit ? "pr-12" : ""}
            ${error ? "border-red-400" : "border-stone-300"}
            ${props.type === "number" ? "font-mono text-right" : ""}
            ${className}`}
          {...props}
        />
        {unit && <span className="input-unit">{unit}</span>}
      </div>
      {error && <p className="text-xs text-red-600">{error}</p>}
    </div>
  ),
);

Input.displayName = "Input";
