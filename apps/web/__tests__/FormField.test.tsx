import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { FormField } from "@/components/ui/form-field";

describe("FormField", () => {
  it("associates the label with the input", () => {
    render(<FormField label="Email" name="email" />);

    const input = screen.getByLabelText("Email");
    expect(input).toBeInstanceOf(HTMLInputElement);
    expect(input).toHaveAttribute("id", "email");
    expect(input).toHaveAttribute("name", "email");
  });

  it("marks required fields and keeps the label linked", () => {
    render(<FormField label="Email" name="email" required />);

    expect(screen.getByLabelText(/Email/)).toBeRequired();
    expect(screen.getByText("*")).toBeInTheDocument();
  });

  it("is not marked invalid when there is no error", () => {
    render(<FormField label="Email" name="email" />);

    const input = screen.getByLabelText("Email");
    expect(input).not.toHaveAttribute("aria-invalid");
    expect(input).not.toHaveAttribute("aria-describedby");
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("renders the error and wires aria-invalid / aria-describedby to it", () => {
    render(<FormField label="Email" name="email" error="Enter a valid email" />);

    const input = screen.getByLabelText("Email");
    const error = screen.getByRole("alert");

    expect(error).toHaveTextContent("Enter a valid email");
    expect(input).toHaveAttribute("aria-invalid", "true");
    expect(error).toHaveAttribute("id");
    expect(input).toHaveAttribute("aria-describedby", error.id);
    expect(input).toHaveAccessibleDescription("Enter a valid email");
  });

  it("passes type, placeholder and onChange through to the input", () => {
    const onChange = vi.fn();
    render(
      <FormField
        label="Password"
        name="password"
        type="password"
        placeholder="Enter password"
        onChange={onChange}
      />
    );

    const input = screen.getByLabelText("Password");
    expect(input).toHaveAttribute("type", "password");
    expect(input).toHaveAttribute("placeholder", "Enter password");

    fireEvent.change(input, { target: { value: "hunter2" } });
    expect(onChange).toHaveBeenCalledTimes(1);
  });

  it("forwards extra input attributes and disabled state", () => {
    render(<FormField label="Username" name="username" disabled autoComplete="username" />);

    const input = screen.getByLabelText("Username");
    expect(input).toBeDisabled();
    expect(input).toHaveAttribute("autocomplete", "username");
  });
});
