"use client";

import Image from "next/image";
import { Button } from "@/components/ui/button";
import { motion } from "framer-motion";

export function PricingSection() {
  const fadeInUp = {
    hidden: { opacity: 0, y: 50 },
    visible: {
      opacity: 1,
      y: 0,
      transition: { duration: 0.6, ease: "easeOut" as const },
    },
  };

  return (
    <section id="pricing" className="w-full bg-[#fffbe9] py-24 select-none">
      <div className="w-full max-w-[1240px] mx-auto px-4 flex flex-col items-center">
        {/* Pill */}
        <motion.div
          initial="hidden"
          whileInView="visible"
          viewport={{ once: true }}
          variants={fadeInUp}
          className="bg-[#FDDA23] text-black px-6 py-2 rounded-full font-medium text-sm mb-12"
        >
          Pricing Plan
        </motion.div>

        {/* Cards Container */}
        <div className="flex flex-col md:flex-row gap-8 items-center md:items-stretch justify-center w-full">
          {/* --- Free Card --- */}
          <motion.div
            initial="hidden"
            whileInView="visible"
            viewport={{ once: true }}
            variants={fadeInUp}
            className="w-full max-w-[400px] bg-white rounded-3xl p-8 border border-black/5 shadow-xl flex flex-col relative overflow-hidden"
          >
            {/* gradients */}
            <div
              className="absolute top-[40%] -left-[10%] w-[50%] h-[40%] rounded-full blur-[80px]"
              style={{
                background:
                  "linear-gradient(186.86deg, rgba(248, 184, 188, 0.5) 1.47%, rgba(255, 114, 114, 0.5) 25.55%, rgba(190, 228, 190, 0.5) 72.73%)",
              }}
            />
            <div
              className="absolute top-[40%] -right-[50%] w-[70%] h-[30%] rounded-full blur-[50px]"
              style={{
                background:
                  "linear-gradient(186.52deg, rgba(254, 241, 195, 0.5) 0.94%, rgba(253, 218, 35, 0.5) 25.16%, rgba(253, 218, 35, 0.5) 72.59%)",
              }}
            />

            <div className="flex-1 relative z-10">
              <h3 className="italic font-light text-black mb-6 text-xl">
                agora basic
              </h3>
              <h2 className="text-4xl font-semibold mb-2 text-black">Free</h2>
              <div className="text-4xl font-semibold mb-8 text-black">
                $0{" "}
                <span className="text-lg font-normal text-gray-500">
                  / forever
                </span>
              </div>

              <ul className="space-y-4 mb-8 font-medium text-xl">
                <ListItem text="Unlimited free events" dark={false} />
                <ListItem text="Up to 3 paid events/month" dark={false} />
                <ListItem text="5% Platform fee" dark={false} />
                <ListItem text="Basic analytics" dark={false} />
                <ListItem text="Standard payouts (24-48hrs)" dark={false} />
              </ul>
            </div>

            <div className="relative z-10 flex justify-center">
              <Button
                className="w-[215px]"
                backgroundColor="bg-[#FDDA23]"
                textColor="text-black"
                shadowColor="rgba(0,0,0,1)"
              >
                <span className="text-sm font-semibold">Get Started</span>
                <Image
                  src="/icons/dollar-circle.svg"
                  alt="Free"
                  width={24}
                  height={24}
                />
              </Button>
            </div>
          </motion.div>

          {/* --- Pro Card --- */}
          <motion.div
            initial="hidden"
            whileInView="visible"
            viewport={{ once: true }}
            variants={fadeInUp}
            className="w-full max-w-[400px] bg-[#060606] rounded-3xl p-8 relative overflow-hidden flex flex-col"
          >
            <div className="absolute top-0 left-0 w-full h-full pointer-events-none opacity-60 bg-[#060606]" />
            {/* gradients */}
            <div
              className="absolute top-[40%] -left-[10%] w-[50%] h-[40%] rounded-full blur-[100px]"
              style={{
                background:
                  "linear-gradient(186.86deg, rgba(248, 184, 188, 0.5) 1.47%, rgba(255, 114, 114, 0.5) 25.55%, rgba(190, 228, 190, 0.5) 72.73%)",
              }}
            />
            <div
              className="absolute top-[40%] -right-[50%] w-[70%] h-[30%] rounded-full blur-[100px]"
              style={{
                background:
                  "linear-gradient(186.52deg, rgba(254, 241, 195, 0.5) 0.94%, rgba(253, 218, 35, 0.5) 25.16%, rgba(253, 218, 35, 0.5) 72.59%)",
              }}
            />

            <div className="relative z-10 flex-1">
              <h3 className="italic font-light text-white mb-6 text-xl">
                agora plus
              </h3>
              <h2 className="text-4xl font-semibold mb-2 text-white">Pro</h2>
              <div className="text-4xl font-semibold mb-8 text-white">
                $29{" "}
                <span className="text-lg font-normal text-gray-400">
                  / month or $290 / year
                </span>
              </div>

              <ul className="space-y-4 mb-8 font-medium text-xl">
                <ListItem text="Unlimited paid events" dark={true} />
                <ListItem text="0% platform fee" dark={true} />
                <ListItem text="Advanced analytics" dark={true} />
                <ListItem text="Custom branding" dark={true} />
                <ListItem text="Early payouts (12h)" dark={true} />
              </ul>
            </div>

            <div className="relative z-10 flex justify-center">
              <Button
                className="w-[215px]"
                backgroundColor="bg-[#FDDA23]"
                textColor="text-black"
                shadowColor="rgba(255,255,255,1)"
              >
                <span className="text-sm font-semibold">
                  Get Started for $29
                </span>
                <Image
                  src="/icons/dollar-circle.svg"
                  alt="Pro"
                  width={24}
                  height={24}
                />
              </Button>
            </div>
          </motion.div>
        </div>
      </div>
    </section>
  );
}

function ListItem({ text, dark }: { text: string; dark: boolean }) {
  return (
    <li
      className={`flex items-center gap-3 ${dark ? "text-white" : "text-black"}`}
    >
      <Image
        src={
          dark
            ? "/icons/checkmark-circle-01.svg"
            : "/icons/checkmark-circle-01.svg"
        }
        alt="Check"
        width={24}
        height={24}
        className={dark ? "" : "invert"}
      />
      <span className="text-lg">{text}</span>
    </li>
  );
}
