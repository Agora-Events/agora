"use client";

import React, { useState } from "react";
import Image from "next/image";
import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";

export default function CreateEventPage() {
  const [visibility, setVisibility] = useState<"Public" | "Private">("Public");

  return (
    <main className="flex flex-col min-h-screen bg-[#FFFBE9]">
      <Navbar />

      <div className="w-full max-w-[1221px] mx-auto px-4 lg:px-0 py-8 lg:py-12 flex-1 flex flex-col">
        <h1 className="text-[58px] font-semibold italic text-[#131517] mb-8 lg:mb-10 tracking-tight leading-[66px]">
          Create your Event
        </h1>

        <div className="flex flex-col lg:flex-row gap-8 lg:gap-[60px] items-start">
          <div className="w-full lg:w-[450px] shrink-0">
            <button className="relative w-full aspect-square rounded-[24px] overflow-hidden group border border-black/5 shadow-sm text-left block">
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
            <div className="bg-white/50 backdrop-blur-sm border-[1.5px] border-black/3 rounded-[16px] p-6 lg:p-7 flex flex-col justify-center relative shadow-sm min-h-[120px]">
              <label className="text-[15px] font-semibold text-[#1C1C1C] absolute top-4 left-6 leading-[66px]">
                Event Title
              </label>
              <input
                type="text"
                placeholder="Event Name"
                className="text-[38px] font-semibold placeholder:text-[#747475]/30 text-[#747475] outline-none w-full bg-transparent mt-12 mb-2"
              />
            </div>

            <div className="flex flex-col lg:flex-row gap-4 mt-2">
              <div className="flex-2 bg-white/50 backdrop-blur-sm border-[1.5px] border-black/3 rounded-[16px] p-6 flex flex-col justify-between relative min-h-[130px] shadow-sm">
                <div className="absolute left-[39px] top-[46px] bottom-[46px] w-px bg-black/50" />

                <div className="flex items-center">
                  <div className="w-[10px] h-[10px] rounded-[10px] bg-[#171402] shrink-0 relative z-10 ml-3" />
                  <span className="text-[15px] font-semibold w-12 text-[#000000] ml-3 leading-[24px]">
                    Start
                  </span>
                  <div className="flex-1 flex gap-[3px] ml-2">
                    <input
                      type="date"
                      className="flex h-[34px] min-w-[118px] bg-[#2F2E24]/4 rounded-[8px] px-2 items-center justify-center text-[14px] font-semibold text-black outline-none cursor-pointer w-full text-center"
                    />
                    <div className="w-px h-[34px] bg-black/10 shrink-0" />
                    <input
                      type="time"
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
                      className="flex h-[34px] min-w-[118px] bg-[#2F2E24]/4 rounded-[8px] px-2 items-center justify-center text-[14px] font-semibold text-black outline-none cursor-pointer w-full text-center"
                    />
                    <div className="w-px h-[34px] bg-black/10 shrink-0" />
                    <input
                      type="time"
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

            <div className="bg-white/50 backdrop-blur-sm border-[1.5px] border-black/3 rounded-[16px] p-6 flex flex-col justify-center relative shadow-sm min-h-[120px] mt-2">
              <label className="text-[15px] font-semibold text-[#1C1C1C] absolute top-3 left-6 leading-[66px]">
                Add Event Location
              </label>
              <div className="flex items-center justify-between w-full mt-[50px]">
                <input
                  type="text"
                  placeholder="Offline location or virtual link"
                  className="text-[19px] font-semibold placeholder:text-[#747475]/30 text-[#747475] outline-none bg-transparent flex-1"
                />
                <div className="flex gap-[10px] shrink-0">
                  <button className="w-[49px] h-[49px] rounded-[120px] bg-[#D5D5D6]/50 flex items-center justify-center hover:bg-[#D5D5D6]/70 transition-colors">
                    <Image
                      src="/icons/video.svg"
                      width={24}
                      height={24}
                      alt="Virtual"
                    />
                  </button>
                  <button className="w-[49px] h-[49px] rounded-[120px] bg-[#D5D5D6]/50 flex items-center justify-center hover:bg-[#D5D5D6]/70 transition-colors">
                    <Image
                      src="/icons/map-pin.svg"
                      width={24}
                      height={24}
                      alt="Offline"
                    />
                  </button>
                </div>
              </div>
            </div>

            <div className="bg-white/50 backdrop-blur-sm border-[1.5px] border-black/3 rounded-[16px] p-6 flex flex-col justify-center relative shadow-sm min-h-[120px] mt-2">
              <label className="text-[15px] font-semibold text-[#1C1C1C] absolute top-3 left-6 leading-[66px]">
                Add Description
              </label>
              <div className="flex items-end justify-between w-full mt-[50px] gap-4">
                <input
                  type="text"
                  placeholder="Add Description about this Event..."
                  className="text-[19px] font-semibold placeholder:text-[#747475]/30 text-[#747475] outline-none flex-1 bg-transparent pb-1"
                />
                <button className="w-[49px] h-[49px] rounded-[120px] bg-[#FFFBE9] flex items-center justify-center hover:bg-[#F2ECCD] transition-colors shrink-0">
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
                  onClick={() => setVisibility("Public")}
                  className={`flex-1 border-[1.5px] backdrop-blur-sm rounded-[16px] h-[80px] px-4 flex items-center justify-center gap-4 transition-colors ${visibility === "Public" ? "bg-white border-black shadow-sm" : "bg-[#D5D5D6]/50 border-black/3 hover:bg-[#D5D5D6]/70"}`}
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
                  onClick={() => setVisibility("Private")}
                  className={`flex-1 border-[1.5px] backdrop-blur-sm rounded-[16px] h-[80px] px-4 flex items-center justify-center gap-4 transition-colors ${visibility === "Private" ? "bg-white border-black shadow-sm" : "bg-[#D5D5D6]/50 border-black/3 hover:bg-[#D5D5D6]/70"}`}
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
                    placeholder="Unlimited"
                    className="text-[19px] font-semibold placeholder:text-[#747475]/30 text-[#747475] outline-none flex-1 bg-transparent"
                  />
                  <button className="w-[49px] h-[49px] rounded-[120px] bg-[#FFFBE9] flex items-center justify-center hover:bg-[#F2ECCD] transition-colors shrink-0">
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

            <div className="bg-white/50 backdrop-blur-sm border-[1.5px] border-black/3 rounded-[16px] p-6 flex flex-col justify-center relative shadow-sm min-h-[120px] mt-2">
              <label className="text-[15px] font-semibold text-[#1C1C1C] absolute top-3 left-6 leading-[66px]">
                Ticket Price
              </label>
              <div className="flex items-center justify-between w-full mt-[50px] gap-4">
                <input
                  type="number"
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
            </div>

            <div className="flex justify-end gap-14 mt-6 mr-4">
              <div className="relative w-[212px] h-[58px]">
                <div className="absolute w-[212px] h-[50px] left-0 top-[6px] bg-[#000000] rounded-[32px]" />
                <button
                  type="button"
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
                  className="absolute w-[212px] h-[50px] left-[3px] top-0 bg-[#FDDA23] border-2 border-[#000000] rounded-[32px] flex items-center justify-center gap-[10px] hover:bg-[#f0ce1e] transition-transform active:translate-y-1 active:translate-x-px"
                >
                  <span className="font-semibold text-[15px] text-[#000000] text-center leading-[30px]">
                    Create Event
                  </span>
                  <div className="w-[24px] h-[24px] flex items-center justify-center">
                    <Image
                      src="/icons/arrow-up-right-01.svg"
                      width={24}
                      height={24}
                      alt="Create"
                    />
                  </div>
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <Footer />
    </main>
  );
}
