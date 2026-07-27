# Component Catalog

Welcome to the Agora UI Component Catalog! This document lists all reusable UI components, their props, and usage examples.

## Navigation

- [Button](#button)
- [EmptyState](#emptystate)
- [ErrorBanner](#errorbanner)
- [FormField](#formfield)
- [Icons](#icons)
- [LazyImage](#lazyimage)
- [LoadingBar](#loadingbar)
- [Overlay](#overlay)

---

## Button

A custom Button component with distinctive shadow and hover effects.

### Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `className` | `string` | `""` | Additional CSS classes for the button |
| `shadowColor` | `string` | `"rgba(0,0,0,1)"` | Shadow color for the button's drop shadow effect |
| `textColor` | `string` | `"text-black"` | Text color class or custom color |
| `backgroundColor` | `string` | `"bg-white"` | Background color class or custom color |
| `children` | `React.ReactNode` | (required) | Content to render inside the button |
| `isLoading` | `boolean` | `false` | When true, shows a spinner and reduces label opacity |
| `disabled` | `boolean` | `false` | Disables the button |
| ...rest | `React.ButtonHTMLAttributes<HTMLButtonElement>` | — | All standard button props |

### Example

```tsx
import { Button } from "@/components/ui/button";

// Basic usage
<Button onClick={() => console.log("Clicked!")}>
  Create Event
</Button>

// With custom colors
<Button 
  backgroundColor="bg-black" 
  textColor="text-white"
  shadowColor="rgba(0,0,0,0.5)"
>
  Dark Button
</Button>

// Loading state
<Button isLoading>
  Submitting...
</Button>
```

### Storybook

- [Story: Primary](http://localhost:6006/?path=/story/ui-button--primary)
- [Story: Secondary](http://localhost:6006/?path=/story/ui-button--secondary)
- [Story: Dark](http://localhost:6006/?path=/story/ui-button--dark)
- [Story: Ghost](http://localhost:6006/?path=/story/ui-button--ghost)
- [Story: Disabled](http://localhost:6006/?path=/story/ui-button--disabled)

---

## EmptyState

<<<<<<< HEAD
Displays a friendly empty state with an illustration or icon, title, description,
and an optional action (button or link).

**File:** `apps/web/components/ui/empty-state.tsx`

### Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `title` | `string` | (required) | Heading text |
| `description` | `string` | — | Supporting message / body copy |
| `message` | `string` | — | Alias for `description` (legacy) |
| `icon` | `React.ReactNode` | — | Custom icon node |
| `illustrationSrc` | `string` | `"/icons/404-illustration.svg"` | Illustration when no `icon` is set |
| `action` | `{ label: string; onClick?: () => void; href?: string }` | — | Optional call-to-action; renders as a `<Link>` when `href` is provided, or as a `<Button>` when `onClick` is provided |
| `ctaLabel` | `string` | — | Optional link CTA label |
| `ctaLink` | `string` | — | Optional link CTA href |

### Example

```tsx
import { EmptyState } from "@/components/ui/empty-state";
import Image from "next/image";

// Icon + action button
<EmptyState
  icon={
    <Image
      src="/icons/search.svg"
      alt="Search"
      width={32}
      height={32}
    />
  }
  title="No events found"
  description="Try adjusting your filters to find what you're looking for."
  action={{ label: "Clear Search", onClick: () => setSearch("") }}
/>

// Illustration + link CTA
<EmptyState
  title="No events found"
  message="There are no events matching your current filters."
  ctaLabel="Create an Event"
  ctaLink="/events/create"
/>
```
---

## ErrorBanner

Accessible error alert with a retry button. Displays when an API request fails.

### Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `message` | `string` | (required) | Primary error message to display |
| `description` | `string` | — | Optional detailed description of the error |
| `onRetry` | `() => void` | (required) | Callback fired when the "Retry" button is clicked |

### Example

```tsx
import { ErrorBanner } from "@/components/ui/error-banner";

<ErrorBanner
  message="Failed to load events"
  description="Please check your internet connection and try again."
  onRetry={() => refetchEvents()}
/>
```

---

## FormField

A styled form input field with label and error state.

### Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `label` | `string` | (required) | Input label text |
| `name` | `string` | (required) | Input name attribute |
| `type` | `string` | (required) | Input type (e.g. "text", "email", "password") |
| `value` | `string` | (required) | Current input value |
| `onChange` | `(e: React.ChangeEvent<HTMLInputElement>) => void` | (required) | Change event handler |
| `error` | `string` | — | Error message to display below the input |
| `placeholder` | `string` | — | Input placeholder text |

### Example

```tsx
import { FormField } from "@/components/ui/form-field";
import { useState } from "react";

function MyForm() {
  const [email, setEmail] = useState("");

  return (
    <FormField
      label="Email Address"
      name="email"
      type="email"
      value={email}
      onChange={(e) => setEmail(e.target.value)}
      placeholder="you@example.com"
      error={!email.includes("@") ? "Please enter a valid email" : undefined}
    />
  );
}
```

---

## Icons

A collection of reusable icon components.

### Available Icons

- `ChevronDown`
- `ChevronUp`
- `Camera`
- `CheckCircle2`
- `Home`
- `ExternalLink`
- `X`
- `Minus`
- `Plus`
- `Ticket`
- `ArrowRight`
- `Gift`

### Props

All icons accept the same props:

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `size` | `number` | `24` | Icon size in pixels |
| `className` | `string` | — | Additional CSS classes |

### Example

```tsx
import { Home, Plus, ArrowRight } from "@/components/ui/icons";

// Basic usage
<Home />

// Custom size
<Plus size={32} />

// With custom class
<ArrowRight className="text-blue-500" />
```

---

## LazyImage

Lazy loads images using Intersection Observer for better performance.

### Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `src` | `string` | (required) | Image source URL |
| `alt` | `string` | (required) | Image alt text |
| `placeholder` | `string` | `'data:image/gif;base64,R0lGODlhAQABAAD/ACwAAAAAAQABAAACADs='` | Placeholder image to show before loading |
| ...rest | `React.ImgHTMLAttributes<HTMLImageElement>` | — | All standard img attributes |

### Example

```tsx
import { LazyImage } from "@/components/ui/LazyImage";

<LazyImage
  src="/images/event1.png"
  alt="Event photo"
  width={400}
  height={300}
  className="rounded-lg"
/>
```

---

## LoadingBar

A top navigation progress bar using NProgress. Shows during page transitions.

### Usage

```tsx
// In your root layout or page
import LoadingBar from "@/components/ui/loading-bar";

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html>
      <body>
        <LoadingBar />
        {children}
      </body>
    </html>
  );
}
```

---

## Overlay

Semi-transparent backdrop that covers the viewport. Renders behind modals/drawers and closes them on click.

### Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `isOpen` | `boolean` | (required) | Whether the overlay is visible |
| `onClose` | `() => void` | (required) | Callback fired when the overlay is clicked or Escape is pressed |
| `zIndex` | `number` | `40` | Optional z-index override |

### Example

```tsx
import { Overlay } from "@/components/ui/overlay";

function Modal({ isOpen, onClose }) {
  return (
    <>
      <Overlay isOpen={isOpen} onClose={onClose} />
      {isOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center">
          {/* Modal content */}
        </div>
      )}
    </>
  );
}
```