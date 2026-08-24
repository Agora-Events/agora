"use client";

import { useState } from "react";
import { SellTicketModal } from "@/components/events/sell-ticket-modal";
import { Button } from "@/components/ui/button";

interface SellTicketSectionProps {
  event: {
    id: number;
    title: string;
    date: string;
    location: string;
    price: string;
  };
}

/**
 * SellTicketSection — a client component that renders the "Sell Ticket"
 * button and manages the SellTicketModal state.
 */
export function SellTicketSection({ event }: SellTicketSectionProps) {
  const [isModalOpen, setIsModalOpen] = useState(false);

  return (
    <>
      <Button
        variant="primary"
        onClick={() => setIsModalOpen(true)}
        backgroundColor="bg-black"
        textColor="text-white"
        className="h-11 px-6 rounded-full text-sm whitespace-nowrap"
        aria-label="Sell your tickets on the secondary marketplace"
      >
        Sell Ticket
      </Button>

      <SellTicketModal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        event={event}
        ticketQuantity={1}
      />
    </>
  );
}