"use client";

import { useState, useEffect } from "react";
import Image from "next/image";
import { usePathname } from "next/navigation";
import { Button } from "@/components/ui/button";
import { motion, AnimatePresence } from "framer-motion";

// Sub-components
import { GuestNav } from "./navbar/guest-nav";
import { UserNav } from "./navbar/user-nav";
import { MobileNavLink } from "./navbar/mobile-nav-link";

export function Navbar() {
  const pathname = usePathname();
  const [isOpen, setIsOpen] = useState(false);
  const [isLoggedIn, setIsLoggedIn] = useState(false);

  // Lock body scroll when menu is open
  useEffect(() => {
    if (isOpen) {
      document.body.style.overflow = "hidden";
    } else {
      document.body.style.overflow = "unset";
    }
    return () => {
      document.body.style.overflow = "unset";
    };
  }, [isOpen]);

  const toggleMenu = () => setIsOpen(!isOpen);

  const menuVariants = {
    closed: {
      x: "100%",
      transition: {
        type: "spring" as const,
        stiffness: 400,
        damping: 40,
      },
    },
    open: {
      x: "0%",
      transition: {
        type: "spring" as const,
        stiffness: 400,
        damping: 40,
      },
    },
  };

  const linkVariants = {
    closed: { opacity: 0, y: 20 },
    open: (i: number) => ({
      opacity: 1,
      y: 0,
      transition: {
        delay: i * 0.1 + 0.2,
        duration: 0.4,
        ease: "easeOut" as const,
      },
    }),
  };

  return (
    <>
      <nav className="w-full max-w-[1221px] h-[56px] mt-[35px] mx-auto flex px-4 lg:px-0 items-center justify-between relative z-50">
        {isLoggedIn ? (
          <UserNav pathname={pathname} />
        ) : (
          <GuestNav pathname={pathname} />
        )}

        <div className="flex items-center lg:hidden">
          <button
            onClick={toggleMenu}
            className="z-50 flex flex-col justify-center items-center w-12 h-12 rounded-full bg-white/10 backdrop-blur-md border border-black/10 hover:bg-white/20 transition-colors"
            aria-label="Toggle Menu"
          >
            <div className="w-6 h-6 flex flex-col justify-center gap-[5px]">
              <motion.span
                animate={isOpen ? { rotate: 45, y: 7 } : { rotate: 0, y: 0 }}
                className="w-full h-[2px] bg-black rounded-full origin-center"
              />
              <motion.span
                animate={isOpen ? { opacity: 0 } : { opacity: 1 }}
                className="w-full h-[2px] bg-black rounded-full"
              />
              <motion.span
                animate={isOpen ? { rotate: -45, y: -7 } : { rotate: 0, y: 0 }}
                className="w-full h-[2px] bg-black rounded-full origin-center"
              />
            </div>
          </button>
        </div>

      </nav>

      <AnimatePresence>
        {isOpen && (
          <>
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              onClick={toggleMenu}
              className="fixed inset-0 bg-black/20 backdrop-blur-sm z-40 lg:hidden"
            />

            <motion.div
              variants={menuVariants}
              initial="closed"
              animate="open"
              exit="closed"
              className="fixed top-0 right-0 h-full w-[300px] bg-white z-50 shadow-2xl flex flex-col p-8 pt-24 lg:hidden"
            >
              <button
                onClick={toggleMenu}
                className="absolute top-6 right-6 p-2 rounded-full hover:bg-gray-100 transition-colors"
                aria-label="Close Menu"
              >
                <svg
                  width="24"
                  height="24"
                  viewBox="0 0 24 24"
                  fill="none"
                  xmlns="http://www.w3.org/2000/svg"
                >
                  <path
                    d="M18 6L6 18M6 6L18 18"
                    stroke="black"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  />
                </svg>
              </button>

              <div className="flex flex-col gap-6">
                {isLoggedIn ? (
                  <>
                    <MobileNavLink
                      i={0}
                      href="/"
                      icon="/icons/home.svg"
                      text="Home"
                      isActive={pathname === "/"}
                    />
                    <MobileNavLink
                      i={1}
                      href="/discover"
                      icon="/icons/earth-yellow.svg"
                      text="Discover Events"
                      isActive={pathname === "/discover"}
                    />
                    <MobileNavLink
                      i={2}
                      href="/organizers"
                      icon="/icons/user-group.svg"
                      text="Organizers"
                      isActive={pathname === "/organizers"}
                    />
                    <MobileNavLink
                      i={3}
                      href="/stellar"
                      icon="/icons/stellar-xlm-logo 1.svg"
                      text="Stellar Ecosystem"
                      isActive={pathname === "/stellar"}
                    />
                  </>
                ) : (
                  <>
                    <MobileNavLink
                      i={0}
                      href="/discover"
                      icon="/icons/earth.svg"
                      text="Discover Events"
                      isActive={pathname === "/discover"}
                    />
                    <MobileNavLink
                      i={1}
                      href="/pricing"
                      icon="/icons/dollar-circle.svg"
                      text="Pricing"
                      isActive={pathname === "/pricing"}
                    />
                    <MobileNavLink
                      i={2}
                      href="/stellar"
                      icon="/icons/stellar-xlm-logo 1.svg"
                      text="Stellar Ecosystem"
                      isActive={pathname === "/stellar"}
                    />
                    <MobileNavLink
                      i={3}
                      href="/faqs"
                      icon="/icons/help-circle.svg"
                      text="FAQs"
                      isActive={pathname === "/faqs"}
                    />
                  </>
                )}

                <motion.div custom={4} variants={linkVariants} className="mt-4">
                  <Button
                    className="w-full justify-center"
                    backgroundColor="bg-black"
                    textColor="text-white"
                    shadowColor="rgba(0,0,0,0.5)"
                  >
                    <span>Create Your Event</span>
                    <Image
                      src="/icons/arrow-up-right-01.svg"
                      alt="Arrow"
                      width={24}
                      height={24}
                      className="invert group-hover:translate-x-0.5 group-hover:-translate-y-0.5 transition-transform"
                    />
                  </Button>
                </motion.div>
              </div>
            </motion.div>
          </>
        )}
      </AnimatePresence>
    </>
  );
}
