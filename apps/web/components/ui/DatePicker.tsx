"use client";

import React from "react";

export interface DatePickerProps {
  /** If true, renders a connected Start/End range picker as used in event creation. */
  isRange?: boolean;

  // Single date/time mode props:
  label?: string;
  value?: string;
  onChange?: (date: string) => void;
  timeValue?: string;
  onTimeChange?: (time: string) => void;
  placeholder?: string;
  timePlaceholder?: string;
  name?: string;
  timeName?: string;
  hasTime?: boolean;

  // Range mode props:
  startDate?: string;
  startTime?: string;
  endDate?: string;
  endTime?: string;
  onStartDateChange?: (val: string) => void;
  onStartTimeChange?: (val: string) => void;
  onEndDateChange?: (val: string) => void;
  onEndTimeChange?: (val: string) => void;
  /** Generic field change callback for form state handlers */
  onFieldChange?: (
    e: React.ChangeEvent<HTMLInputElement>,
  ) => void;

  startDatePlaceholder?: string;
  startTimePlaceholder?: string;
  endDatePlaceholder?: string;
  endTimePlaceholder?: string;

  // General props:
  disabled?: boolean;
  className?: string;
}

const NEUBRUTALIST_INPUT_CLASS =
  "w-full bg-white border border-gray-100 rounded-xl focus-within:border-black focus-within:border-2 focus-within:shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] transition-all";

/**
 * Reusable DatePicker component.
 *
 * Supports both standalone single date/time selection and connected
 * Start/End date range selection with Agora's neubrutalist styling.
 *
 * Refactored as part of Issue #1546.
 */
export function DatePicker({
  isRange = false,
  label,
  value = "",
  onChange,
  timeValue = "",
  onTimeChange,
  placeholder = "Thu, 19 Feb",
  timePlaceholder = "08:00AM",
  name = "date",
  timeName = "time",
  hasTime = false,

  startDate = "",
  startTime = "",
  endDate = "",
  endTime = "",
  onStartDateChange,
  onStartTimeChange,
  onEndDateChange,
  onEndTimeChange,
  onFieldChange,

  startDatePlaceholder = "Thu, 19 Feb",
  startTimePlaceholder = "08:00AM",
  endDatePlaceholder = "Thu, 20 Feb",
  endTimePlaceholder = "09:00AM",

  disabled = false,
  className = "",
}: DatePickerProps) {
  // If isRange is true (or start/end props are supplied), render connected Start/End fields
  if (isRange || startDate || endDate || onStartDateChange) {
    return (
      <div
        className={`p-4 flex-1 flex flex-col gap-3 shadow-sm ${NEUBRUTALIST_INPUT_CLASS} ${className}`}
        role="group"
        aria-label="Event date and time range"
      >
        {/* Start Date & Time */}
        <div className="flex items-center gap-4">
          <span className="text-sm font-semibold w-12 flex items-center gap-2">
            <span
              className="w-2 h-2 rounded-full bg-black block"
              aria-hidden="true"
            />
            Start
          </span>
          <input
            type="text"
            name="startDate"
            value={startDate}
            disabled={disabled}
            aria-label="Start date"
            onChange={(e) => {
              onStartDateChange?.(e.target.value);
              onFieldChange?.(e);
            }}
            placeholder={startDatePlaceholder}
            className="bg-muted rounded-lg px-3 py-2 text-sm font-medium w-full outline-none focus:ring-1 focus:ring-black disabled:opacity-50"
          />
          <input
            type="text"
            name="startTime"
            value={startTime}
            disabled={disabled}
            aria-label="Start time"
            onChange={(e) => {
              onStartTimeChange?.(e.target.value);
              onFieldChange?.(e);
            }}
            placeholder={startTimePlaceholder}
            className="bg-muted rounded-lg px-3 py-2 text-sm font-medium w-32 outline-none focus:ring-1 focus:ring-black disabled:opacity-50"
          />
        </div>

        {/* End Date & Time */}
        <div className="flex items-center gap-4 relative">
          <div
            className="absolute left-1 top-[-10px] w-px h-6 bg-dashed border-l border-dashed border-gray-300"
            aria-hidden="true"
          />
          <span className="text-sm font-semibold w-12 flex items-center gap-2">
            <span
              className="w-2 h-2 rounded-full border-2 border-gray-300 block"
              aria-hidden="true"
            />
            End
          </span>
          <input
            type="text"
            name="endDate"
            value={endDate}
            disabled={disabled}
            aria-label="End date"
            onChange={(e) => {
              onEndDateChange?.(e.target.value);
              onFieldChange?.(e);
            }}
            placeholder={endDatePlaceholder}
            className="bg-muted rounded-lg px-3 py-2 text-sm font-medium w-full outline-none focus:ring-1 focus:ring-black disabled:opacity-50"
          />
          <input
            type="text"
            name="endTime"
            value={endTime}
            disabled={disabled}
            aria-label="End time"
            onChange={(e) => {
              onEndTimeChange?.(e.target.value);
              onFieldChange?.(e);
            }}
            placeholder={endTimePlaceholder}
            className="bg-muted rounded-lg px-3 py-2 text-sm font-medium w-32 outline-none focus:ring-1 focus:ring-black disabled:opacity-50"
          />
        </div>
      </div>
    );
  }

  // Single date/time mode
  return (
    <div
      className={`p-4 shadow-sm ${NEUBRUTALIST_INPUT_CLASS} ${className}`}
      role="group"
      aria-label={label || "Date and time"}
    >
      {label && <label className="block text-sm font-semibold mb-3">{label}</label>}
      <div className="flex items-center gap-4">
        <input
          type="text"
          name={name}
          value={value}
          disabled={disabled}
          onChange={(e) => {
            onChange?.(e.target.value);
            onFieldChange?.(e);
          }}
          placeholder={placeholder}
          className="bg-muted rounded-lg px-3 py-2 text-sm font-medium w-full outline-none focus:ring-1 focus:ring-black disabled:opacity-50"
        />
        {(hasTime || timeValue || onTimeChange) && (
          <input
            type="text"
            name={timeName}
            value={timeValue}
            disabled={disabled}
            onChange={(e) => {
              onTimeChange?.(e.target.value);
              onFieldChange?.(e);
            }}
            placeholder={timePlaceholder}
            className="bg-muted rounded-lg px-3 py-2 text-sm font-medium w-32 outline-none focus:ring-1 focus:ring-black disabled:opacity-50"
          />
        )}
      </div>
    </div>
  );
}

export default DatePicker;
