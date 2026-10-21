import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import {
  NetworkSwitcher,
  NetworkSwitcherProvider,
  RPC_URLS,
  useNetworkSwitcher,
} from "@/components/layout/network-switcher";

function ActiveRpcProbe() {
  const { activeNetwork, activeRpcUrl } = useNetworkSwitcher();
  return (
    <div>
      <span data-testid="active-network">{activeNetwork}</span>
      <span data-testid="active-rpc">{activeRpcUrl}</span>
    </div>
  );
}

function renderSwitcher(defaultNetwork?: "testnet" | "mainnet") {
  return render(
    <NetworkSwitcherProvider defaultNetwork={defaultNetwork}>
      <NetworkSwitcher />
      <ActiveRpcProbe />
    </NetworkSwitcherProvider>,
  );
}

describe("NetworkSwitcher", () => {
  it("defaults the active RPC state to testnet", () => {
    renderSwitcher();
    expect(screen.getByTestId("active-network")).toHaveTextContent("testnet");
    expect(screen.getByTestId("active-rpc")).toHaveTextContent(RPC_URLS.testnet);
  });

  it("toggles the active RPC state in the context provider when a network is selected", () => {
    renderSwitcher();

    fireEvent.click(screen.getByRole("button", { name: "mainnet" }));

    expect(screen.getByTestId("active-network")).toHaveTextContent("mainnet");
    expect(screen.getByTestId("active-rpc")).toHaveTextContent(RPC_URLS.mainnet);
  });

  it("reflects the active network through aria-pressed", () => {
    renderSwitcher();

    expect(screen.getByRole("button", { name: "testnet" })).toHaveAttribute(
      "aria-pressed",
      "true",
    );
    expect(screen.getByRole("button", { name: "mainnet" })).toHaveAttribute(
      "aria-pressed",
      "false",
    );

    fireEvent.click(screen.getByRole("button", { name: "mainnet" }));

    expect(screen.getByRole("button", { name: "testnet" })).toHaveAttribute(
      "aria-pressed",
      "false",
    );
    expect(screen.getByRole("button", { name: "mainnet" })).toHaveAttribute(
      "aria-pressed",
      "true",
    );
  });

  it("honours a non-default initial network", () => {
    renderSwitcher("mainnet");
    expect(screen.getByTestId("active-rpc")).toHaveTextContent(RPC_URLS.mainnet);
  });

  it("exposes toggleNetwork to flip the active RPC state", () => {
    function ToggleProbe() {
      const { activeRpcUrl, toggleNetwork } = useNetworkSwitcher();
      return (
        <button type="button" onClick={toggleNetwork}>
          {activeRpcUrl}
        </button>
      );
    }

    render(
      <NetworkSwitcherProvider>
        <ToggleProbe />
      </NetworkSwitcherProvider>,
    );

    const button = screen.getByRole("button");
    expect(button).toHaveTextContent(RPC_URLS.testnet);
    fireEvent.click(button);
    expect(button).toHaveTextContent(RPC_URLS.mainnet);
  });
});
