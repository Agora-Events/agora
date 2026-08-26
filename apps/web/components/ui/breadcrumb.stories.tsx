import type { Meta, StoryObj } from "@storybook/react";
import { Breadcrumb } from "./breadcrumb";

const meta: Meta<typeof Breadcrumb> = {
  title: "UI/Breadcrumb",
  component: Breadcrumb,
  tags: ["autodocs"],
  parameters: {
    layout: "padded",
  },
};

export default meta;
type Story = StoryObj<typeof Breadcrumb>;

export const TwoLevel: Story = {
  name: "2-level",
  args: {
    items: [
      { label: "Home", href: "/" },
      { label: "Help Center" },
    ],
  },
};

export const FourLevel: Story = {
  name: "4-level",
  args: {
    items: [
      { label: "Home", href: "/" },
      { label: "Events", href: "/events" },
      { label: "Tech", href: "/events/category/tech" },
      { label: "Stellar Summit 2026" },
    ],
  },
};

export const TruncatedLabel: Story = {
  name: "Truncated-label",
  args: {
    items: [
      { label: "Home", href: "/" },
      { label: "Organizers", href: "/organizers" },
      {
        label: "International Web3 & Blockchain Technology Conference & Global Developers Summit 2026",
      },
    ],
  },
};
