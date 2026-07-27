# Reusable UI Component Library

This document is the source of truth for reusable UI pieces in the Agora web app. Check here before building a new view so we reuse existing patterns, avoid duplicate components, and keep the brand consistent across landing, discovery, profile, and event pages.

## Directory Structure

```text
apps/web/
├── DOCS/
│   └── COMPONENTS.md
└── components/
    ├── ui/
    ├── landing/
    ├── events/
    ├── layout/
    │   └── navbar/
    └── profile/
```

## UI Base

Base primitives live in `components/ui`. This folder should stay small and reusable.

### `components/ui/button.tsx`

Purpose:
- Shared button primitive for call-to-action buttons across landing pages, nav drawers, pricing cards, and forms.
- Keeps the hard-shadow, rounded-pill, and motion behavior consistent with the Agora visual style.

**Props**
| Name | Type | Required | Default | Description |
|---|---|---|---|---|
| `children` | `React.ReactNode` | Yes | - | Content to render inside the button |
| `variant` | `"primary" \| "secondary"` | No | - | Visual style variant of the button |
| `shadowColor` | `string` | No | `"rgba(0,0,0,1)"` | Shadow color for the button's drop shadow effect |
| `textColor` | `string` | No | `"text-black"` | Text color class or custom color |
| `backgroundColor` | `string` | No | `"bg-white"` | Background color class or custom color |
| `isLoading` | `boolean` | No | `false` | When true, shows a spinner and reduces label opacity |

Use it when:
- You need a primary or secondary CTA
- You want the standard hover/active motion instead of custom button styling

Avoid when:
- The element is not interactive
- The layout needs a one-off control that should first be promoted into a reusable primitive

**Usage Example:**

```tsx
import { Button } from "@/components/ui/button";

<Button
  backgroundColor="bg-[#FDDA23]"
  textColor="text-black"
  shadowColor="rgba(0,0,0,1)"
>
  Create Your Event
</Button>
```

### `components/ui/form-field.tsx`

Purpose:
- A reusable input field with an integrated label, responsive border styling, and error message display.

**Props**
| Name | Type | Required | Default | Description |
|---|---|---|---|---|
| `label` | `string` | Yes | - | The text label for the input |
| `name` | `string` | Yes | - | The name attribute for the input |
| `type` | `string` | Yes | - | The HTML input type (e.g. text, email) |
| `value` | `string` | Yes | - | The current value of the input |
| `onChange` | `(e: React.ChangeEvent<HTMLInputElement>) => void` | Yes | - | Callback fired when the value changes |
| `error` | `string` | No | - | Error message to display below the input |
| `placeholder` | `string` | No | - | Placeholder text for the input |

**Usage Example:**

```tsx
import { FormField } from "@/components/ui/form-field";

<FormField
  label="Email Address"
  name="email"
  type="email"
  value={emailValue}
  onChange={(e) => setEmailValue(e.target.value)}
  placeholder="Enter your email"
/>
```

### `components/ui/empty-state.tsx`

Purpose:
- Displayed when no content (e.g., events) matches the active filters. Accepts a
  title, description/message, optional icon or illustration, and an optional CTA
  (button or link).

**Props**
| Name | Type | Required | Default | Description |
|---|---|---|---|---|
| `title` | `string` | Yes | - | Heading text |
| `description` | `string` | No | - | Supporting message / body copy |
| `message` | `string` | No | - | Alias for `description` |
| `icon` | `React.ReactNode` | No | - | Custom icon node |
| `illustrationSrc` | `string` | No | `"/icons/404-illustration.svg"` | Illustration when no `icon` is set |
| `action` | `{ label: string; onClick: () => void }` | No | - | Click-handler CTA |
| `ctaLabel` | `string` | No | - | Link CTA label |
| `ctaLink` | `string` | No | - | Link CTA href |

**Usage Example:**

```tsx
import { EmptyState } from "@/components/ui/empty-state";

<EmptyState
  title="No events found"
  message="Try adjusting your filters or search criteria."
  ctaLabel="Create Event"
  ctaLink="/events/new"
/>
```

## Landing Components

Landing components live in `components/landing` and are intended to be composed into marketing pages.

### `HeroSection`

File: `components/landing/hero-section.tsx`

Purpose:
- Top-of-page marketing hero with the global navbar, primary CTAs, world illustration, and animated floating badges.
- Establishes the product voice and above-the-fold visual identity.

Notes:
- Uses the shared `Button` component for both hero CTAs.
- Pulls icon and artwork assets from `public/icons` and `public/images`.
- Includes the `Navbar` directly, so pages using it should not render a duplicate nav above it.

### `InfoSection`

File: `components/landing/info-section.tsx`

Purpose:
- Explains product value and the "How Agora Works" story.
- Pairs static screenshots with brand-colored CTA styling and about-copy.

Notes:
- Good reference for image-heavy marketing sections with centered pills and dark backgrounds.

### `PricingSection`

File: `components/landing/pricing-section.tsx`

Purpose:
- Shows plan comparison cards for Agora Basic and Agora Plus.
- Reuses the shared `Button` component for pricing CTAs and keeps pricing-card styling consistent.

Notes:
- Treat this as the standard pattern for side-by-side commercial plan cards.

### `FAQSection`

File: `components/landing/faq-section.tsx`

Purpose:
- Displays a branded accordion FAQ block with desktop/mobile artwork variants.
- Provides a reusable open-close interaction pattern for simple disclosure content.

Notes:
- The inner `FAQItem` is local to the file today; if another page needs the same accordion behavior, extract it into a shared component instead of re-copying it.

## Event Components

Event-related components live in `components/events`. Reuse these before building new cards, filters, or event detail widgets.

### `CategorySection`

File: `components/events/category-section.tsx`

Purpose:
- Discovery-page header plus reusable category pill buttons.
- Defines the visual language for browsing event categories.

### `PopularEventsSection`

File: `components/events/popular-events-section.tsx`

Purpose:
- Main discovery grid with search, filter drawer integration, and animated event-card list rendering.
- Best reference for list-page composition and discovery state management.

Depends on:
- `EventCard`
- `FilterSidebar`
- `mockups.ts`
- shared `Button`

### `EventCard`

File: `components/events/event-card.tsx`

Purpose:
- Ticket-style event preview card used in event discovery.
- Encodes the standard treatment for event title, date, location, price, and "View Event" affordance.

**Props**
| Name | Type | Required | Default | Description |
|---|---|---|---|---|
| `id` | `string \| number` | Yes | - | The unique identifier for the event |
| `title` | `string` | Yes | - | The title of the event |
| `date` | `string` | Yes | - | The formatted date string for the event |
| `location` | `string` | Yes | - | The location of the event |
| `price` | `string` | Yes | - | The price of the event, or "free" |
| `imageUrl` | `string` | Yes | - | URL of the event's thumbnail image |
| `loading` | `boolean` | No | `false` | When true, shows a small loading spinner overlay in the card |

**Usage Example:**

```tsx
import { EventCard } from "@/components/events/event-card";

<EventCard
  id="123"
  title="Web3 Developers Meetup"
  date="Sat, Oct 24 • 10:00 AM"
  location="Discord"
  price="Free"
  imageUrl="/images/event-thumbnail.jpg"
/>
```

Notes:
- Use this as the default event summary card before inventing a new card layout.
- Automatically swaps the location icon for Discord-style events.

### `EventCardSkeleton`

File: `components/events/event-card-skeleton.tsx`

Purpose:
- A loading placeholder that mimics the layout of the `EventCard` component to prevent layout shifts while fetching data.

**Props**
| Name | Type | Required | Default | Description |
|---|---|---|---|---|
*(This component accepts no props)*

**Usage Example:**

```tsx
import { EventCardSkeleton } from "@/components/events/event-card-skeleton";

<EventCardSkeleton />
```

### `TicketModal`

File: `components/events/TicketModal.tsx`

Purpose:
- A modal dialog that allows users to confirm ticket purchases, select ticket quantity, or gift a ticket to another wallet.

**Props**
| Name | Type | Required | Default | Description |
|---|---|---|---|---|
| `isOpen` | `boolean` | Yes | - | Controls whether the modal is visible |
| `onClose` | `() => void` | Yes | - | Callback fired when the modal should be closed |
| `event` | `{ id: number; title: string; price: string; location: string; date: string; }` | Yes | - | Details of the event being purchased |
| `initialQuantity` | `number` | Yes | - | The default number of tickets to purchase |

**Usage Example:**

```tsx
import { TicketModal } from "@/components/events/TicketModal";

<TicketModal
  isOpen={isModalOpen}
  onClose={() => setIsModalOpen(false)}
  event={{
    id: 123,
    title: "Web3 Developers Meetup",
    price: "$15.00",
    location: "San Francisco, CA",
    date: "Oct 24, 2023"
  }}
  initialQuantity={1}
/>
```

### `FilterSidebar`

File: `components/events/filter-sidebar.tsx`

Purpose:
- Slide-over filter panel for category, location, date, and price filtering.
- Owns the reusable `FilterState` shape used by the discovery view.

Notes:
- If new discovery filters are added, update `FilterState` here first and flow the new state through `PopularEventsSection`.

### `RegistrationBox`

File: `components/events/registration-box.tsx`

Purpose:
- Event detail purchase/registration widget with quantity control, price calculation, and host summary.
- Standard pattern for the event registration CTA area.

Notes:
- Supports both free and paid event flows through `isFree` and `price`.

### `EventLocationMap`

File: `components/events/event-location-map.tsx`

Purpose:
- Location map for event detail pages using `react-leaflet`.
- Handles geocoding, loading, and location-not-found states.

Notes:
- Uses a local pin asset from `public/icons/map-pin.svg`.
- Fetches coordinates from OpenStreetMap/Nominatim, so use it for real place labels rather than decorative maps.

### `CreateEventForm`

File: `components/events/create-event-form.tsx`

Purpose:
- Main form scaffold for creating new events.
- Reuses the shared button primitive and the app's rounded, offset-shadow form style.

Notes:
- Good reference for labeled field grouping and event-form state shape.

### `OrganizerComponent`

File: `components/events/organizer-component.tsx`

Purpose:
- Horizontal organizer showcase for community or ecosystem groups.
- Useful for spotlighting partner collections with branded cards and carousel-like controls.

Notes:
- This file currently defines a local `Button` helper instead of reusing `components/ui/button.tsx`.
- Prefer the shared UI button for future additions unless the visual treatment is intentionally different.

### `mockups.ts`

File: `components/events/mockups.ts`

Purpose:
- Mock event data used by discovery-related components during UI development.

Notes:
- Keep mock display data centralized here instead of duplicating sample objects across pages.

## Layout Components

Layout components live in `components/layout` and provide the shared shell around page content.

### `Navbar`

File: `components/layout/navbar.tsx`

Purpose:
- Global top navigation with responsive mobile drawer behavior.
- Switches between guest and logged-in nav variants.

Depends on:
- `navbar/guest-nav.tsx`
- `navbar/user-nav.tsx`
- `navbar/mobile-nav-link.tsx`
- shared `Button`

Use it when:
- A page needs the standard site header or mobile menu behavior

### `Footer`

File: `components/layout/footer.tsx`

Purpose:
- Shared site footer with logo, navigation links, and social links.
- Sets the standard lower-page visual treatment for marketing and content pages.

### `navbar/guest-nav.tsx`

Purpose:
- Desktop navigation for signed-out users.

### `navbar/user-nav.tsx`

Purpose:
- Desktop navigation for signed-in users.

### `navbar/nav-link.tsx`

Purpose:
- Shared desktop navigation link styling and active-state behavior.

### `navbar/mobile-nav-link.tsx`

Purpose:
- Shared mobile drawer link row with icon, label, and close-on-click behavior.

## Profile Components

Profile-specific components live in `components/profile`.

### `ProfileSidebar`

File: `components/profile/profile-sidebar.tsx`

Purpose:
- Reusable sidebar summary card for profile pages with avatar, joined date, counts, and social links.

Notes:
- Use this as the base profile-summary pattern instead of rebuilding small profile cards per page.

## Usage Examples

### Standard page shell

Use the shared layout components for full-page composition:

```tsx
import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";

export default function ExamplePage() {
  return (
    <>
      <Navbar />
      <main>{/* page sections */}</main>
      <Footer />
    </>
  );
}
```

### Landing page composition

Use the existing landing sections instead of rebuilding marketing blocks:

```tsx
import { HeroSection } from "@/components/landing/hero-section";
import { InfoSection } from "@/components/landing/info-section";
import { PricingSection } from "@/components/landing/pricing-section";
import { FAQSection } from "@/components/landing/faq-section";
import { Footer } from "@/components/layout/footer";

export default function HomePage() {
  return (
    <>
      <HeroSection />
      <InfoSection />
      <PricingSection />
      <FAQSection />
      <Footer />
    </>
  );
}
```

### Discovery page composition

Use the event discovery building blocks together:

```tsx
import { Navbar } from "@/components/layout/navbar";
import { CategorySection } from "@/components/events/category-section";
import { PopularEventsSection } from "@/components/events/popular-events-section";
import { Footer } from "@/components/layout/footer";

export default function DiscoverPage() {
  return (
    <>
      <Navbar />
      <CategorySection />
      <PopularEventsSection />
      <Footer />
    </>
  );
}
```

## Icon Policy

Agora uses local icon assets by default.

Rules:
- Reuse assets from `apps/web/public/icons/` before adding anything new.
- Prefer local SVG files over installing external icon libraries.
- Match the established visual style, stroke weight, and framing of existing icons.
- If a new icon is truly needed, add it to `public/icons` and document the usage in the component that introduces it.

Related asset folders:
- `apps/web/public/icons/`
- `apps/web/public/images/`
- `apps/web/public/logo/`

## How To Contribute A New Component

1. Check this document and the existing `components/*` folders to make sure the pattern does not already exist.
2. Decide the correct home:
   - `components/ui` for cross-app primitives
   - `components/layout` for shell/navigation/footer patterns
   - `components/landing`, `components/events`, or `components/profile` for domain-specific pieces
3. Prefer composing existing components before introducing a new abstraction.
4. Reuse the shared `Button`, shared nav building blocks, and local icon assets whenever possible.
5. Keep props focused on reuse, not page-specific hacks.
6. Add or update an entry in this file when the component is created, renamed, extracted, or deprecated.
7. If you find duplicated markup that should become a component, extract it and update the affected pages instead of leaving two versions in place.

## Before Creating Something New

Ask these questions first:
- Can this be built by composing `Button`, `Navbar`, `Footer`, `EventCard`, or an existing section?
- Does the new component belong to a domain folder instead of `ui`?
- Does it rely on icons already available in `public/icons`?
- Will another page likely reuse it within the next few changes?

If the answer is "not sure," default to reusing an existing component and open a follow-up discussion before adding a parallel pattern.
