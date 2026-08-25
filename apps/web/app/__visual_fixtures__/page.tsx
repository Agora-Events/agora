/**
 * Visual regression fixture page — only rendered during Playwright tests.
 *
 * Renders Button, EventCard, RegistrationBox (static), and TicketModal in
 * isolated sections so each snapshot captures exactly one component.
 *
 * Route: /visual-fixtures
 * Never linked from the main app; excluded from sitemap.
 */

"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { EventCard } from "@/components/events/event-card";
import { TicketModal } from "@/components/events/TicketModal";
import Image from "next/image";

// ── Minimal RegistrationBox stand-in ─────────────────────────────────────────
// We render the visual surface of RegistrationBox without its router/auth
// dependencies so the fixture page stays static-renderable.

function RegistrationBoxFixture({ soldOut = false }: { soldOut?: boolean }) {
  return (
    <div
      className="bg-surface rounded-3xl p-6 sm:p-8 flex flex-col gap-8 relative overflow-hidden border border-black/5 shadow-sm"
      data-testid="registration-box"
    >
      <div className="flex justify-between items-center flex-wrap gap-4">
        <div className="bg-white rounded-full px-6 py-2.5 italic text-gray-400 font-medium text-[17px] shadow-sm">
          Registration
        </div>
        <div className="flex items-center gap-3">
          {["−", "1", "+"].map((label, i) => (
            <div
              key={i}
              className="w-12 h-12 rounded-full bg-white border border-black/5 shadow-sm flex items-center justify-center text-xl font-bold"
            >
              {label}
            </div>
          ))}
        </div>
      </div>

      <p className="text-[19px] text-black font-medium">
        Welcome! To join the event, please register below.
      </p>

      <div className="flex items-center justify-between gap-4 flex-wrap">
        <Button
          variant="primary"
          className="h-16 px-10 rounded-full text-[22px]"
          disabled={soldOut}
        >
          {soldOut ? (
            <>
              <span>Join Waitlist</span>
            </>
          ) : (
            <>
              <Image src="/icons/dollar-circle.svg" width={28} height={28} alt="" aria-hidden="true" />
              <span>$49.00</span>
              <Image src="/icons/arrow-up-right-01.svg" width={24} height={24} alt="" aria-hidden="true" />
            </>
          )}
        </Button>
        <div className="flex items-center gap-3">
          <div className="relative w-14 h-14 rounded-full border-2 border-black overflow-hidden bg-white shadow-[3px_3px_0px_0px_rgba(0,0,0,1)]">
            <Image src="/images/pfp.png" fill alt="Host" className="object-cover" />
          </div>
          <span className="text-[18px] italic font-medium text-black">Daniel James</span>
        </div>
      </div>

      <div className="absolute -right-8 -bottom-8 opacity-[0.06] scale-150 pointer-events-none rotate-12">
        <Image src="/icons/stellar-logo.svg" width={240} height={240} alt="" aria-hidden="true" />
      </div>
    </div>
  );
}

// ── Fixture sections ──────────────────────────────────────────────────────────

const MOCK_EVENT = {
  id: 1,
  title: "Stellar Developer Summit",
  date: "Thu, 22 Jan · 1:00 PM",
  location: "Online",
  price: "49",
  imageUrl: "/images/event1.png",
};

export default function VisualFixturesPage() {
  const [modalOpen, setModalOpen] = useState(false);
  const [soldOutModalOpen, setSoldOutModalOpen] = useState(false);

  return (
    <div className="min-h-screen bg-base p-8 flex flex-col gap-16">

      {/* ── Button variants ───────────────────────────────────────────────── */}
      <section data-testid="fixture-button" className="flex flex-col gap-6 max-w-lg">
        <h2 className="text-2xl font-extrabold">Button</h2>
        <div className="flex flex-wrap gap-4">
          <Button variant="primary">Primary Button</Button>
          <Button variant="secondary">Secondary Button</Button>
          <Button
            backgroundColor="bg-accent"
            textColor="text-black"
            shadowColor="rgba(0,0,0,1)"
          >
            Accent Button
          </Button>
          <Button
            backgroundColor="bg-ink"
            textColor="text-white"
            shadowColor="rgba(0,0,0,0.5)"
          >
            Dark Button
          </Button>
          <Button disabled>Disabled</Button>
          <Button isLoading>Loading</Button>
        </div>
      </section>

      {/* ── EventCard ─────────────────────────────────────────────────────── */}
      <section data-testid="fixture-event-card" className="flex flex-col gap-6 max-w-xl">
        <h2 className="text-2xl font-extrabold">EventCard</h2>
        <EventCard
          id={1}
          title="Stellar Developer Summit"
          date="Thu, 22 Jan · 1:00 PM"
          location="Online"
          price="Free"
          imageUrl="/images/event1.png"
        />
        <EventCard
          id={2}
          title="React Summit 2026 — A Very Long Event Title That Should Wrap Correctly"
          date="Mon, 10 Feb · 9:00 AM"
          location="London, UK"
          price="49"
          imageUrl="/images/event2.png"
        />
      </section>

      {/* ── RegistrationBox ───────────────────────────────────────────────── */}
      <section data-testid="fixture-registration-box" className="flex flex-col gap-6 max-w-xl">
        <h2 className="text-2xl font-extrabold">RegistrationBox</h2>
        <RegistrationBoxFixture />
        <h3 className="text-lg font-bold mt-2">Sold-out / Waitlist state</h3>
        <RegistrationBoxFixture soldOut />
      </section>

      {/* ── TicketModal trigger ───────────────────────────────────────────── */}
      <section data-testid="fixture-ticket-modal" className="flex flex-col gap-6 max-w-xl">
        <h2 className="text-2xl font-extrabold">TicketModal</h2>

        <div className="flex gap-4 flex-wrap">
          <Button
            variant="primary"
            onClick={() => setModalOpen(true)}
            data-testid="open-ticket-modal"
          >
            Open Purchase Modal
          </Button>
          <Button
            variant="secondary"
            onClick={() => setSoldOutModalOpen(true)}
            data-testid="open-waitlist-modal"
          >
            Open Waitlist Modal (Sold Out)
          </Button>
        </div>

        <TicketModal
          isOpen={modalOpen}
          onClose={() => setModalOpen(false)}
          event={{ ...MOCK_EVENT, availableQuantity: 10 }}
          initialQuantity={1}
        />
        <TicketModal
          isOpen={soldOutModalOpen}
          onClose={() => setSoldOutModalOpen(false)}
          event={{ ...MOCK_EVENT, availableQuantity: 0 }}
          initialQuantity={1}
        />
      </section>

    </div>
  );
}
