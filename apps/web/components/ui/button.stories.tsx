import type { Meta, StoryObj } from "@storybook/nextjs";
import { Button } from "./button";

const meta: Meta<typeof Button> = {
  title: "UI/Button",
  component: Button,
  tags: ["autodocs"],
  argTypes: {
    variant: {
      control: "select",
      options: ["primary", "secondary", "dark", "ghost"],
    },
    children: { control: "text" },
    disabled: { control: "boolean" },
  },
};

export default meta;
type Story = StoryObj<typeof Button>;

export const Primary: Story = {
  args: {
    variant: "primary",
    children: "Create Event",
  },
};

export const Secondary: Story = {
  args: {
    variant: "secondary",
    children: "Learn More",
  },
};

export const Dark: Story = {
  args: {
    variant: "primary",
    children: "Filter",
  },
};

export const Ghost: Story = {
  args: {
    variant: "secondary",
    children: "Cancel",
  },
};

export const Disabled: Story = {
  args: {
    variant: "primary",
    children: "Disabled",
    disabled: true,
  },
};
