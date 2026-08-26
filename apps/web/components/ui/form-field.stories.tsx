import type { Meta, StoryObj } from "@storybook/react";
import { FormField } from "./form-field";

const meta: Meta<typeof FormField> = {
  title: "UI/FormField",
  component: FormField,
  tags: ["autodocs"],
  parameters: {
    layout: "padded",
  },
  argTypes: {
    label: { control: "text" },
    name: { control: "text" },
    placeholder: { control: "text" },
    error: { control: "text" },
    disabled: { control: "boolean" },
    required: { control: "boolean" },
  },
};

export default meta;
type Story = StoryObj<typeof FormField>;

export const Default: Story = {
  args: {
    label: "Email Address",
    name: "email",
    type: "email",
    placeholder: "you@example.com",
  },
};

export const WithError: Story = {
  name: "With error",
  args: {
    label: "Email Address",
    name: "email",
    type: "email",
    value: "invalid-email",
    error: "Please enter a valid email address.",
  },
};

export const Disabled: Story = {
  args: {
    label: "Wallet Address",
    name: "wallet",
    value: "GABC123...XYZ890",
    disabled: true,
  },
};

export const Required: Story = {
  args: {
    label: "Event Name",
    name: "eventName",
    placeholder: "e.g. Stellar Hackathon",
    required: true,
  },
};
