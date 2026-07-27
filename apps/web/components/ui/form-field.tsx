import React from "react";

interface FormFieldProps {
  label: string;
  name: string;
  type: string;
  value: string;
  onChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
  error?: string;
  placeholder?: string;
}

export function FormField({
  label,
  name,
  type,
  value,
  onChange,
  error,
  placeholder,
}: FormFieldProps) {
  return (
    <div className="flex flex-col w-full">
      <label htmlFor={name} className="text-sm font-medium mb-2 text-black">
        {label}
      </label>
      <input
        id={name}
        name={name}
        type={type}
        value={value}
        onChange={onChange}
        placeholder={placeholder}
        className="w-full bg-white border-2 border-black rounded-full px-4 py-2 outline-none shadow-[4px_4px_0px_0px_#000] focus:shadow-[2px_2px_0px_0px_#000] transition-shadow"
      />
      {error && (
        <span role="alert" className="text-xs text-red-500 mt-1">
          {error}
        </span>
      )}
    </div>
  );
}
