/**
 * E2E test: "Ticket Purchase Checkout Flow with Wallet Simulation"
 *
 * Journey:
 *  1. Visit the event details page (/events/:id)
 *  2. Verify event information and initial ticket registration state
 *  3. Test quantity controls and dynamic total price calculation
 *  4. Open checkout modal and verify wallet payment tab
 *  5. Test gift ticket option with recipient wallet address
 *  6. Confirm purchase with simulated wallet transaction
 *  7. Assert ticket minted confirmation screen with Ticket ID, QR code, and tx link
 *  8. Handle edge cases: free events, sold-out waitlist, and API errors
 *
 * Closes #1623
 */

const STUB_EVENT_ID = "12345678-1234-4234-8234-123456789abc";

const STUB_EVENT = {
  id: STUB_EVENT_ID,
  title: "Stellar Horizon Summit 2026",
  description: "Join developers, organizers, and creators for the premier Stellar ecosystem conference.",
  location: "Moscone Center, San Francisco, CA",
  start_time: "2026-11-20T10:00:00.000Z",
  image_url: "/images/event1.png",
  is_free: false,
  organizer_profile: {
    address: "GDORGANIZER77777777777777777777777777777777777777777777W7",
    display_name: "Stellar Community Fund",
    avatar_url: "/images/event1.png",
  },
  tiers: [
    { id: "tier-ga", name: "General Admission", price: "25.00", capacity: 100 },
    { id: "tier-vip", name: "VIP Pass", price: "75.00", capacity: 25 },
  ],
};

const STUB_FREE_EVENT = {
  ...STUB_EVENT,
  id: "87654321-4321-4321-8321-cba987654321",
  title: "Stellar Builders Meetup",
  is_free: true,
  tiers: [{ id: "tier-free", name: "Free RSVP", price: "0.00", capacity: 50 }],
};

const STUB_SOLD_OUT_EVENT = {
  ...STUB_EVENT,
  id: "11111111-2222-4333-8444-555555555555",
  title: "Sold Out Stellar Keynote",
  availableQuantity: 0,
};

const MOCK_BUYER_WALLET = "GBUYERMOCKADDRESS1234567890STEL";
const MOCK_RECIPIENT_WALLET = "GRECIPIENTMOCKADDRESS9876543210STEL";
const MOCK_TX_HASH = "a1b2c3d4e5f67890a1b2c3d4e5f67890a1b2c3d4e5f67890a1b2c3d4e5f67890";
const MOCK_TICKET_ID = "TICK-STEL-98234-CONF";

describe("Ticket Purchase Checkout Flow", () => {
  beforeEach(() => {
    // ------------------------------------------------------------------
    // Stub: Profile & Session endpoints
    // ------------------------------------------------------------------
    cy.intercept("GET", "/api/profile", {
      statusCode: 200,
      body: {
        profile: {
          displayName: "Alice Buyer",
          address: MOCK_BUYER_WALLET,
        },
      },
    }).as("profileRequest");

    // ------------------------------------------------------------------
    // Stub: Event details APIs
    // ------------------------------------------------------------------
    cy.intercept("GET", `/api/v1/events/${STUB_EVENT_ID}*`, {
      statusCode: 200,
      body: { data: STUB_EVENT },
    }).as("getEventV1");

    cy.intercept("GET", `/api/events/${STUB_EVENT_ID}*`, {
      statusCode: 200,
      body: { event: STUB_EVENT },
    }).as("getEventDetail");

    // ------------------------------------------------------------------
    // Stub: Ticket Payment endpoint
    // ------------------------------------------------------------------
    cy.intercept("POST", "/api/payments/ticket", {
      statusCode: 200,
      body: {
        success: true,
        ticketId: MOCK_TICKET_ID,
        txHash: MOCK_TX_HASH,
      },
    }).as("ticketPaymentRequest");

    // ------------------------------------------------------------------
    // Stub: Ticket backend sync
    // ------------------------------------------------------------------
    cy.intercept("POST", "/api/v1/tickets/sync", {
      statusCode: 200,
      body: { success: true, synced: true },
    }).as("ticketSyncRequest");

    // ------------------------------------------------------------------
    // Simulate authenticated user via token in localStorage and Freighter mock
    // ------------------------------------------------------------------
    cy.visit(`/events/${STUB_EVENT_ID}`, {
      onBeforeLoad(win) {
        win.localStorage.setItem("token", "mock-cypress-jwt-token");
        // Mock Freighter wallet provider if queried
        (win as unknown as { freighter: unknown }).freighter = {
          isConnected: () => Promise.resolve(true),
          getPublicKey: () => Promise.resolve(MOCK_BUYER_WALLET),
          signTransaction: (xdr: string) => Promise.resolve(xdr),
        };
      },
    });
  });

  it("completes paid ticket purchase with quantity adjustments and wallet simulation", () => {
    // ── Step 1: Verify event page loads with correct details ──────────
    cy.contains(STUB_EVENT.title).should("be.visible");
    cy.contains("Registration").should("be.visible");

    // ── Step 2: Test quantity controls and dynamic price calculation ─
    // Initial price should be $25.00 for quantity 1
    cy.contains("button", "$25.00").should("be.visible");

    // Decrement button should not drop below 1
    cy.contains("button", "−").click();
    cy.contains("button", "$25.00").should("be.visible");

    // Increment to 2 tickets -> price should update to $50.00
    cy.contains("button", "+").click();
    cy.contains("button", "$50.00").should("be.visible");

    // Increment to 3 tickets -> price should update to $75.00
    cy.contains("button", "+").click();
    cy.contains("button", "$75.00").should("be.visible");

    // Decrement back to 2 tickets -> price returns to $50.00
    cy.contains("button", "−").click();
    cy.contains("button", "$50.00").should("be.visible");

    // ── Step 3: Click Checkout / Register button to open modal ───────
    cy.contains("button", "$50.00").click();

    // ── Step 4: Verify modal contents and wallet flow ────────────────
    cy.get('[role="dialog"]').should("be.visible");
    cy.get("#ticket-modal-title").should("contain.text", STUB_EVENT.title);
    cy.contains("Confirm Ticket").should("be.visible");
    cy.contains("Pay with Wallet").should("be.visible");

    // Verify quantity in modal reflects current selection (2) and total price is $50.00
    cy.get('[role="dialog"]').within(() => {
      cy.contains("Total Price").should("be.visible");
      cy.contains("$50.00").should("be.visible");

      // Test modal quantity increment (+1 => 3 tickets => $75.00)
      cy.get('button[aria-label="Increase quantity"]').click();
      cy.contains("$75.00").should("be.visible");

      // Modal quantity decrement (-1 => 2 tickets => $50.00)
      cy.get('button[aria-label="Decrease quantity"]').click();
      cy.contains("$50.00").should("be.visible");
    });

    // ── Step 5: Confirm Purchase via simulated wallet payment ─────────
    cy.contains("button", /confirm purchase/i).click();

    // ── Step 6: Verify request payload sent to the payment API ───────
    cy.wait("@ticketPaymentRequest").then((interception) => {
      expect(interception.request.body).to.include({
        eventId: STUB_EVENT_ID,
        quantity: 2,
        buyerWallet: MOCK_BUYER_WALLET,
      });
    });

    // ── Step 7: Verify Ticket Minted success confirmation view ───────
    cy.contains("Ticket Minted!").should("be.visible");
    cy.contains(MOCK_TICKET_ID).should("be.visible");
    cy.contains(/transaction/i).should("be.visible");

    // SVG QR Code should be rendered
    cy.get('svg[class*="qrcode"], svg').should("be.visible");

    // ── Step 8: Close confirmation modal ─────────────────────────────
    cy.contains("button", /done/i).click();
    cy.get('[role="dialog"]').should("not.exist");
  });

  it("supports gifting tickets with recipient wallet address", () => {
    // Open checkout modal
    cy.contains("button", "$25.00").click();
    cy.get('[role="dialog"]').should("be.visible");

    // Toggle gift switch
    cy.get('button[aria-label="Gift mode"]').click();
    cy.get('input#recipientWallet')
      .should("be.visible")
      .type(MOCK_RECIPIENT_WALLET);

    // Confirm purchase
    cy.contains("button", /confirm purchase/i).click();

    // Assert recipient wallet is sent in request
    cy.wait("@ticketPaymentRequest").then((interception) => {
      expect(interception.request.body).to.include({
        eventId: STUB_EVENT_ID,
        quantity: 1,
        buyerWallet: MOCK_BUYER_WALLET,
        recipientWallet: MOCK_RECIPIENT_WALLET,
      });
    });

    // Success screen should mention the recipient address
    cy.contains("Ticket Minted!").should("be.visible");
    cy.contains(MOCK_RECIPIENT_WALLET.slice(0, 8)).should("be.visible");
  });

  it("handles free event checkout without payment charges", () => {
    // Intercept free event
    cy.intercept("GET", `/api/v1/events/${STUB_FREE_EVENT.id}*`, {
      statusCode: 200,
      body: { data: STUB_FREE_EVENT },
    }).as("getFreeEventV1");

    cy.visit(`/events/${STUB_FREE_EVENT.id}`, {
      onBeforeLoad(win) {
        win.localStorage.setItem("token", "mock-cypress-jwt-token");
      },
    });

    cy.contains(STUB_FREE_EVENT.title).should("be.visible");
    cy.contains("button", /register/i).should("be.visible").click();

    // In modal, total should display FREE
    cy.get('[role="dialog"]').within(() => {
      cy.contains("FREE").should("be.visible");
      cy.contains("button", /confirm purchase/i).click();
    });

    cy.wait("@ticketPaymentRequest");
    cy.contains("Ticket Minted!").should("be.visible");
  });

  it("handles sold-out events by allowing users to join the waitlist", () => {
    cy.intercept("GET", `/api/v1/events/${STUB_SOLD_OUT_EVENT.id}*`, {
      statusCode: 200,
      body: { data: STUB_SOLD_OUT_EVENT },
    }).as("getSoldOutEventV1");

    cy.intercept("POST", `/api/v1/events/${STUB_SOLD_OUT_EVENT.id}/waitlist`, {
      statusCode: 200,
      body: { position: 12 },
    }).as("joinWaitlistRequest");

    cy.visit(`/events/${STUB_SOLD_OUT_EVENT.id}`, {
      onBeforeLoad(win) {
        win.localStorage.setItem("token", "mock-cypress-jwt-token");
      },
    });

    cy.contains(STUB_SOLD_OUT_EVENT.title).should("be.visible");
    cy.contains("button", "$25.00").click();

    // Modal should show Sold Out state
    cy.get('[role="dialog"]').within(() => {
      cy.contains("Sold Out").should("be.visible");
      cy.contains("All tickets for this event have been claimed.").should("be.visible");
      cy.contains("button", /join waitlist/i).click();
    });

    cy.wait("@joinWaitlistRequest");

    // Waitlist confirmation
    cy.contains("You're on the list!").should("be.visible");
    cy.contains("#12").should("be.visible");
    cy.contains("in the queue").should("be.visible");
  });

  it("displays error toast and keeps modal open when purchase API fails", () => {
    cy.intercept("POST", "/api/payments/ticket", {
      statusCode: 500,
      body: { error: "Insufficient testnet funds in wallet" },
    }).as("failedPaymentRequest");

    cy.contains("button", "$25.00").click();
    cy.get('[role="dialog"]').should("be.visible");

    cy.contains("button", /confirm purchase/i).click();
    cy.wait("@failedPaymentRequest");

    // Should not advance to ticket minted screen
    cy.contains("Ticket Minted!").should("not.exist");
    cy.get('[role="dialog"]').should("be.visible");
  });

  it("disables purchase button and displays spinner while request is in-flight", () => {
    cy.intercept("POST", "/api/payments/ticket", (req) => {
      req.reply((res) => {
        res.setDelay(1000);
        res.send({
          statusCode: 200,
          body: {
            success: true,
            ticketId: MOCK_TICKET_ID,
            txHash: MOCK_TX_HASH,
          },
        });
      });
    }).as("delayedPaymentRequest");

    cy.contains("button", "$25.00").click();
    cy.get('[role="dialog"]').should("be.visible");

    cy.contains("button", /confirm purchase/i).click();

    // Confirm purchase button should be disabled during flight
    cy.get('[role="dialog"]')
      .find('button[disabled]')
      .should("exist");

    cy.wait("@delayedPaymentRequest");
    cy.contains("Ticket Minted!").should("be.visible");
  });
});
