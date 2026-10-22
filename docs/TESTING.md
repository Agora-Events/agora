# Testing Guide

This document covers all test layers in the Agora frontend — unit tests, end-to-end tests, and visual regression tests.

---

## Table of Contents

1. [Unit & Integration Tests (Vitest)](#unit--integration-tests-vitest)
2. [End-to-End Tests (Cypress)](#end-to-end-tests-cypress)
3. [Visual Regression Tests (Playwright)](#visual-regression-tests-playwright)
4. [CI Pipeline](#ci-pipeline)

---

## Unit & Integration Tests (Vitest)

Tests live in `apps/web/__tests__/`.

```bash
# Run all unit tests once
pnpm --filter web test

# Run with coverage
pnpm --filter web test:ci
```

Coverage threshold: **80 %** on all included component files.

---

## End-to-End Tests (Cypress)

Cypress specs live in `cypress/integration/` at the repo root and are configured via `cypress.config.ts`.

### Prerequisites

Start the web application first in a separate terminal (the default `baseUrl` configured in `cypress.config.ts` is `http://localhost:3000`):

```bash
pnpm --filter web dev
```

> **Note**: If your local frontend runs on a different port, override the base URL by appending `--config baseUrl=http://localhost:<PORT>` or exporting `CYPRESS_BASE_URL`.

### Running Specs

You can run Cypress using the root package scripts or directly with `pnpm exec`:

#### Option A: Package Scripts (Recommended)

```bash
# Run all Cypress specs headlessly
pnpm cy:run

# Open the interactive Cypress GUI runner
pnpm cy:open

# Run a specific spec headlessly
pnpm cy:run --spec "cypress/integration/create_event.spec.ts"
```

#### Option B: Executable CLI Commands

```bash
# Run all specs headlessly
pnpm exec cypress run

# Open interactive runner
pnpm exec cypress open

# Run in a specific browser
pnpm exec cypress run --browser chrome
```

All backend API calls are stubbed with `cy.intercept()` — no live database required.

---

## Visual Regression Tests (Playwright)

### Overview

Visual snapshot tests guard the **Neubrutalist design system** (hard borders, specific drop-shadow offsets, accent colour tokens) against accidental CSS regressions.

Four core components are tested in both **light** and **dark** mode, producing **8 base snapshots**:

| Component                | Light snapshot                    | Dark snapshot                    |
| ------------------------ | --------------------------------- | -------------------------------- |
| `Button`                 | `button-light.png`                | `button-dark.png`                |
| `EventCard`              | `event-card-light.png`            | `event-card-dark.png`            |
| `RegistrationBox`        | `registration-box-light.png`      | `registration-box-dark.png`      |
| `TicketModal` (purchase) | `ticket-modal-purchase-light.png` | `ticket-modal-purchase-dark.png` |
| `TicketModal` (waitlist) | `ticket-modal-waitlist-light.png` | `ticket-modal-waitlist-dark.png` |

Snapshots are committed to `tests/visual/__snapshots__/` and diff-visible in pull request reviews.

### Setup

Playwright and its browsers are listed as dev dependencies. Install them once:

```bash
# From the repo root
pnpm install

# Install Playwright browser binaries (Chromium only, as configured)
pnpm --filter web exec playwright install chromium
```

### Running visual tests

```bash
# From apps/web — starts the Next.js dev server automatically
npx playwright test

# Or via the pnpm script shortcut
pnpm --filter web test:visual
```

### Updating base snapshots

Run this command any time you make an **intentional** design change and want to accept the new visuals as the new baseline:

```bash
npx playwright test --update-snapshots
```

> **Important:** Always commit the updated snapshots in the same PR as the design change so reviewers can diff the images directly in GitHub.

### Viewing the HTML report

After a test run, open the HTML report to inspect screenshot diffs:

```bash
npx playwright show-report
```

### How a test fails

A test fails when:

- **Pixel diff > 0.2%** between the new screenshot and the stored base snapshot.
- A **CSS design token** (e.g. `--color-accent`, `border-black`, shadow offset) changes and affects a component in the fixture page.
- The **component is not visible** (network/auth error in the fixture page).

The `Neubrutalist design tokens – runtime assertion` test group will also fail immediately — before any screenshot comparison — if:

- The `Button` loses its hard `0px` blur drop-shadow.
- The `EventCard` loses its `rgba(0,0,0)` shadow colour.

### Fixture page

The fixture page (`/apps/web/app/__visual_fixtures__/page.tsx`) renders all four components in isolation using static props — no authentication, no API calls, no routing dependencies. It is never linked from the public app and is excluded from the sitemap.

---

## CI Pipeline

The GitHub Actions workflow (`.github/workflows/frontend.yml`) runs:

1. **Build & Lint** — `pnpm --filter web build` + ESLint
2. **Cypress E2E** — headless Cypress against the dev server
3. **Playwright Visual** — snapshot comparison (added in issue #1106)

The Playwright job uploads the HTML report and any failed screenshot diffs as CI artifacts on failure.
