"use client";

import { Button } from "@/components/ui/button";
import Image from "next/image";
import { useState } from "react";

/**
 * Form data structure for creating a new event
 * @interface EventFormData
 */
export type EventFormData = {
  /** Title of the event */
  title: string;
  /** Start date in YYYY-MM-DD format */
  startDate: string;
  /** Start time in HH:MM format */
  startTime: string;
  /** End date in YYYY-MM-DD format */
  endDate: string;
  /** End time in HH:MM format */
  endTime: string;
  /** Timezone identifier (e.g., "GMT+00:00 UTC") */
  timezone: string;
  /** Physical or virtual location */
  location: string;
  /** Detailed description of the event */
  description: string;
  /** Event visibility setting */
  visibility: "Public" | "Private";
  /** Maximum number of attendees */
  capacity: string;
  /** Ticket price (empty string for free events) */
  price: string;
};

const initialFormState: EventFormData = {
  title: "",
  startDate: "",
  startTime: "",
  endDate: "",
  endTime: "",
  timezone: "GMT+00:00 UTC",
  location: "",
  description: "",
  visibility: "Public",
  capacity: "",
  price: "",
};

/**
 * CreateEventForm component for creating new events
 *
 * @returns React component that renders a form for creating events
 */
export default function CreateEventForm() {
  const [formData, setFormData] = useState<EventFormData>(initialFormState);
  const [locationMode, setLocationMode] = useState<"Virtual" | "Physical">(
    "Physical",
  );

  const handleChange = (
    e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>,
  ) => {
    const { name, value } = e.target;

    if (name === "capacity") {
      const numericValue = value.replace(/[^0-9]/g, "");
      setFormData((prev) => ({ ...prev, [name]: numericValue }));
      return;
    }

    if (name === "price") {
      const decimalValue = value.replace(/[^0-9.]/g, "");
      setFormData((prev) => ({ ...prev, [name]: decimalValue }));
      return;
    }

    setFormData((prev) => ({ ...prev, [name]: value }));
  };

  const handleVisibilityChange = (visibility: "Public" | "Private") => {
    setFormData((prev) => ({ ...prev, visibility }));
  };

  const handleClear = () => {
    setFormData(initialFormState);
  };

  const handleSubmit = () => {
    console.log("Submitting Event Data:", formData);
  };

  const isSubmitDisabled = !formData.title.trim() || !formData.startDate.trim();

  // Common Neubrutalist class for form controls
  const neubrutalistInputClass =
    "w-full bg-white border border-gray-100 rounded-xl focus-within:border-black focus-within:border-2 focus-within:shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] transition-all";

  return (
    <div className="flex flex-col gap-6 w-full">
      {/* Event Title Section */}
      <div className={`p-6 shadow-sm ${neubrutalistInputClass}`}>
        <label className="block text-sm font-semibold mb-3">Event Title</label>
        <input
          type="text"
          name="title"
          value={formData.title}
          onChange={handleChange}
          placeholder="Event Name"
          className="w-full text-3xl font-bold bg-transparent border-none outline-none placeholder:text-gray-300"
        />
      </div>

      {/* Date & Time Section */}
      <div className="flex flex-col sm:flex-row gap-4">
        <div
          className={`p-4 flex-1 flex flex-col gap-3 shadow-sm ${neubrutalistInputClass}`}
        >
          <div className="flex items-center gap-4">
            <span className="text-sm font-semibold w-12 flex items-center gap-2">
              <span className="w-2 h-2 rounded-full bg-black block"></span>
              Start
            </span>
            <input
              type="text"
              name="startDate"
              value={formData.startDate}
              onChange={handleChange}
              placeholder="Thu, 19 Feb"
              className="bg-[#FAF9F6] rounded-lg px-3 py-2 text-sm font-medium w-full outline-none focus:ring-1 focus:ring-black"
            />
            <input
              type="text"
              name="startTime"
              value={formData.startTime}
              onChange={handleChange}
              placeholder="08:00AM"
              className="bg-[#FAF9F6] rounded-lg px-3 py-2 text-sm font-medium w-32 outline-none focus:ring-1 focus:ring-black"
            />
          </div>
          <div className="flex items-center gap-4 relative">
            <div className="absolute left-1 top-[-10px] w-px h-6 bg-dashed border-l border-dashed border-gray-300"></div>
            <span className="text-sm font-semibold w-12 flex items-center gap-2">
              <span className="w-2 h-2 rounded-full border-2 border-gray-300 block"></span>
              End
            </span>
            <input
              type="text"
              name="endDate"
              value={formData.endDate}
              onChange={handleChange}
              placeholder="Thu, 20 Feb"
              className="bg-[#FAF9F6] rounded-lg px-3 py-2 text-sm font-medium w-full outline-none focus:ring-1 focus:ring-black"
            />
            <input
              type="text"
              name="endTime"
              value={formData.endTime}
              onChange={handleChange}
              placeholder="09:00AM"
              className="bg-[#FAF9F6] rounded-lg px-3 py-2 text-sm font-medium w-32 outline-none focus:ring-1 focus:ring-black"
            />
          </div>
        </div>

        <div className="bg-[#FFFBEA] rounded-xl p-4 shadow-sm w-full sm:w-auto min-w-[140px] flex items-center justify-between gap-4 border border-black shadow-[-2px_2px_0px_0px_rgba(0,0,0,1)]">
          <div className="flex flex-col">
            <span className="text-sm font-semibold">GMT+00:00</span>
            <span className="text-xs text-gray-500">UTC</span>
          </div>
          <div className="w-10 h-10 rounded-full bg-white flex items-center justify-center border border-black shadow-sm">
            <Image src="/icons/global.svg" width={20} height={20} alt="Globe" />
          </div>
        </div>
      </div>

      {/* Location Section */}
      <div className={`p-4 shadow-sm ${neubrutalistInputClass}`}>
        <label className="block text-sm font-semibold mb-3">
          Add Event Location
        </label>
        <div className="flex items-center gap-4">
          <input
            type="text"
            name="location"
            value={formData.location}
            onChange={handleChange}
            placeholder={
              locationMode === "Virtual"
                ? "Virtual meeting link"
                : "Offline location or map pin"
            }
            className="flex-1 text-base font-medium bg-transparent outline-none placeholder:text-gray-300"
          />
          <div className="flex gap-2">
            <button
              type="button"
              onClick={() => setLocationMode("Virtual")}
              className={`w-10 h-10 rounded-full flex items-center justify-center transition-colors ${locationMode === "Virtual" ? "bg-black" : "bg-[#FAF9F6] hover:bg-gray-100"}`}
            >
              <Image
                src="/icons/video.svg"
                width={20}
                height={20}
                alt="Video"
                className={
                  locationMode === "Virtual"
                    ? "invert brightness-0"
                    : "opacity-60"
                }
              />
            </button>
            <button
              type="button"
              onClick={() => setLocationMode("Physical")}
              className={`w-10 h-10 rounded-full flex items-center justify-center transition-colors ${locationMode === "Physical" ? "bg-black" : "bg-[#FAF9F6] hover:bg-gray-100"}`}
            >
              <Image
                src="/icons/location.svg"
                width={20}
                height={20}
                alt="Map"
                className={
                  locationMode === "Physical"
                    ? "invert brightness-0"
                    : "opacity-60"
                }
              />
            </button>
          </div>
        </div>
      </div>

      {/* Description Section */}
      <div
        className={`p-4 shadow-sm min-h-[140px] flex flex-col ${neubrutalistInputClass}`}
      >
        <label className="block text-sm font-semibold mb-3">
          Add Description
        </label>
        <div className="flex items-start gap-4 flex-1">
          <textarea
            name="description"
            value={formData.description}
            onChange={handleChange}
            onInput={(e) => {
              const target = e.target as HTMLTextAreaElement;
              target.style.height = "auto";
              target.style.height = `${target.scrollHeight}px`;
            }}
            placeholder="Add Description about this Event..."
            className="flex-1 text-base font-medium bg-transparent outline-none placeholder:text-gray-300 resize-none overflow-hidden min-h-[80px]"
          />
          <div className="w-10 h-10 rounded-full bg-[#FAF9F6] flex items-center justify-center shrink-0 mt-1">
            <Image
              src="/icons/edit.svg"
              width={20}
              height={20}
              alt="Edit"
              className="opacity-60"
            />
          </div>
        </div>
      </div>

      {/* Event Options Section */}
      <div className="mt-4">
        <h3 className="text-lg font-bold mb-4">Event Options</h3>

        <div className="flex flex-col md:flex-row gap-4 mb-4">
          {/* Visibility */}
          <div className="bg-white rounded-xl p-4 shadow-sm flex-1 border border-gray-100">
            <label className="block text-sm font-semibold mb-3">
              Event Visibility
            </label>
            <div className="flex bg-[#FAF9F6] p-1 rounded-xl w-full">
              <button
                type="button"
                onClick={() => handleVisibilityChange("Public")}
                className={`flex-1 flex items-center justify-center gap-2 py-3 rounded-lg font-semibold transition-all ${
                  formData.visibility === "Public"
                    ? "bg-white shadow-sm text-black border border-gray-100"
                    : "text-gray-500 hover:text-black"
                }`}
              >
                Public
                <div
                  className={`w-6 h-6 rounded-full flex items-center justify-center ${formData.visibility === "Public" ? "bg-white border border-gray-100 shadow-sm" : "bg-white border border-gray-100 shadow-sm"} `}
                >
                  <Image
                    src="/icons/megaphone.svg"
                    width={14}
                    height={14}
                    alt="Megaphone"
                    className={
                      formData.visibility === "Public" ? "" : "opacity-40"
                    }
                  />
                </div>
              </button>
              <button
                type="button"
                onClick={() => handleVisibilityChange("Private")}
                className={`flex-1 flex items-center justify-center gap-2 py-3 rounded-lg font-semibold transition-all ${
                  formData.visibility === "Private"
                    ? "bg-white shadow-sm text-black border border-gray-100"
                    : "text-gray-500 hover:text-black"
                }`}
              >
                Private
                <div
                  className={`w-6 h-6 rounded-full flex items-center justify-center ${formData.visibility === "Private" ? "bg-white border border-gray-100 shadow-sm" : "bg-white border border-gray-100 shadow-sm"} `}
                >
                  <Image
                    src="/icons/lock.svg"
                    width={12}
                    height={12}
                    alt="Lock"
                    className={
                      formData.visibility === "Private" ? "" : "opacity-40"
                    }
                  />
                </div>
              </button>
            </div>
          </div>

          {/* Capacity */}
          <div className={`p-4 shadow-sm flex-1 ${neubrutalistInputClass}`}>
            <label className="block text-sm font-semibold mb-3">
              Set Capacity
            </label>
            <div className="flex items-center justify-between">
              <input
                type="text"
                name="capacity"
                value={formData.capacity}
                onChange={handleChange}
                placeholder="Unlimited"
                className="w-full text-base font-medium bg-transparent outline-none placeholder:text-gray-300"
              />
              <div className="w-10 h-10 bg-[#FFFBEA] border border-black rounded-lg shadow-[-2px_2px_0px_0px_rgba(0,0,0,1)] flex items-center justify-center shrink-0">
                <Image
                  src="/icons/edit.svg"
                  width={20}
                  height={20}
                  alt="Edit"
                />
              </div>
            </div>
          </div>
        </div>

        {/* Ticket Price */}
        <div
          className={`p-4 shadow-sm w-full md:w-[calc(50%-8px)] ${neubrutalistInputClass}`}
        >
          <label className="block text-sm font-semibold mb-3">
            Ticket Price
          </label>
          <div className="flex items-center justify-between">
            <input
              type="text"
              name="price"
              value={formData.price}
              onChange={handleChange}
              placeholder="Free"
              className="w-full text-base font-medium bg-transparent outline-none placeholder:text-gray-300"
            />
            <div className="w-10 h-10 bg-[#FFFBEA] border border-black rounded-lg shadow-[-2px_2px_0px_0px_rgba(0,0,0,1)] flex items-center justify-center shrink-0">
              <Image
                src="/icons/ticket.svg"
                width={20}
                height={20}
                alt="Ticket"
              />
            </div>
          </div>
        </div>
      </div>

      {/* Action Buttons */}
      <div className="flex flex-col sm:flex-row justify-end items-center gap-4 mt-6 mb-8">
        <Button
          onClick={handleClear}
          backgroundColor="bg-white"
          textColor="text-black"
          shadowColor="transparent"
          className="w-full sm:w-auto hover:bg-gray-50 border border-transparent"
        >
          Clear Event
        </Button>
        <Button
          disabled={isSubmitDisabled}
          onClick={handleSubmit}
          backgroundColor="bg-[#FFD233]"
          textColor="text-black"
          className={`w-full sm:w-auto ${
            isSubmitDisabled
              ? "opacity-50 cursor-not-allowed hover:translate-x-0 hover:translate-y-0 active:translate-x-0 active:translate-y-0"
              : ""
          }`}
        >
          Create Event <span className="ml-1 text-lg">↗</span>
        </Button>
      </div>
    </div>
  );
}
