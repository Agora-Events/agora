"use client";

import { useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import Image from "next/image";
import { useState } from "react";

const CREATE_EVENT_DRAFT_KEY = "agora:draft:create-event";

// Description length constraint matches DB CHECK constraint
export const MAX_DESCRIPTION_LENGTH = 10000;

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
  /** Optional cover image */
  coverImage?: File | null;
};

function getBrowserTimezone(): string {
  try {
    return Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC";
  } catch {
    return "UTC";
  }
}

const initialFormState: EventFormData = {
  title: "",
  startDate: "",
  startTime: "",
  endDate: "",
  endTime: "",
  timezone: getBrowserTimezone(),
  location: "",
  description: "",
  visibility: "Public",
  capacity: "",
  price: "",
  coverImage: null,
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
  const [isDraftAvailable, setIsDraftAvailable] = useState(false);
  const [isDirty, setIsDirty] = useState(false);
  const [imagePreview, setImagePreview] = useState<string | null>(null);
  const [imageError, setImageError] = useState<string | null>(null);
  const [descriptionLength, setDescriptionLength] = useState(0);
  const lastPersistedAt = useRef(0);
  const persistTimeout = useRef<ReturnType<typeof setTimeout> | null>(null);
  const pendingDraft = useRef<EventFormData | null>(null);

  useEffect(() => {
    try {
      const storedDraft = window.localStorage.getItem(CREATE_EVENT_DRAFT_KEY);

      if (storedDraft) {
        const parsedDraft = JSON.parse(storedDraft) as Partial<EventFormData>;
        setFormData({ ...initialFormState, ...parsedDraft });
        setIsDraftAvailable(true);
        setDescriptionLength(parsedDraft.description?.length ?? 0);
      }
    } catch {
      // localStorage can be unavailable or contain invalid data.
    }
  }, []);

  // Cleanup object URLs on unmount
  useEffect(() => {
    return () => {
      if (imagePreview) {
        URL.revokeObjectURL(imagePreview);
      }
    };
  }, [imagePreview]);

  // Register beforeunload handler when form is dirty
  useEffect(() => {
    const handleBeforeUnload = (e: BeforeUnloadEvent) => {
      if (isDirty) {
        e.preventDefault();
        e.returnValue = "";
      }
    };

    window.addEventListener("beforeunload", handleBeforeUnload);

    return () => {
      window.removeEventListener("beforeunload", handleBeforeUnload);
    };
  }, [isDirty]);

  useEffect(
    () => () => {
      if (persistTimeout.current) {
        clearTimeout(persistTimeout.current);
      }
    },
    [],
  );

  const persistDraft = () => {
    persistTimeout.current = null;

    if (!pendingDraft.current) {
      return;
    }

    try {
      window.localStorage.setItem(
        CREATE_EVENT_DRAFT_KEY,
        JSON.stringify(pendingDraft.current),
      );
      lastPersistedAt.current = Date.now();
      pendingDraft.current = null;
    } catch {
      // Ignore storage failures, including private-browsing restrictions.
    }
  };

  const scheduleDraftPersist = (nextFormData: EventFormData) => {
    pendingDraft.current = nextFormData;
    const delay = Math.max(0, 1000 - (Date.now() - lastPersistedAt.current));

    if (delay === 0 && !persistTimeout.current) {
      persistDraft();
    } else if (!persistTimeout.current) {
      persistTimeout.current = setTimeout(persistDraft, delay);
    }

    // Track if form is dirty (differs from initial state)
    // Note: File objects are not serializable, so we exclude coverImage from the comparison
    const formDataWithoutImage = { ...nextFormData, coverImage: null };
    const initialWithoutImage = { ...initialFormState, coverImage: null };
    setIsDirty(JSON.stringify(formDataWithoutImage) !== JSON.stringify(initialWithoutImage));
  };

  const cancelPendingDraft = () => {
    if (persistTimeout.current) {
      clearTimeout(persistTimeout.current);
      persistTimeout.current = null;
    }
    pendingDraft.current = null;
    setIsDirty(false);
  };

  const clearImagePreview = () => {
    if (imagePreview) {
      URL.revokeObjectURL(imagePreview);
      setImagePreview(null);
    }
    setImageError(null);
  };

  const clearStoredDraft = () => {
    try {
      window.localStorage.removeItem(CREATE_EVENT_DRAFT_KEY);
    } catch {
      // Ignore storage failures, including private-browsing restrictions.
    }
  };

  const resetDirtyState = () => {
    setIsDirty(false);
  };

  const validateImage = (file: File): Promise<string | null> => {
    return new Promise((resolve) => {
      // Check file type
      const allowedTypes = ["image/jpeg", "image/png", "image/webp"];
      if (!allowedTypes.includes(file.type)) {
        resolve("Please select a JPEG, PNG, or WebP image.");
        return;
      }

      // Check file size (5 MB)
      const maxSize = 5 * 1024 * 1024; // 5 MB
      if (file.size > maxSize) {
        const sizeInMB = (file.size / (1024 * 1024)).toFixed(1);
        resolve(`File size is ${sizeInMB} MB. Maximum allowed is 5 MB.`);
        return;
      }

      // Check image dimensions
      const img = new Image();
      img.onload = () => {
        // Minimum dimensions: 600x400
        if (img.width < 600 || img.height < 400) {
          resolve(
            `Image dimensions are ${img.width}×${img.height}. Minimum required is 600×400 pixels.`,
          );
        } else {
          resolve(null); // Valid
        }
      };
      img.onerror = () => {
        resolve("Failed to load image. Please try a different file.");
      };
      img.src = URL.createObjectURL(file);
    });
  };

  const handleImageChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    // Revoke previous preview URL to prevent leaks
    if (imagePreview) {
      URL.revokeObjectURL(imagePreview);
      setImagePreview(null);
    }

    // Clear any previous error
    setImageError(null);

    validateImage(file).then((error) => {
      if (error) {
        setImageError(error);
        setFormData((prev) => ({ ...prev, coverImage: null }));
      } else {
        const previewUrl = URL.createObjectURL(file);
        setImagePreview(previewUrl);
        setFormData((prev) => ({ ...prev, coverImage: file }));
      }
    });
  };

  const handleRemoveImage = () => {
    if (imagePreview) {
      URL.revokeObjectURL(imagePreview);
    }
    setImagePreview(null);
    setImageError(null);
    setFormData((prev) => ({ ...prev, coverImage: null }));
  };

  const handleRestore = () => {
    try {
      const storedDraft = window.localStorage.getItem(CREATE_EVENT_DRAFT_KEY);

      if (storedDraft) {
        const parsedDraft = JSON.parse(storedDraft) as Partial<EventFormData>;
        setFormData({
          ...initialFormState,
          ...parsedDraft,
        });
      }
    } catch {
      // Ignore storage failures or invalid draft data.
    }

    setIsDraftAvailable(false);
    resetDirtyState();
    clearImagePreview();
    setDescriptionLength(0);
  };

  const handleDiscard = () => {
    cancelPendingDraft();
    clearStoredDraft();
    setIsDraftAvailable(false);
    resetDirtyState();
    clearImagePreview();
    setDescriptionLength(0);
  };

  const handleChange = (
    e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>,
  ) => {
    const { name, value } = e.target;
    let nextFormData: EventFormData;

    if (name === "capacity") {
      const numericValue = value.replace(/[^0-9]/g, "");
      nextFormData = { ...formData, [name]: numericValue };
    } else if (name === "price") {
      const decimalValue = value.replace(/[^0-9.]/g, "");
      nextFormData = { ...formData, [name]: decimalValue };
    } else if (name === "description") {
      // Limit to MAX_DESCRIPTION_LENGTH to match DB constraint
      const limitedValue = value.slice(0, MAX_DESCRIPTION_LENGTH);
      nextFormData = { ...formData, [name]: limitedValue };
      setDescriptionLength(limitedValue.length);
    } else {
      nextFormData = { ...formData, [name]: value };
    }

    setFormData(nextFormData);
    scheduleDraftPersist(nextFormData);
  };

  const handleVisibilityChange = (visibility: "Public" | "Private") => {
    const nextFormData = { ...formData, visibility };
    setFormData(nextFormData);
    scheduleDraftPersist(nextFormData);
  };

  const handleClear = () => {
    cancelPendingDraft();
    setFormData(initialFormState);
    clearStoredDraft();
    resetDirtyState();
    clearImagePreview();
    setDescriptionLength(0);
  };

  const handleSubmit = () => {
    console.log("Submitting Event Data:", formData);
    cancelPendingDraft();
    clearStoredDraft();
    resetDirtyState();
    clearImagePreview();
    setDescriptionLength(0);
  };

  const isSubmitDisabled = !formData.title.trim() || !formData.startDate.trim();

  // Common Neubrutalist class for form controls
  const neubrutalistInputClass =
    "w-full bg-white border border-gray-100 rounded-xl focus-within:border-black focus-within:border-2 focus-within:shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] transition-all";

  return (
    <div className="flex flex-col gap-6 w-full">
      {isDraftAvailable && (
        <div
          className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 border border-black rounded-xl bg-[#FFFBEA] p-4 shadow-[-3px_3px_0px_0px_rgba(0,0,0,1)]"
          role="status"
        >
          <span className="font-semibold">Restore your unsaved draft?</span>
          <div className="flex gap-2">
            <Button
              onClick={handleRestore}
              backgroundColor="bg-black"
              textColor="text-white"
              shadowColor="transparent"
              className="px-4"
            >
              Restore
            </Button>
            <Button
              onClick={handleDiscard}
              backgroundColor="bg-white"
              textColor="text-black"
              shadowColor="transparent"
              className="px-4 border border-black"
            >
              Discard
            </Button>
          </div>
        </div>
      )}

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

      {/* Cover Image Section */}
      <div className={`p-6 shadow-sm ${neubrutalistInputClass}`}>
        <label className="block text-sm font-semibold mb-3">Cover Image</label>
        
        {imagePreview ? (
          <div className="flex flex-col gap-3">
            <div className="relative rounded-xl overflow-hidden border border-gray-100">
              <img
                src={imagePreview}
                alt="Cover preview"
                className="w-full h-48 object-cover"
              />
              <button
                type="button"
                onClick={handleRemoveImage}
                className="absolute top-2 right-2 bg-black/70 text-white px-3 py-1 rounded-lg text-sm font-semibold hover:bg-black/90 transition-colors"
              >
                Remove
              </button>
            </div>
            {imageError && (
              <div className="text-red-600 text-sm font-medium bg-red-50 border border-red-100 rounded-lg px-3 py-2">
                {imageError}
              </div>
            )}
          </div>
        ) : (
          <div className="flex flex-col gap-2">
            <label className="flex items-center justify-center w-full h-48 border-2 border-dashed border-gray-300 rounded-xl hover:border-black hover:bg-gray-50 transition-all cursor-pointer">
              <div className="flex flex-col items-center gap-2 text-center px-4">
                <div className="w-10 h-10 bg-black/5 rounded-full flex items-center justify-center">
                  <svg
                    className="w-6 h-6 text-black"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"
                    />
                  </svg>
                </div>
                <div className="text-sm">
                  <span className="font-semibold text-black">Click to upload</span>
                  <span className="text-gray-500"> or drag and drop</span>
                </div>
                <div className="text-xs text-gray-400 mt-1">
                  JPEG, PNG, or WebP (max 5 MB, 600×400 min)
                </div>
              </div>
              <input
                type="file"
                accept="image/jpeg,image/png,image/webp"
                onChange={handleImageChange}
                className="hidden"
                aria-label="Upload cover image"
              />
            </label>
            {imageError && (
              <div className="text-red-600 text-sm font-medium bg-red-50 border border-red-100 rounded-lg px-3 py-2">
                {imageError}
              </div>
            )}
          </div>
        )}
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
              className="bg-muted rounded-lg px-3 py-2 text-sm font-medium w-full outline-none focus:ring-1 focus:ring-black"
            />
            <input
              type="text"
              name="startTime"
              value={formData.startTime}
              onChange={handleChange}
              placeholder="08:00AM"
              className="bg-muted rounded-lg px-3 py-2 text-sm font-medium w-32 outline-none focus:ring-1 focus:ring-black"
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
              className="bg-muted rounded-lg px-3 py-2 text-sm font-medium w-full outline-none focus:ring-1 focus:ring-black"
            />
            <input
              type="text"
              name="endTime"
              value={formData.endTime}
              onChange={handleChange}
              placeholder="09:00AM"
              className="bg-muted rounded-lg px-3 py-2 text-sm font-medium w-32 outline-none focus:ring-1 focus:ring-black"
            />
          </div>
        </div>

        <div className="bg-base-alt rounded-xl p-4 shadow-sm w-full sm:w-auto min-w-[140px] flex items-center justify-between gap-4 border border-black shadow-[-2px_2px_0px_0px_rgba(0,0,0,1)]">
          <div className="flex flex-col">
            <span className="text-sm font-semibold">{formData.timezone}</span>
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
              className={`w-10 h-10 rounded-full flex items-center justify-center transition-colors ${locationMode === "Virtual" ? "bg-black" : "bg-muted hover:bg-gray-100"}`}
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
              className={`w-10 h-10 rounded-full flex items-center justify-center transition-colors ${locationMode === "Physical" ? "bg-black" : "bg-muted hover:bg-gray-100"}`}
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
            id="description-input"
            name="description"
            value={formData.description}
            onChange={handleChange}
            onInput={(e) => {
              const target = e.target as HTMLTextAreaElement;
              target.style.height = "auto";
              target.style.height = `${target.scrollHeight}px`;
            }}
            placeholder="Add Description about this Event..."
            className={`flex-1 text-base font-medium bg-transparent outline-none placeholder:text-gray-300 resize-none overflow-hidden min-h-[80px] ${
              descriptionLength >= MAX_DESCRIPTION_LENGTH
                ? "text-red-600"
                : ""
            }`}
            aria-describedby="description-counter"
            maxLength={MAX_DESCRIPTION_LENGTH}
          />
          <div className="w-10 h-10 rounded-full bg-muted flex items-center justify-center shrink-0 mt-1">
            <Image
              src="/icons/edit.svg"
              width={20}
              height={20}
              alt="Edit"
              className="opacity-60"
            />
          </div>
        </div>
        <div
          id="description-counter"
          className={`mt-2 text-sm font-medium flex justify-end ${
            descriptionLength >= MAX_DESCRIPTION_LENGTH
              ? "text-red-600"
              : descriptionLength >= Math.round(MAX_DESCRIPTION_LENGTH * 0.9)
              ? "text-amber-600"
              : "text-gray-500"
          }`}
          aria-live="polite"
        >
          {descriptionLength} / {MAX_DESCRIPTION_LENGTH}
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
            <div className="flex bg-muted p-1 rounded-xl w-full">
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
              <div className="w-10 h-10 bg-base-alt border border-black rounded-lg shadow-[-2px_2px_0px_0px_rgba(0,0,0,1)] flex items-center justify-center shrink-0">
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
            <div className="w-10 h-10 bg-base-alt border border-black rounded-lg shadow-[-2px_2px_0px_0px_rgba(0,0,0,1)] flex items-center justify-center shrink-0">
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
          variant="secondary"
          onClick={handleClear}
          className="w-full sm:w-auto"
        >
          Clear Event
        </Button>
        <Button
          variant="primary"
          disabled={isSubmitDisabled}
          onClick={handleSubmit}
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
