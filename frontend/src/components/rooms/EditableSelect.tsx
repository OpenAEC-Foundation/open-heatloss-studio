interface EditableSelectProps {
  value: string;
  onChange: (value: string) => void;
  options: Record<string, string>;
  className?: string;
}

export function EditableSelect({
  value,
  onChange,
  options,
  className = "",
}: EditableSelectProps) {
  return (
    <select
      value={value}
      onChange={(e) => onChange(e.target.value)}
      className={`w-full cursor-pointer rounded border-none bg-transparent px-1 py-0.5
        text-sm text-stone-900 outline-none hover:bg-stone-100
        focus:bg-white focus:ring-1 focus:ring-primary ${className}`}
    >
      {Object.entries(options).map(([key, label]) => (
        <option key={key} value={key}>
          {label}
        </option>
      ))}
    </select>
  );
}
