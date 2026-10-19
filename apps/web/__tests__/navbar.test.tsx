import { render, screen, fireEvent } from "@testing-library/react";
import { expect, afterEach, describe, it, vi } from "vitest";
import { cleanup } from "@testing-library/react";
import "@testing-library/jest-dom";
import { Navbar } from "@/components/layout/navbar";
import { useState } from "react";
import Image from "next/image";

// Mock next/image
vi.mock("next/image", () => ({
  default: ({ src, alt, width, height }: { src: string; alt: string; width: number; height: number }) => (
    <Image src={src} alt={alt} width={width} height={height} />
  ),
}));

// Mock next/navigation
vi.mock("next/navigation", () => ({
  usePathname: vi.fn(() => "/home"),
  useRouter: vi.fn(),
}));

// Mock framer-motion
vi.mock("framer-motion", () => ({
  motion: {
    div: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
    span: ({ children }: { children: React.ReactNode }) => <span>{children}</span>,
  },
  AnimatePresence: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

// Mock Overlay
vi.mock("@/components/ui/overlay", () => ({
  Overlay: ({ isOpen, onClose }: { isOpen: boolean; onClose: () => void }) =>
    isOpen ? <div data-testid="overlay" onClick={onClose} /> : null,
}));

// Mock CSS module
vi.mock("./navbar/drawer.module.css", () => ({
  default: {
    drawer: "drawer",
    drawerOpen: "drawer-open",
  },
}));

describe("Navbar", () => {
  afterEach(() => {
    cleanup();
  });

  it("renders correctly with logged-in user", () => {
    // Mock isLoggedIn to be true
    vi.mocked(useState).mockReturnValue([true, vi.fn()]);
    
    render(<Navbar />);
    
    // Check that UserNav is rendered (not GuestNav)
    expect(screen.getByText(/Home/i)).toBeInTheDocument();
    expect(screen.getByText(/Discover Events/i)).toBeInTheDocument();
  });

  it("renders correctly with guest user", () => {
    // Mock isLoggedIn to be false
    vi.mocked(useState).mockReturnValue([false, vi.fn()]);
    
    render(<Navbar />);
    
    // Check that GuestNav is rendered (not UserNav)
    expect(screen.getByText(/Discover Events/i)).toBeInTheDocument();
    expect(screen.getByText(/Pricing/i)).toBeInTheDocument();
  });

  it("toggles mobile menu when button is clicked", () => {
    // Mock isLoggedIn to be true
    vi.mocked(useState).mockReturnValue([true, vi.fn()]);
    
    render(<Navbar />);
    
    // Find and click the mobile menu button
    const menuButton = screen.getByRole("button", { name: /Toggle Menu/i });
    fireEvent.click(menuButton);
    
    // Check that mobile menu is visible
    expect(screen.getByText(/Home/i)).toBeInTheDocument();
    expect(screen.getByText(/Discover Events/i)).toBeInTheDocument();
  });
});