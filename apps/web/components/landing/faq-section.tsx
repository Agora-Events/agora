"use client";

import { useState } from "react";
import Image from "next/image";
import { motion, AnimatePresence } from "framer-motion";

export function FAQSection() {
  return (
    <section id="faqs" className="w-full bg-[#fffbe9] pb-24 select-none">
      <div className="w-full max-w-[1240px] mx-auto px-4 flex flex-col items-center">
        <div className="bg-[#FDDA23] text-black px-6 py-2 rounded-full font-medium text-sm mb-16">
          FAQs
        </div>

        <div className="w-full flex flex-col lg:flex-row gap-12 lg:gap-20 items-start">
          {/* Left Image */}
          <div className="w-full lg:w-1/2 flex justify-center lg:justify-start">
            {/* Desktop Image (Vertical) */}
            <div className="relative w-full max-w-[334px] hidden lg:block">
              <Image
                src="/images/FAQs.png"
                alt="FAQs"
                width={334}
                height={554}
                className="object-contain w-full h-auto"
              />
            </div>

            {/* Mobile Image (Horizontal) */}
            <div className="relative w-full block lg:hidden justify-center items-center">
              <Image
                src="/images/FAQs-horizontal.png"
                alt="FAQs"
                width={500} // Assuming roughly this width, layout will constrain it
                height={300}
                className="object-contain w-full h-auto max-w-[500px] mx-auto"
              />
            </div>
          </div>

          {/* Right Content - Accordion */}
          <div className="w-full lg:w-2/2 flex flex-col gap-4">
            <FAQItem
              question="How do I create an event on Agora?"
              answer="Simply click the 'Create Event' button, fill in your event details, set your ticket prices (or make it free), and publish. Your event page will be live instantly."
            />
            <FAQItem
              question="What are the fees for paid events?"
              answer="For free events, Agora is completely free. For paid events, we charge a small service fee. If you are on the Pro plan, there is a 0% platform fee."
            />
            <FAQItem
              question="How do I get paid?"
              answer="Payouts are processed automatically via Stellar USDC. You can connect your wallet and receive funds directly, often within hours of your event's completion."
            />
            <FAQItem
              question="Can I customize my event page?"
              answer="Yes! You can upload custom banners, add detailed descriptions, and manage your brand settings to make your event page look exactly how you want."
            />
            <FAQItem
              question="Is there support if I have issues?"
              answer="Absolutely. We offer 24/7 support for all organizers. Pro plan members get priority support and dedicated account assistance."
            />
          </div>
        </div>
      </div>
    </section>
  );
}

function FAQItem({ question, answer }: { question: string; answer: string }) {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <div className="w-full bg-[#060606] rounded-2xl overflow-hidden transition-all duration-300">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="w-full flex items-center justify-between p-6 text-left"
      >
        <span className="text-white font-semibold text-lg">{question}</span>
        <Image
          src={isOpen ? "/icons/remove-circle.svg" : "/icons/add-circle.svg"}
          alt="toggle"
          width={24}
          height={24}
          className="shrink-0 transition-transform duration-300"
        />
      </button>

      <AnimatePresence>
        {isOpen && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: "auto", opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.3, ease: "easeInOut" }}
            className="overflow-hidden"
          >
            <div className="p-6 pt-0 text-gray-300 leading-relaxed">
              {answer}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
