/**
 * E2E test: "Create Event" flow
 *
 * Journey:
 *  1. Visit the landing page
 *  2. Log in via the email auth endpoint (stubbed)
 *  3. Navigate to /create-event via the navbar CTA
 *  4. Fill in all required fields
 *  5. Submit the form (stubbed POST /api/events)
 *  6. Assert redirect to the new event detail page
 *  7. Assert the event title is visible on that page'
 */

const TEST_EVENT = {
  title: "Cypress Test Event",
  startDate: "2026-09-15",
  startTime: "14:00",
  location: "Virtual – Zoom",
  price: "0",
  description: "Automated E2E test event created by Cypress",
};

const STUB_EVENT_ID = "cypress-event-123";
const STUB_EVENT_RESPONSE = {
  event: {
    id: STUB_EVENT_ID,
    title: TEST_EVENT.title,
    startsAt: `${TEST_EVENT.startDate}T${TEST_EVENT.startTime}:00.000Z`,
    location: TEST_EVENT.location,
    category: "Tech",
    organizerName: "Test User",
    organizerWallet: "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
    description: TEST_EVENT.description,
    ticketPrice: 0,
    totalTickets: 100,
    followersOnly: false,
    hostEmail: "test@example.com",
  },
};

describe("Create Event flow", () => {
  beforeEach(() => {
    // ------------------------------------------------------------------
    // Stub: POST /api/auth/email — returns success and sets a session
    // ------------------------------------------------------------------
    cy.intercept("POST", "/api/auth/email", {
      statusCode: 200,
      body: { success: true },
    }).as("loginRequest");

    // ------------------------------------------------------------------
    // Stub: GET /api/profile — returns organizer data used by the form
    // ------------------------------------------------------------------
    cy.intercept("GET", "/api/profile", {
      statusCode: 200,
      body: {
        profile: {
          displayName: "Test User",
          address: "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
        },
      },
    }).as("profileRequest");

    // ------------------------------------------------------------------
    // Stub: POST /api/events — simulates successful event creation
    // ------------------------------------------------------------------
    cy.intercept("POST", "/api/events", {
      statusCode: 201,
      body: STUB_EVENT_RESPONSE,
    }).as("createEventRequest");

    // ------------------------------------------------------------------
    // Stub: GET /api/events — event list used on home/discover pages
    // ------------------------------------------------------------------
    cy.intercept("GET", "/api/events*", {
      statusCode: 200,
      body: {
        items: [STUB_EVENT_RESPONSE.event],
        tab: "upcoming",
        type: "all",
      },
    }).as("eventsListRequest");

    // ------------------------------------------------------------------
    // Stub: event detail page API if applicable
    // ------------------------------------------------------------------
    cy.intercept("GET", `/api/events/${STUB_EVENT_ID}`, {
      statusCode: 200,
      body: { event: STUB_EVENT_RESPONSE.event },
    }).as("eventDetailRequest");
  });

  it("logs in, opens Create Event page, fills the form, submits, and lands on the event page", () => {
    // ── Step 1: Visit the auth page and log in ───────────────────────
    cy.visit("/auth");
    cy.get('input[name="email"], input[type="email"]')
      .should("be.visible")
      .type("test@example.com");

    cy.contains("button", /continue with email/i).click();
    cy.wait("@loginRequest");

    // After login the app pushes to /home
    cy.url().should("include", "/home");

    // ── Step 2: Navigate to Create Event via the navbar CTA ──────────
    // The "Create Your Event" button renders as a link to /create-event
    cy.contains("a", /create your event/i)
      .first()
      .click();

    cy.url().should("include", "/create-event");

    // ── Step 3: Fill in required fields ─────────────────────────────
    // Event Title
    cy.get('input[name="title"]')
      .should("be.visible")
      .type(TEST_EVENT.title);

    // Start Date
    cy.get('input[name="startDate"]')
      .should("exist")
      .type(TEST_EVENT.startDate);

    // Start Time
    cy.get('input[name="startTime"]')
      .should("exist")
      .type(TEST_EVENT.startTime);

    // Location
    cy.get('input[name="location"]')
      .should("be.visible")
      .type(TEST_EVENT.location);

    // Ticket Price (required; 0 = free)
    cy.get('input[name="price"]')
      .should("be.visible")
      .type(TEST_EVENT.price);

    // Optional: description
    cy.get('textarea[name="description"]')
      .should("exist")
      .type(TEST_EVENT.description);

    // ── Step 4: Submit the form ──────────────────────────────────────
    cy.contains("button", /create event/i)
      .should("not.be.disabled")
      .click();

    cy.wait("@createEventRequest").then((interception) => {
      // Verify the request body contains the expected title
      expect(interception.request.body).to.have.property(
        "title",
        TEST_EVENT.title,
      );
      expect(interception.request.body).to.have.property(
        "location",
        TEST_EVENT.location,
      );
    });

    // ── Step 5: Verify redirect to the new event detail page ────────
    cy.url().should("include", `/events/${STUB_EVENT_ID}`);

    // ── Step 6: Verify the event title is visible on the detail page ─
    cy.contains(TEST_EVENT.title).should("be.visible");
  });

  it("shows validation errors when required fields are empty", () => {
    // Navigate directly to create-event (no login needed for UI validation)
    cy.visit("/create-event");

    // Click submit without filling anything
    cy.contains("button", /create event/i).click();

    // The form should NOT have called the API
    cy.get("@createEventRequest.all").should("have.length", 0);

    // At least one validation error should be visible
    cy.contains(/required/i).should("be.visible");
  });

  it("disables the submit button while the request is in-flight", () => {
    // Delay the response to observe the loading state
    cy.intercept("POST", "/api/events", (req) => {
      req.reply((res) => {
        res.setDelay(1000);
        res.send({ statusCode: 201, body: STUB_EVENT_RESPONSE });
      });
    }).as("slowCreateEventRequest");

    cy.visit("/create-event");

    cy.get('input[name="title"]').type(TEST_EVENT.title);
    cy.get('input[name="startDate"]').type(TEST_EVENT.startDate);
    cy.get('input[name="startTime"]').type(TEST_EVENT.startTime);
    cy.get('input[name="location"]').type(TEST_EVENT.location);
    cy.get('input[name="price"]').type(TEST_EVENT.price);

    cy.contains("button", /create event/i).click();

    // Button should show "Creating..." and be disabled during the request
    cy.contains("button", /creating/i).should("be.disabled");

    cy.wait("@slowCreateEventRequest");
  });
});
