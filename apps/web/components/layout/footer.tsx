"use client";

import Image from "next/image";
import Link from "next/link";
import { useTranslations } from "next-intl";
import { LanguageSwitcher } from "./language-switcher";

/**
 * SOLUTION FOR ISSUE #448:
 * 1. Replaced all placeholder "#" hrefs with functional internal and external routes.
 * 2. Added `hover:bg-white/5` and `transition-all` to create a professional hover "pill" effect.
 * 3. Integrated `mailto:hello@agora.com` for the contact link.
 * 4. Ensured mobile responsiveness via existing `flex-col md:flex-row` logic.
 * 5. Translated all strings (and added the language switcher) in Issue #1343.
 */

export function Footer() {
  const t = useTranslations("footer");

  return (
    <footer className="w-full bg-ink pt-20 pb-12 relative overflow-hidden text-white select-none">
      {/* Background Graphic */}
      <div className="absolute -bottom-12 left-1/2 -translate-x-1/2 w-full max-w-[700px] h-[500px] pointer-events-none opacity-30 mix-blend-screen">
        <Image
          src="/images/World1.png"
          alt="World Background"
          fill
          className="object-contain object-bottom"
        />
      </div>

      <div className="w-full max-w-[1240px] mx-auto px-4 relative z-10 flex flex-col md:flex-row justify-between items-start gap-12">
        {/* Left Column: Branding & Copyright */}
        <div className="flex flex-col gap-6">
          <Image
            src="/logo/agora logo footer.svg"
            alt="Agora Logo"
            width={180}
            height={54}
            className="w-auto h-12"
          />
          <p className="text-gray-400 text-sm">{t("copyright")}</p>
          <LanguageSwitcher />
        </div>

        {/* Right Columns Container */}
        <div className="flex gap-16 md:gap-24">
          
          {/* Nav Links Column: Updated with real routes & hover styles */}
          <div className="flex flex-col gap-2">
            <Link
              href="/events"
              className="text-gray-300 hover:text-white hover:bg-white/5 px-3 py-1.5 -ml-3 rounded-md transition-all duration-200 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
            >
              {t("discoverEvents")}
            </Link>
            <Link
              href="/pricing"
              className="text-gray-300 hover:text-white hover:bg-white/5 px-3 py-1.5 -ml-3 rounded-md transition-all duration-200 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
            >
              {t("pricing")}
            </Link>
            <Link
              href="https://stellar.org"
              target="_blank"
              rel="noopener noreferrer"
              className="text-gray-300 hover:text-white hover:bg-white/5 px-3 py-1.5 -ml-3 rounded-md transition-all duration-200 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
            >
              {t("stellarEcosystem")}
            </Link>
            <Link
              href="/help"
              className="text-gray-300 hover:text-white hover:bg-white/5 px-3 py-1.5 -ml-3 rounded-md transition-all duration-200 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
            >
              {t("faqs")}
            </Link>
            <Link
              href="/settings#cookie-preferences"
              className="text-gray-300 hover:text-white transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
            >
              {t("cookiePreferences")}
            </Link>
          </div>

          {/* Socials Column: Updated with real handles & contact mail */}
          <div className="flex flex-col gap-3">
            {/* Instagram */}
            <a
              href="https://instagram.com/agora"
              target="_blank"
              rel="noopener noreferrer"
              className="text-gray-300 hover:text-white hover:bg-white/5 px-3 py-1.5 -ml-3 rounded-md transition-all duration-200 flex items-center gap-2 group focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
            >
              <Image src="/icons/instagram.svg" width={20} height={20} alt="Instagram" className="text-gray-300 group-hover:text-white" />
              <span className="text-sm">{t("instagram")}</span>
            </a>

            {/* X (Twitter) */}
            <a
              href="https://x.com/agora"
              target="_blank"
              rel="noopener noreferrer"
              className="text-gray-300 hover:text-white hover:bg-white/5 px-3 py-1.5 -ml-3 rounded-md transition-all duration-200 flex items-center gap-2 group focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
            >
              <Image src="/icons/x.svg" width={20} height={20} alt="X" className="text-gray-300 group-hover:text-white" />
              <span className="text-sm">{t("x")}</span>
            </a>

            {/* Mail: Integrated hello@agora.com */}
            <a
              href="mailto:hello@agora.com"
              className="text-gray-300 hover:text-white hover:bg-white/5 px-3 py-1.5 -ml-3 rounded-md transition-all duration-200 flex items-center gap-2 group focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
            >
              <Image src="/icons/mail.svg" width={20} height={20} alt="Mail" className="text-gray-300 group-hover:text-white" />
              <span className="text-sm">{t("mail")}</span>
            </a>

            {/* Stellar Link (Internal/Help) */}
            <Link
              href="/stellar"
              className="text-gray-300 hover:text-white hover:bg-white/5 px-3 py-1.5 -ml-3 rounded-md transition-all duration-200 flex items-center gap-2 group focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
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
              <span className="text-sm">{t("stellar")}</span>
            </Link>
          </div>
        </div>
      </div>
    </footer>
  );
}