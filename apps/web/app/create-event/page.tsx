"use client";

import React, { useState } from "react";
import Image from "next/image";
import Link from "next/link";
import { motion, AnimatePresence } from "framer-motion";
import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";
import { CheckCircle2, ArrowRight, Home, ExternalLink } from "lucide-react";
import { toast } from "sonner";

export default function CreateEventPage() {
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isSuccess, setIsSuccess] = useState(false);
  const [createdEventId, setCreatedEventId] = useState<string | null>(null);

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
    visibility: "Public" as "Public" | "Private",
  });

  // Error State
  const [errors, setErrors] = useState<Record<string, string>>({});

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => {
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

  const validate = () => {
    const newErrors: Record<string, string> = {};
    if (!formData.title.trim()) newErrors.title = "Event title is required";
    if (!formData.startDate) newErrors.startDate = "Start date is required";
    if (!formData.startTime) newErrors.startTime = "Start time is required";
    if (!formData.location.trim()) newErrors.location = "Location is required";
    if (!formData.price.trim()) newErrors.price = "Price is required (put 0 for free)";

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!validate()) {
      toast.error("Please fill in all required fields");
      return;
    }

    setIsSubmitting(true);
    try {
      const response = await fetch("/api/events", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          title: formData.title,
          startsAt: `${formData.startDate}T${formData.startTime}:00.000Z`,
          location: formData.location,
          category: "Tech", // Default category
          organizerName: "Stellar Community", // Mocked
          organizerWallet: "GD...MOCK_WALLET", // Mocked
          description: formData.description,
          ticketPrice: parseFloat(formData.price) || 0,
          totalTickets: parseInt(formData.capacity) || 100,
          followersOnly: formData.visibility === "Private",
        }),
      });

      const data = await response.json();

      if (!response.ok) {
        throw new Error(data.error || "Failed to create event");
      }

      setCreatedEventId(data.event.id);
      setIsSuccess(true);
      toast.success("Event created successfully!");
    } catch (error: any) {
      toast.error(error.message || "Something went wrong");
    } finally {
      setIsSubmitting(false);
    }
  };

  if (isSuccess) {
    return (
      <main className="flex flex-col min-h-screen bg-[#FFFBE9]">
        <Navbar />
        <div className="flex-1 flex items-center justify-center p-6">
          <motion.div
            initial={{ scale: 0.9, opacity: 0 }}
            animate={{ scale: 1, opacity: 1 }}
            className="w-full max-w-[600px] bg-white rounded-[40px] p-10 lg:p-16 border border-black/5 shadow-2xl flex flex-col items-center text-center gap-8"
          >
            <div className="w-24 h-24 rounded-full bg-green-100 flex items-center justify-center text-green-600">
              <CheckCircle2 size={56} />
            </div>

            <div className="flex flex-col gap-3">
              <h1 className="text-[40px] lg:text-[48px] font-bold text-black font-heading leading-tight italic">
                Your event has been created!
              </h1>
              <p className="text-xl text-black/60 font-medium">
                Awesome! Your event is now live and ready for registrations.
              </p>
            </div>

            <div className="flex flex-col w-full gap-4 mt-4">
              <Link href={`/events/${createdEventId?.replace("evt_", "")}`} className="w-full">
                <button className="w-full bg-[#FDDA23] text-black font-bold text-xl h-16 rounded-full border-2 border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] active:translate-x-[2px] active:translate-y-[2px] active:shadow-none transition-all flex items-center justify-center gap-3">
                  <span>View Event</span>
                  <ExternalLink size={24} />
                </button>
              </Link>
              <Link href="/home" className="w-full text-black/60 font-bold text-lg flex items-center justify-center gap-2 hover:text-black transition-colors">
                <Home size={20} />
                <span>Back to Dashboard</span>
              </Link>
            </div>
          </motion.div>
        </div>
        <Footer />
      </main>
    );
  }

  return (
    <main className="flex flex-col min-h-screen bg-[#FFFBE9]">
      <Navbar />

      <form onSubmit={handleSubmit} className="w-full max-w-[1221px] mx-auto px-4 lg:px-0 py-8 lg:py-12 flex-1 flex flex-col">
        <h1 className="text-[58px] font-semibold italic text-[#131517] mb-8 lg:mb-10 tracking-tight leading-[66px]">
          Create your Event
        </h1>

        <div className="flex flex-col lg:flex-row gap-8 lg:gap-[60px] items-start">
          <div className="w-full lg:w-[450px] shrink-0">
            <button type="button" className="relative w-full aspect-square rounded-[24px] overflow-hidden group border border-black/5 shadow-sm text-left block">
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
            <div className={`bg-white/50 backdrop-blur-sm border-[1.5px] rounded-[16px] p-6 lg:p-7 flex flex-col justify-center relative shadow-sm min-h-[120px] transition-colors ${errors.title ? "border-red-500 bg-red-50/10" : "border-black/3"}`}>
              <label className="text-[15px] font-semibold text-[#1C1C1C] absolute top-4 left-6 leading-[66px]">
                Event Title
              </label>
              <input
                type="text"
                name="title"
                value={formData.title}
                onChange={handleChange}
                placeholder="Event Name"
                className="text-[38px] font-semibold placeholder:text-[#747475]/30 text-[#747475] outline-none w-full bg-transparent mt-12 mb-2"
              />
              {errors.title && <span className="text-red-500 text-sm font-bold absolute bottom-2 right-6">{errors.title}</span>}
            </div>

            <div className="flex flex-col lg:flex-row gap-4 mt-2">
              <div className={`flex-2 bg-white/50 backdrop-blur-sm border-[1.5px] rounded-[16px] p-6 flex flex-col justify-between relative min-h-[130px] shadow-sm transition-colors ${errors.startDate || errors.startTime ? "border-red-500 bg-red-50/10" : "border-black/3"}`}>
                <div className="absolute left-[39px] top-[46px] bottom-[46px] w-px bg-black/50" />

                <div className="flex items-center">
                  <div className="w-[10px] h-[10px] rounded-[10px] bg-[#171402] shrink-0 relative z-10 ml-3" />
                  <span className="text-[15px] font-semibold w-12 text-[#000000] ml-3 leading-[24px]">
                    Start
                  </span>
                  <div className="flex-1 flex gap-[3px] ml-2">
                    <input
                      type="date"
                      name="startDate"
                      value={formData.startDate}
                      onChange={handleChange}
                      className="flex h-[34px] min-w-[118px] bg-[#2F2E24]/4 rounded-[8px] px-2 items-center justify-center text-[14px] font-semibold text-black outline-none cursor-pointer w-full text-center"
                    />
                    <div className="w-px h-[34px] bg-black/10 shrink-0" />
                    <input
                      type="time"
                      name="startTime"
                      value={formData.startTime}
                      onChange={handleChange}
                      className="flex h-[34px] min-w-[118px] bg-[#2F2E24]/4 rounded-[8px] px-2 items-center justify-center text-[14px] font-semibold text-black outline-none cursor-pointer w-full text-center"
                    />
                  </div>
                </div>

                <div className="flex items-center mt-[18px]">
                  <div className="w-[10px] h-[10px] rounded-[10px] border border-[#171402]/50 bg-transparent shrink-0 relative z-10 ml-3" />
                  <span className="text-[15px] font-semibold w-12 text-[#000000] ml-3 leading-[24px]">
                    End
                  </span>
                  <div className="flex-1 flex gap-[3px] ml-2">
                    <input
                      type="date"
                      name="endDate"
                      value={formData.endDate}
                      onChange={handleChange}
                      className="flex h-[34px] min-w-[118px] bg-[#2F2E24]/4 rounded-[8px] px-2 items-center justify-center text-[14px] font-semibold text-black outline-none cursor-pointer w-full text-center"
                    />
                    <div className="w-px h-[34px] bg-black/10 shrink-0" />
                    <input
                      type="time"
                      name="endTime"
                      value={formData.endTime}
                      onChange={handleChange}
                      className="flex h-[34px] min-w-[118px] bg-[#2F2E24]/4 rounded-[8px] px-2 items-center justify-center text-[14px] font-semibold text-black outline-none cursor-pointer w-full text-center"
                    />
                  </div>
                </div>
              </div>

              <div className="flex-1 bg-white/50 backdrop-blur-sm border-[1.5px] border-black/3 rounded-[16px] p-4 px-5 flex items-center justify-between shadow-sm min-w-[200px]">
                <div className="flex flex-col gap-0 justify-center h-full">
                  <span className="text-[15px] font-medium text-[#000000] leading-[18px]">
                    GMT+00:00
                  </span>
                  <span className="text-[15px] text-black/50 leading-[20px] -mt-[2px]">
                    UTC
                  </span>
                </div>
                <div className="w-[49px] h-[49px] bg-[#FFFBE9] rounded-[120px] flex items-center justify-center shrink-0">
                  <Image
                    src="/icons/global.svg"
                    width={24}
                    height={24}
                    alt="timezone"
                  />
                </div>
              </div>
            </div>

            <div className={`bg-white/50 backdrop-blur-sm border-[1.5px] rounded-[16px] p-6 flex flex-col justify-center relative shadow-sm min-h-[120px] mt-2 transition-colors ${errors.location ? "border-red-500 bg-red-50/10" : "border-black/3"}`}>
              <label className="text-[15px] font-semibold text-[#1C1C1C] absolute top-3 left-6 leading-[66px]">
                Add Event Location
              </label>
              <div className="flex items-center justify-between w-full mt-[50px]">
                <input
                  type="text"
                  name="location"
                  value={formData.location}
                  onChange={handleChange}
                  placeholder="Offline location or virtual link"
                  className="text-[19px] font-semibold placeholder:text-[#747475]/30 text-[#747475] outline-none bg-transparent flex-1"
                />
                <div className="flex gap-[10px] shrink-0">
                  <button type="button" className="w-[49px] h-[49px] rounded-[120px] bg-[#D5D5D6]/50 flex items-center justify-center hover:bg-[#D5D5D6]/70 transition-colors">
                    <Image
                      src="/icons/video.svg"
                      width={24}
                      height={24}
                      alt="Virtual"
                    />
                  </button>
                  <button type="button" className="w-[49px] h-[49px] rounded-[120px] bg-[#D5D5D6]/50 flex items-center justify-center hover:bg-[#D5D5D6]/70 transition-colors">
                    <Image
                      src="/icons/map-pin.svg"
                      width={24}
                      height={24}
                      alt="Offline"
                    />
                  </button>
                </div>
              </div>
              {errors.location && <span className="text-red-500 text-sm font-bold absolute bottom-2 left-6">{errors.location}</span>}
            </div>

            <div className="bg-white/50 backdrop-blur-sm border-[1.5px] border-black/3 rounded-[16px] p-6 flex flex-col justify-center relative shadow-sm min-h-[120px] mt-2">
              <label className="text-[15px] font-semibold text-[#1C1C1C] absolute top-3 left-6 leading-[66px]">
                Add Description
              </label>
              <div className="flex items-end justify-between w-full mt-[50px] gap-4">
                <textarea
                  name="description"
                  value={formData.description}
                  onChange={handleChange}
                  placeholder="Add Description about this Event..."
                  className="text-[19px] font-semibold placeholder:text-[#747475]/30 text-[#747475] outline-none flex-1 bg-transparent pb-1 resize-none"
                  rows={1}
                />
                <button type="button" className="w-[49px] h-[49px] rounded-[120px] bg-[#FFFBE9] flex items-center justify-center hover:bg-[#F2ECCD] transition-colors shrink-0">
                  <Image
                    src="/icons/edit.svg"
                    width={24}
                    height={24}
                    alt="Edit"
                  />
                </button>
              </div>
            </div>

            <h2 className="text-[19px] font-bold mt-4 text-[#1C1C1C] leading-[66px] h-[30px] flex items-center">
              Event Options
            </h2>

            <div className="flex flex-col lg:flex-row gap-4 mt-2">
              <div className="flex-3 bg-[#D5D5D6]/50 border-[1.5px] border-black/3 backdrop-blur-sm rounded-[16px] p-4 flex gap-[10px] shadow-sm relative pt-[50px]">
                <label className="text-[15px] font-semibold text-[#1C1C1C] absolute top-2 left-4 leading-[66px]">
                  Event Visibility
                </label>

                <button
                  type="button"
                  onClick={() => setFormData((prev) => ({ ...prev, visibility: "Public" }))}
                  className={`flex-1 border-[1.5px] backdrop-blur-sm rounded-[16px] h-[80px] px-4 flex items-center justify-center gap-4 transition-colors ${formData.visibility === "Public" ? "bg-white border-black shadow-sm" : "bg-[#D5D5D6]/50 border-black/3 hover:bg-[#D5D5D6]/70"}`}
                >
                  <span className="font-semibold text-[#000000] text-[19px] leading-[18px]">
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
                  onClick={() => setFormData((prev) => ({ ...prev, visibility: "Private" }))}
                  className={`flex-1 border-[1.5px] backdrop-blur-sm rounded-[16px] h-[80px] px-4 flex items-center justify-center gap-4 transition-colors ${formData.visibility === "Private" ? "bg-white border-black shadow-sm" : "bg-[#D5D5D6]/50 border-black/3 hover:bg-[#D5D5D6]/70"}`}
                >
                  <span className="font-semibold text-[#000000] text-[19px] leading-[18px]">
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
                <label className="text-[15px] font-semibold text-[#1C1C1C] absolute top-3 left-4 leading-[66px]">
                  Set Capacity
                </label>
                <div className="flex items-center justify-between w-full mt-[50px] gap-4">
                  <input
                    type="number"
                    name="capacity"
                    value={formData.capacity}
                    onChange={handleChange}
                    placeholder="Unlimited"
                    className="text-[19px] font-semibold placeholder:text-[#747475]/30 text-[#747475] outline-none flex-1 bg-transparent"
                  />
                  <button type="button" className="w-[49px] h-[49px] rounded-[120px] bg-[#FFFBE9] flex items-center justify-center hover:bg-[#F2ECCD] transition-colors shrink-0">
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

            <div className={`bg-white/50 backdrop-blur-sm border-[1.5px] rounded-[16px] p-6 flex flex-col justify-center relative shadow-sm min-h-[120px] mt-2 transition-colors ${errors.price ? "border-red-500 bg-red-50/10" : "border-black/3"}`}>
              <label className="text-[15px] font-semibold text-[#1C1C1C] absolute top-3 left-6 leading-[66px]">
                Ticket Price
              </label>
              <div className="flex items-center justify-between w-full mt-[50px] gap-4">
                <input
                  type="number"
                  name="price"
                  value={formData.price}
                  onChange={handleChange}
                  placeholder="Free"
                  className="text-[19px] font-semibold placeholder:text-[#747475]/30 text-[#747475] outline-none flex-1 bg-transparent"
                />
                <div className="w-[49px] h-[49px] rounded-[120px] bg-[#FFFBE9] flex items-center justify-center shrink-0">
                  <Image
                    src="/icons/ticket.svg"
                    width={24}
                    height={24}
                    alt="Price"
                  />
                </div>
              </div>
              {errors.price && <span className="text-red-500 text-sm font-bold absolute bottom-2 left-6">{errors.price}</span>}
            </div>

            <div className="flex justify-end gap-14 mt-6 mr-4">
              <div className="relative w-[212px] h-[58px]">
                <div className="absolute w-[212px] h-[50px] left-0 top-[6px] bg-[#000000] rounded-[32px]" />
                <button
                  type="button"
                  onClick={() => setFormData({
                    title: "",
                    startDate: "",
                    startTime: "",
                    endDate: "",
                    endTime: "",
                    location: "",
                    description: "",
                    capacity: "",
                    price: "",
                    visibility: "Public",
                  })}
                  className="absolute w-[212px] h-[50px] left-[3px] top-0 bg-[#FFFFFF] border-2 border-[#000000] rounded-[32px] flex items-center justify-center hover:bg-gray-50 transition-transform active:translate-y-1 active:translate-x-px"
                >
                  <span className="font-semibold text-[15px] text-[#000000] text-center w-[131px] leading-[30px]">
                    Clear Event
                  </span>
                </button>
              </div>

              <div className="relative w-[215px] h-[58px]">
                <div className="absolute w-[212px] h-[50px] left-0 top-[6px] bg-[#000000] rounded-[32px]" />
                <button
                  type="submit"
                  disabled={isSubmitting}
                  className="absolute w-[212px] h-[50px] left-[3px] top-0 bg-[#FDDA23] border-2 border-[#000000] rounded-[32px] flex items-center justify-center gap-[10px] hover:bg-[#f0ce1e] transition-transform active:translate-y-1 active:translate-x-px disabled:opacity-70 disabled:cursor-not-allowed"
                >
                  <span className="font-semibold text-[15px] text-[#000000] text-center leading-[30px]">
                    {isSubmitting ? "Creating..." : "Create Event"}
                  </span>
                  {!isSubmitting && (
                    <div className="w-[24px] h-[24px] flex items-center justify-center">
                      <Image
                        src="/icons/arrow-up-right-01.svg"
                        width={24}
                        height={24}
                        alt="Create"
                      />
                    </div>
                  )}
                </button>
              </div>
            </div>
          </div>
        </div>
      </form>

      <Footer />
    </main>
  );
}
