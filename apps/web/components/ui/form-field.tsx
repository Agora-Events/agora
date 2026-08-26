import React from "react";

export interface FormFieldProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label: string;
  name: string;
  type?: string;
  value?: string;
  onChange?: (e: React.ChangeEvent<HTMLInputElement>) => void;
  error?: string;
  placeholder?: string;
  disabled?: boolean;
  required?: boolean;
}

export function FormField({
  label,
  name,
  type = "text",
  value = "",
  onChange,
  error,
  placeholder,
  disabled = false,
  required = false,
  className = "",
  ...props
}: FormFieldProps) {
  return (
    <div className="flex flex-col w-full">
      <label htmlFor={name} className="text-sm font-medium mb-2 text-black">
        {label}
        {required && <span className="text-red-500 ml-1">*</span>}
      </label>
      <input
        id={name}
        name={name}
        type={type}
        value={value}
        onChange={onChange}
        placeholder={placeholder}
        disabled={disabled}
        required={required}
        className={`w-full bg-white border-2 border-black rounded-full px-4 py-2 outline-none shadow-[4px_4px_0px_0px_#000] focus:shadow-[2px_2px_0px_0px_#000] transition-shadow disabled:opacity-50 disabled:cursor-not-allowed ${className}`}
        {...props}
      />
      {error && (
        <span role="alert" className="text-xs text-red-500 mt-1">
          {error}
        </span>
      )}
    </div>
  );
}
