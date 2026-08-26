import type { Meta, StoryObj } from "@storybook/react";
import { EventCard } from "./event-card";

const meta: Meta<typeof EventCard> = {
  title: "Events/EventCard",
  component: EventCard,
  tags: ["autodocs"],
  parameters: {
    layout: "padded",
  },
  argTypes: {
    price: { control: "text" },
    loading: { control: "boolean" },
  },
};

export default meta;
type Story = StoryObj<typeof EventCard>;

const baseArgs = {
  id: "1",
  title: "Stellar Dev Meetup — Lagos Edition",
  date: "Thu, 22 Jan, 1:00 PM",
  location: "Victoria Island, Lagos",
  imageUrl: "/og-image.png",
};

export const Default: Story = {
  args: {
    ...baseArgs,
    price: "25",
  },
};

export const FreeEvent: Story = {
  args: {
    ...baseArgs,
    price: "free",
    title: "Agora Community Hangout",
  },
};

export const Loading: Story = {
  args: {
    ...baseArgs,
    price: "10",
    loading: true,
  },
};

export const LongTitle: Story = {
  args: {
    ...baseArgs,
    price: "50",
    title:
      "International Web3 & Blockchain Developers Conference — Annual Summit 2026",
  },
};

export const DiscordEvent: Story = {
  args: {
    ...baseArgs,
    price: "free",
    location: "Discord Server",
    title: "Web3 AMA with Stellar Foundation",
  },
};
