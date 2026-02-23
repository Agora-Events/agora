"use client";

import Image from "next/image";
import React, { useState } from "react";

interface RegistrationBoxProps {
    isFree: boolean;
    price: string;
    host: {
        name: string;
        avatar: string;
        handle: string;
        hostPfp: string;
    };
}

export function RegistrationBox({ isFree, price, host }: RegistrationBoxProps) {
    const [quantity, setQuantity] = useState(1);

    return (
        <div className="bg-[#FFEFD3] rounded-[24px] p-6 sm:p-8 flex flex-col gap-8 relative overflow-hidden border border-black/5 shadow-sm">
            <div className="flex justify-between items-center z-10 flex-wrap gap-4">
                <div className="bg-white rounded-full px-6 py-2.5 italic text-gray-400 font-medium text-[17px] sm:text-[20px] shadow-sm flex-1 min-w-[150px]">
                    Registration
                </div>
                <div className="flex items-center gap-3">
                    <button
                        onClick={() => setQuantity(Math.max(1, quantity - 1))}
                        className="w-10 h-10 sm:w-12 sm:h-12 rounded-full bg-white border border-black/5 shadow-sm flex items-center justify-center hover:bg-gray-50 transition-colors text-2xl font-light text-black"
                    >
                        −
                    </button>
                    <div className="w-10 h-10 sm:w-12 sm:h-12 rounded-full bg-white border border-black/5 shadow-sm flex items-center justify-center">
                        <span className="text-lg sm:text-xl font-bold text-black">
                            {quantity}
                        </span>
                    </div>
                    <button
                        onClick={() => setQuantity(quantity + 1)}
                        className="w-10 h-10 sm:w-12 sm:h-12 rounded-full bg-white border border-black/5 shadow-sm flex items-center justify-center hover:bg-gray-50 transition-colors text-2xl font-light text-black"
                    >
                        +
                    </button>
                </div>
            </div>

            <p className="text-[16px] sm:text-[19px] text-black font-medium z-10">
                Welcome! To join the event, please register below.
            </p>

            <div className="flex items-center justify-between z-10 gap-4 flex-wrap">
                <button className="bg-[#FDDA23] text-black font-bold text-[18px] sm:text-[22px] h-14 sm:h-16 px-8 sm:px-10 rounded-full border-2 border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] active:translate-x-[2px] active:translate-y-[2px] active:shadow-none flex items-center justify-center gap-4">
                    {!isFree && (
                        <Image
                            src="/icons/dollar-circle.svg"
                            width={28}
                            height={28}
                            alt="dollar"
                        />
                    )}

                    {isFree
                        ? "Register"
                        : `$${(parseFloat(price) * quantity).toFixed(2)}`}

                    {isFree && (
                        <Image
                            src="/icons/arrow-up-right-01.svg"
                            width={24}
                            height={24}
                            alt="arrow-up-right"
                        />
                    )}
                </button>
                <div className="flex items-center gap-3">
                    <div className="relative w-11 h-11 sm:w-14 sm:h-14 rounded-full border-2 border-black overflow-hidden bg-white shadow-[3px_3px_0px_0px_rgba(0,0,0,1)]">
                        <Image
                            src={host.hostPfp}
                            fill
                            alt={host.handle}
                            className="object-cover"
                        />
                    </div>
                    <span className="text-[16px] sm:text-[18px] italic font-medium text-black whitespace-nowrap">
                        {host.handle}
                    </span>
                </div>
            </div>

            <div className="absolute -right-8 -bottom-8 opacity-[0.06] scale-150 pointer-events-none rotate-12 z-0">
                <Image
                    src="/icons/stellar-logo.svg"
                    width={240}
                    height={240}
                    alt="bg-logo"
                />
            </div>
        </div>
    );
}
