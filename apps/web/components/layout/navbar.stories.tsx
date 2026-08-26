import type { Meta, StoryObj } from "@storybook/react";
import { GuestNav } from "./navbar/guest-nav";

// GuestNav is the simpler variant — doesn't require auth context.
// For full Navbar (user-authenticated), see UserNav stories.
const meta: Meta<typeof GuestNav> = {
  title: "Layout/Navbar",
  component: GuestNav,
  tags: ["autodocs"],
  parameters: {
    layout: "fullscreen",
    nextjs: {
      navigation: {
        pathname: "/discover",
      },
    },
  },
};

export default meta;
type Story = StoryObj<typeof GuestNav>;

export const Guest: Story = {};

export const OnHomePage: Story = {
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/",
      },
    },
  },
};

export const OnDiscoverPage: Story = {
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/discover",
      },
    },
  },
};
