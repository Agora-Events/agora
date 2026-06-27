"use client";

import React, { useState, useEffect } from "react";
import Image from "next/image";
import { useRouter } from "next/navigation";
import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";
import { Button } from "@/components/ui/button";
import { toast } from "sonner";
import { createEventSchema, CreateEventInput } from "@/lib/validation";
import { useIsMounted } from "@/hooks/useIsMounted";
import { z } from "zod";

const eventSchema = z.object({
  title: z.string().min(1, "Event title is required"),
  startDate: z.string().min(1, "Start date is required"),
  startTime: z.string().min(1, "Start time is required"),
  location: z.string().min(1, "Location is required"),
  price: z.string().min(1, "Price is required (put 0 for free)"),
  endDate: z.string().optional(),
  endTime: z.string().optional(),
  description: z.string().optional(),
  capacity: z.string().optional(),
  visibility: z.enum(["Public", "Private"]),
});

export default function CreateEventPage() {
  const router = useRouter();
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [organizer, setOrganizer] = useState<{
    name: string;
    wallet: string;
  } | null>(null);
  const isMounted = useIsMounted();

  useEffect(() => {
    fetch("/api/profile")
      .then((r) => (r.ok ? r.json() : null))
      .then((data) => {
        if (data?.profile) {
          setOrganizer({
            name: data.profile.displayName,
            wallet: data.profile.address,
          });
        }
      })
      .catch(() => null);
  }, []);

  // Form State
  const [formData, setFormData] = useState({
    title: "",
    startDate: "",
    startTime: "",
    endDate: "",
    endTime: "",
    location: "",
    description: "",
    capacity: "",
    price: "",
    imageUrl: "",
    visibility: "Public" as "Public" | "Private",
  });
  const [imageError, setImageError] = useState(false);

  const isValidUrl = (url: string) => {
    try {
      new URL(url);
      return true;
    } catch {
      return false;
    }
  };

  // Error State
  const [errors, setErrors] = useState<Record<string, string>>({});

  const getStartsAt = (data: CreateEventInput) => {
    const startsAt = new Date(`${data.startDate}T${data.startTime}`);
    return startsAt.toISOString();
  };

  const handleChange = (
    e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>,
  ) => {
    const { name, value } = e.target;
    setFormData((prev) => ({ ...prev, [name]: value }));
    // Clear error when user starts typing
    if (errors[name]) {
      setErrors((prev) => {
        const newErrors = { ...prev };
        delete newErrors[name];
        return newErrors;
      });
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    const result = eventSchema.safeParse(formData);
    if (!result.success) {
      const fieldErrors: Record<string, string> = {};
      for (const issue of result.error.issues) {
        const field = issue.path[0] as string;
        fieldErrors[field] = issue.message;
      }
      setErrors(fieldErrors);
      toast.error("Please fill in all required fields");
      return;
    }
    setErrors({});

    setErrors({});
    setIsSubmitting(true);
    try {
      const values = result.data;
      const response = await fetch("/api/events", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          title: formData.title,
          startsAt: `${formData.startDate}T${formData.startTime}:00.000Z`,
          location: formData.location,
          category: "Tech",
          organizerName: organizer?.name ?? "Agora Organizer",
          organizerWallet:
            organizer?.wallet ??
            "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
          description: formData.description,
          ticketPrice: parseFloat(formData.price) || 0,
          totalTickets: parseInt(formData.capacity) || 100,
          followersOnly: formData.visibility === "Private",
        }),
      });

      const data = await response.json();
      if (!response.ok) throw new Error(data.error || "Failed to create event");

      toast.success("Event created successfully!");
      router.push(`/events/${data.event.id}`);
    } catch (error: unknown) {
      toast.error(
        error instanceof Error ? error.message : "Something went wrong",
      );
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <main className="flex flex-col min-h-screen bg-base">
      <Navbar />

      <form
        onSubmit={handleSubmit}
        className="w-full max-w-[1221px] mx-auto px-4 lg:px-0 py-8 lg:py-12 flex-1 flex flex-col"
      >
        <h1 className="text-[58px] font-semibold italic text-ink-deep mb-8 lg:mb-10 tracking-tight leading-[66px]">
          Create your Event
        </h1>

        <div className="flex flex-col lg:flex-row gap-8 lg:gap-[60px] items-start">
          <div className="w-full lg:w-[450px] shrink-0">
            <button
              type="button"
              className="relative w-full aspect-square rounded-[24px] overflow-hidden group border border-black/5 shadow-sm text-left block"
            >
              <div className="absolute inset-0 bg-linear-to-br from-[#0B7A75] via-[#314FB5] to-[#E35661]">
                <div className="absolute inset-4 border-[1.5px] border-dashed border-white/30 rounded-[16px] pointer-events-none" />

                <div className="absolute inset-0 flex flex-col items-center justify-center p-8 pointer-events-none">
                  <div className="-rotate-15 flex flex-col items-center gap-5">
                    <span className="border-2 border-white text-white rounded-full px-8 py-2 text-4xl lg:text-[40px] font-medium tracking-wide">
                      You&apos;re
                    </span>
                    <span className="border-2 border-white text-white rounded-full px-10 py-2 text-4xl lg:text-[40px] font-medium tracking-wide translate-x-4">
                      Invited
                    </span>
                  </div>
                </div>
              </div>

              <div className="absolute inset-0 flex items-center justify-center bg-black/0 group-hover:bg-black/10 transition-colors duration-300">
                <div className="w-[60px] h-[60px] bg-white/90 backdrop-blur-sm rounded-full flex items-center justify-center shadow-lg text-black group-hover:scale-110 transition-transform duration-300">
                  <Image
                    src="/icons/camera.svg"
                    alt="Upload Cover Photo"
                    width={26}
                    height={26}
                    className="opacity-80"
                  />
                </div>
              </div>

              <span className="sr-only">Upload cover photo for your event</span>
            </button>
          </div>

          <div className="flex-1 w-full flex flex-col gap-4">
            <div
              className={`bg-white/50 backdrop-blur-sm border-[1.5px] rounded-[16px] p-6 lg:p-7 flex flex-col justify-center relative shadow-sm min-h-[120px] transition-colors ${errors.title ? "border-red-500 bg-red-50/10" : "border-black/3"}`}
            >
              <label className="text-[15px] font-semibold text-ink-alt absolute top-4 left-6 leading-[66px]">
                Event Title
              </label>
              <input
                type="text"
                name="title"
                value={formData.title}
                onChange={handleChange}
                placeholder="Event Name"
                className="text-[38px] font-semibold placeholder:text-muted-text/30 text-muted-text outline-none w-full bg-transparent mt-12 mb-2"
              />
              {errors.title && (
                <span className="text-red-500 text-sm font-bold absolute bottom-2 right-6">
                  {errors.title}
                </span>
              )}
            </div>

            <div className="flex flex-col lg:flex-row gap-4 mt-2">
              <div
                className={`flex-2 bg-white/50 backdrop-blur-sm border-[1.5px] rounded-[16px] p-6 flex flex-col justify-between relative min-h-[130px] shadow-sm transition-colors ${errors.startDate || errors.startTime ? "border-red-500 bg-red-50/10" : "border-black/3"}`}
              >
                <div className="absolute left-[39px] top-[46px] bottom-[46px] w-px bg-black/50" />

                <div className="flex items-center">
                  <div className="w-[10px] h-[10px] rounded-[10px] bg-dark-deep shrink-0 relative z-10 ml-3" />
                  <span className="text-[15px] font-semibold w-12 text-black ml-3 leading-[24px]">
                    Start
                  </span>
                  <div className="flex-1 flex gap-[3px] ml-2">
                    <input
                      type="date"
                      name="startDate"
                      value={formData.startDate}
                      onChange={handleChange}
                      className="flex h-[34px] min-w-[118px] bg-dark-alt/4 rounded-[8px] px-2 items-center justify-center text-[14px] font-semibold text-black outline-none cursor-pointer w-full text-center"
                    />
                    <div className="w-px h-[34px] bg-black/10 shrink-0" />
                    <input
                      type="time"
                      name="startTime"
                      value={formData.startTime}
                      onChange={handleChange}
                      className="flex h-[34px] min-w-[118px] bg-dark-alt/4 rounded-[8px] px-2 items-center justify-center text-[14px] font-semibold text-black outline-none cursor-pointer w-full text-center"
                    />
                  </div>
                </div>

                {(errors.startDate || errors.startTime) && (
                  <span className="text-red-500 text-xs font-bold mt-1 ml-10">
                    {errors.startDate || errors.startTime}
                  </span>
                )}

                <div className="flex items-center mt-[18px]">
                  <div className="w-[10px] h-[10px] rounded-[10px] border border-dark-deep/50 bg-transparent shrink-0 relative z-10 ml-3" />
                  <span className="text-[15px] font-semibold w-12 text-black ml-3 leading-[24px]">
                    End
                  </span>
                  <div className="flex-1 flex gap-[3px] ml-2">
                    <input
                      type="date"
                      name="endDate"
                      value={formData.endDate}
                      onChange={handleChange}
                      className="flex h-[34px] min-w-[118px] bg-dark-alt/4 rounded-[8px] px-2 items-center justify-center text-[14px] font-semibold text-black outline-none cursor-pointer w-full text-center"
                    />
                    <div className="w-px h-[34px] bg-black/10 shrink-0" />
                    <input
                      type="time"
                      name="endTime"
                      value={formData.endTime}
                      onChange={handleChange}
                      className="flex h-[34px] min-w-[118px] bg-dark-alt/4 rounded-[8px] px-2 items-center justify-center text-[14px] font-semibold text-black outline-none cursor-pointer w-full text-center"
                    />
                  </div>
                </div>
              </div>

              <div className="flex-1 bg-white/50 backdrop-blur-sm border-[1.5px] border-black/3 rounded-[16px] p-4 px-5 flex items-center justify-between shadow-sm min-w-[200px]">
                <div className="flex flex-col gap-0 justify-center h-full">
                  {isMounted ? (
                    <>
                      <span className="text-[15px] font-medium text-black leading-[18px]">
                        {(() => {
                          const offsetMinutes = -new Date().getTimezoneOffset();
                          const sign = offsetMinutes >= 0 ? "+" : "-";
                          const absMinutes = Math.abs(offsetMinutes);
                          const hours = String(
                            Math.floor(absMinutes / 60),
                          ).padStart(2, "0");
                          const mins = String(absMinutes % 60).padStart(2, "0");
                          return `GMT${sign}${hours}:${mins}`;
                        })()}
                      </span>
                      <span className="text-[15px] text-black/50 leading-[20px] -mt-[2px]">
                        {Intl.DateTimeFormat().resolvedOptions().timeZone}
                      </span>
                    </>
                  ) : (
                    <>
                      <span className="text-[15px] font-medium text-black leading-[18px] invisible">
                        GMT+00:00
                      </span>
                      <span className="text-[15px] text-black/50 leading-[20px] -mt-[2px] invisible">
                        UTC
                      </span>
                    </>
                  )}
                </div>
                <div className="w-[49px] h-[49px] bg-base rounded-[120px] flex items-center justify-center shrink-0">
                  <Image
                    src="/icons/global.svg"
                    width={24}
                    height={24}
                    alt="timezone"
                  />
                </div>
              </div>
            </div>

            <div
              className={`bg-white/50 backdrop-blur-sm border-[1.5px] rounded-[16px] p-6 flex flex-col justify-center relative shadow-sm min-h-[120px] mt-2 transition-colors ${errors.location ? "border-red-500 bg-red-50/10" : "border-black/3"}`}
            >
              <label className="text-[15px] font-semibold text-ink-alt absolute top-3 left-6 leading-[66px]">
                Add Event Location
              </label>
              <div className="flex items-center justify-between w-full mt-[50px]">
                <input
                  type="text"
                  name="location"
                  value={formData.location}
                  onChange={handleChange}
                  placeholder="Offline location or virtual link"
                  className="text-[19px] font-semibold placeholder:text-muted-text/30 text-muted-text outline-none bg-transparent flex-1"
                />
                <div className="flex gap-[10px] shrink-0">
                  <button
                    type="button"
                    className="w-[49px] h-[49px] rounded-[120px] bg-subtle/50 flex items-center justify-center hover:bg-subtle/70 transition-colors"
                  >
                    <Image
                      src="/icons/video.svg"
                      width={24}
                      height={24}
                      alt="Virtual"
                    />
                  </button>
                  <button
                    type="button"
                    className="w-[49px] h-[49px] rounded-[120px] bg-subtle/50 flex items-center justify-center hover:bg-subtle/70 transition-colors"
                  >
                    <Image
                      src="/icons/map-pin.svg"
                      width={24}
                      height={24}
                      alt="Offline"
                    />
                  </button>
                </div>
              </div>
              {errors.location && (
                <span className="text-red-500 text-sm font-bold absolute bottom-2 left-6">
                  {errors.location}
                </span>
              )}
            </div>

            <div className="bg-white/50 backdrop-blur-sm border-[1.5px] border-black/3 rounded-[16px] p-6 flex flex-col justify-center relative shadow-sm min-h-[120px] mt-2">
              <label className="text-[15px] font-semibold text-ink-alt absolute top-3 left-6 leading-[66px]">
                Add Description
              </label>
              <div className="flex items-end justify-between w-full mt-[50px] gap-4">
                <textarea
                  name="description"
                  value={formData.description}
                  onChange={handleChange}
                  placeholder="Add Description about this Event..."
                  className="text-[19px] font-semibold placeholder:text-muted-text/30 text-muted-text outline-none flex-1 bg-transparent pb-1 resize-none"
                  rows={1}
                />
                <button
                  type="button"
                  className="w-[49px] h-[49px] rounded-[120px] bg-base flex items-center justify-center hover:bg-accent-muted transition-colors shrink-0"
                >
                  <Image
                    src="/icons/edit.svg"
                    width={24}
                    height={24}
                    alt="Edit"
                  />
                </button>
              </div>
            </div>

            <div className="bg-white/50 backdrop-blur-sm border-[1.5px] border-black/3 rounded-[16px] p-6 flex flex-col justify-center relative shadow-sm min-h-[120px] mt-2">
              <label className="text-[15px] font-semibold text-ink-alt absolute top-3 left-6 leading-[66px]">
                Cover Image URL
              </label>
              <div className="flex items-center justify-between w-full mt-[50px] gap-4">
                <input
                  type="url"
                  name="imageUrl"
                  value={formData.imageUrl}
                  onChange={(e) => {
                    handleChange(e);
                    setImageError(false);
                  }}
                  placeholder="https://example.com/image.jpg"
                  className="text-[19px] font-semibold placeholder:text-muted-text/30 text-muted-text outline-none flex-1 bg-transparent"
                />
                <div className="w-[49px] h-[49px] rounded-[120px] bg-base flex items-center justify-center shrink-0">
                  <Image
                    src="/icons/camera.svg"
                    width={24}
                    height={24}
                    alt="Image URL"
                  />
                </div>
              </div>
            </div>

            {formData.imageUrl && isValidUrl(formData.imageUrl) && (
              <div className="rounded-[16px] overflow-hidden border-[1.5px] border-black/3 shadow-sm mt-2 aspect-video relative bg-subtle/30">
                {imageError ? (
                  <div className="absolute inset-0 flex flex-col items-center justify-center gap-2 text-muted-text">
                    <Image
                      src="/icons/camera.svg"
                      width={32}
                      height={32}
                      alt=""
                      className="opacity-30"
                    />
                    <span className="text-[15px] font-semibold opacity-50">
                      Failed to load image
                    </span>
                  </div>
                ) : (
                  // eslint-disable-next-line @next/next/no-img-element
                  <img
                    src={formData.imageUrl}
                    alt="Event cover preview"
                    className="w-full h-full object-cover"
                    onError={() => setImageError(true)}
                  />
                )}
              </div>
            )}

            <h2 className="text-[19px] font-bold mt-4 text-ink-alt leading-[66px] h-[30px] flex items-center">
              Event Options
            </h2>

            <div className="flex flex-col lg:flex-row gap-4 mt-2">
              <div className="flex-3 bg-subtle/50 border-[1.5px] border-black/3 backdrop-blur-sm rounded-[16px] p-4 flex gap-[10px] shadow-sm relative pt-[50px]">
                <label className="text-[15px] font-semibold text-ink-alt absolute top-2 left-4 leading-[66px]">
                  Event Visibility
                </label>

                <button
                  type="button"
                  onClick={() =>
                    setFormData((prev) => ({ ...prev, visibility: "Public" }))
                  }
                  className={`flex-1 border-[1.5px] backdrop-blur-sm rounded-[16px] h-[80px] px-4 flex items-center justify-center gap-4 transition-colors ${formData.visibility === "Public" ? "bg-white border-black shadow-sm" : "bg-subtle/50 border-black/3 hover:bg-subtle/70"}`}
                >
                  <span className="font-semibold text-black text-[19px] leading-[18px]">
                    Public
                  </span>
                  <div className="w-[49px] h-[49px] rounded-[30px] bg-white flex items-center justify-center shrink-0 shadow-sm border border-black/5">
                    <Image
                      src="/icons/megaphone.svg"
                      width={24}
                      height={24}
                      alt="Public"
                    />
                  </div>
                </button>

                <button
                  type="button"
                  onClick={() =>
                    setFormData((prev) => ({ ...prev, visibility: "Private" }))
                  }
                  className={`flex-1 border-[1.5px] backdrop-blur-sm rounded-[16px] h-[80px] px-4 flex items-center justify-center gap-4 transition-colors ${formData.visibility === "Private" ? "bg-white border-black shadow-sm" : "bg-subtle/50 border-black/3 hover:bg-subtle/70"}`}
                >
                  <span className="font-semibold text-black text-[19px] leading-[18px]">
                    Private
                  </span>
                  <div className="w-[49px] h-[49px] rounded-[30px] bg-white flex items-center justify-center shrink-0 shadow-sm border border-black/5">
                    <Image
                      src="/icons/lock.svg"
                      width={24}
                      height={24}
                      alt="Private"
                    />
                  </div>
                </button>
              </div>

              <div className="flex-2 bg-white/50 backdrop-blur-sm border-[1.5px] border-black/3 rounded-[16px] p-6 flex flex-col justify-center relative shadow-sm min-h-[150px]">
                <label className="text-[15px] font-semibold text-ink-alt absolute top-3 left-4 leading-[66px]">
                  Set Capacity
                </label>
                <div className="flex items-center justify-between w-full mt-[50px] gap-4">
                  <input
                    type="number"
                    name="capacity"
                    value={formData.capacity}
                    onChange={handleChange}
                    placeholder="Unlimited"
                    className="text-[19px] font-semibold placeholder:text-muted-text/30 text-muted-text outline-none flex-1 bg-transparent"
                  />
                  <button
                    type="button"
                    className="w-[49px] h-[49px] rounded-[120px] bg-base flex items-center justify-center hover:bg-accent-muted transition-colors shrink-0"
                  >
                    <Image
                      src="/icons/edit.svg"
                      width={24}
                      height={24}
                      alt="Edit"
                    />
                  </button>
                </div>
              </div>
            </div>

            <div
              className={`bg-white/50 backdrop-blur-sm border-[1.5px] rounded-[16px] p-6 flex flex-col justify-center relative shadow-sm min-h-[120px] mt-2 transition-colors ${errors.price ? "border-red-500 bg-red-50/10" : "border-black/3"}`}
            >
              <label className="text-[15px] font-semibold text-ink-alt absolute top-3 left-6 leading-[66px]">
                Ticket Price
              </label>
              <div className="flex items-center justify-between w-full mt-[50px] gap-4">
                <input
                  type="number"
                  name="price"
                  value={formData.price}
                  onChange={handleChange}
                  placeholder="Free"
                  className="text-[19px] font-semibold placeholder:text-muted-text/30 text-muted-text outline-none flex-1 bg-transparent"
                />
                <div className="w-[49px] h-[49px] rounded-[120px] bg-base flex items-center justify-center shrink-0">
                  <Image
                    src="/icons/ticket.svg"
                    width={24}
                    height={24}
                    alt="Price"
                  />
                </div>
              </div>
              {errors.price && (
                <span className="text-red-500 text-sm font-bold absolute bottom-2 left-6">
                  {errors.price}
                </span>
              )}
            </div>

            <div className="flex justify-end gap-4 mt-6 mr-4">
              <Button
                type="button"
                variant="secondary"
                className="w-[212px] h-[50px] rounded-[32px]"
                onClick={() => {
                  setFormData({
                    title: "",
                    startDate: "",
                    startTime: "",
                    endDate: "",
                    endTime: "",
                    location: "",
                    description: "",
                    capacity: "",
                    price: "",
                    imageUrl: "",
                    visibility: "Public",
                  });
                  setErrors({});
                  setImageError(false);
                }}
              >
                Clear Event
              </Button>
              <Button
                type="submit"
                variant="primary"
                disabled={isSubmitting}
                className="w-[212px] h-[50px] rounded-[32px] disabled:opacity-70 disabled:cursor-not-allowed"
              >
                {isSubmitting ? "Creating..." : "Create Event"}
                {!isSubmitting && (
                  <Image
                    src="/icons/arrow-up-right-01.svg"
                    width={24}
                    height={24}
                    alt="Create"
                  />
                )}
              </Button>
            </div>
          </div>
        </div>
      </form>
      <Footer />
    </main>
  );
}
