"use client";

import Image from "next/image";
import Link from "next/link";

export function Footer() {
  return (
    <footer className="w-full bg-[#060606] pt-20 pb-12 relative overflow-hidden text-white select-none">
      <div className="absolute -bottom-12 left-1/2 -translate-x-1/2 w-full max-w-[700px] h-[500px] pointer-events-none opacity-30 mix-blend-screen">
        <Image
          src="/images/World1.png"
          alt="World Background"
          fill
          className="object-contain object-bottom"
        />
      </div>

      <div className="w-full max-w-[1240px] mx-auto px-4 relative z-10 flex flex-col md:flex-row justify-between items-start gap-12">
        {/* Left Column */}
        <div className="flex flex-col gap-6">
          <Image
            src="/logo/agora logo footer.svg"
            alt="Agora Logo"
            width={180}
            height={54}
            className="w-auto h-12"
          />
          <p className="text-gray-400 text-sm">
            © 2026 agora. All rights reserved.
          </p>
        </div>

        {/* Right Columns Container */}
        <div className="flex gap-16 md:gap-24">
          {/* Nav Links Column */}
          <div className="flex flex-col gap-4">
            <Link
              href="#"
              className="text-gray-300 hover:text-white transition-colors"
            >
              Discover Events
            </Link>
            <Link
              href="#"
              className="text-gray-300 hover:text-white transition-colors"
            >
              Pricing
            </Link>
            <Link
              href="#"
              className="text-gray-300 hover:text-white transition-colors"
            >
              Stellar Ecosystem
            </Link>
            <Link
              href="#"
              className="text-gray-300 hover:text-white transition-colors"
            >
              FAQs
            </Link>
            <Link
              href="/settings#cookie-preferences"
              className="text-gray-300 hover:text-white transition-colors"
            >
              Cookie preferences
            </Link>
          </div>

          {/* Socials Column */}
          <div className="flex flex-col gap-4">
            {/* Instagram */}
            <a
              href="#"
              className="text-gray-300 hover:text-white transition-colors flex items-center gap-2 group"
            >
              <div className="w-5 h-5 flex items-center justify-center">
                <svg
                  width="20"
                  height="20"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <rect x="2" y="2" width="20" height="20" rx="5" ry="5"></rect>
                  <path d="M16 11.37A4 4 0 1 1 12.63 8 4 4 0 0 1 16 11.37z"></path>
                  <line x1="17.5" y1="6.5" x2="17.51" y2="6.5"></line>
                </svg>
              </div>
              <span className="text-sm">Instagram</span>
            </a>

            {/* X (Twitter) */}
            <a
              href="#"
              className="text-gray-300 hover:text-white transition-colors flex items-center gap-2 group"
            >
              <div className="w-5 h-5 flex items-center justify-center">
                <svg
                  width="20"
                  height="20"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <path d="M4 4l11.733 16h4.267l-11.733 -16z" />
                  <path d="M4 20l6.768 -6.768m2.46 -2.46l6.772 -6.772" />
                </svg>
              </div>
              <span className="text-sm">X</span>
            </a>

            {/* Mail */}
            <a
              href="#"
              className="text-gray-300 hover:text-white transition-colors flex items-center gap-2 group"
            >
              <div className="w-5 h-5 flex items-center justify-center">
                <svg
                  width="20"
                  height="20"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"></path>
                  <polyline points="22,6 12,13 2,6"></polyline>
                </svg>
              </div>
              <span className="text-sm">Mail</span>
            </a>

            {/* Stellar Ecosystem Link */}
            <a
              href="#"
              className="text-gray-300 hover:text-white transition-colors flex items-center gap-2 group"
            >
              <div className="w-5 h-5 rounded-full bg-white/10 flex items-center justify-center p-1">
                <Image
                  src="/icons/stellar-xlm-logo 1.svg"
                  alt="Stellar"
                  width={16}
                  height={16}
                  className="w-full h-full object-contain"
                />
              </div>
              <span className="text-sm">Stellar</span>
            </a>
          </div>
        </div>
      </div>
    </footer>
  );
}
