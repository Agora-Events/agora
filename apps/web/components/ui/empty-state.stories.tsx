import type { Meta, StoryObj } from "@storybook/react";
import { EmptyState } from "./empty-state";

const meta: Meta<typeof EmptyState> = {
  title: "UI/EmptyState",
  component: EmptyState,
  tags: ["autodocs"],
  parameters: {
    layout: "padded",
  },
};

export default meta;
type Story = StoryObj<typeof EmptyState>;

export const Default: Story = {
  args: {
    title: "No Events Found",
    description: "There are currently no events scheduled. Check back later or create a new event.",
  },
};

export const WithAction: Story = {
  name: "With action",
  args: {
    title: "No Tickets Purchased",
    description: "You haven't bought any tickets yet. Explore upcoming events and get yours!",
    action: {
      label: "Explore Events",
      href: "/discover",
    },
  },
};

export const LongDescription: Story = {
  name: "Long description",
  args: {
    title: "No Recommendations Available At This Time",
    description:
      "We couldn't find any recommendations matching your preferences right now. Try updating your profile interests, following more event organizers, or broadening your search filters to discover exciting events happening around you.",
  },
};
