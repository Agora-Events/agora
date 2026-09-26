https://www.figma.com/design/cpRUhrSlBVxGElm18Fa2Uh/Agora-event?node-id=813-2162&t=iDIJUBtGHnMs5zvg-4

# Agora: Upcoming Events (Filled State) - Design Description

This document provides a detailed technical breakdown of the "Upcoming Events (Filled State)" designed for the Agora platform, based on issue #244.

## 1. Overall Theme & Palette
*   **Background**: Soft cream/off-white (approx `#FDFCEE`).
*   **Primary Accent**: Vibrant Yellow (`#FFD900`) used for active states, logos, and primary action buttons.
*   **Secondary Accent**: Soft Orange/Peach (used for tab backgrounds).
*   **Text/Contrast**: Deep Black (`#000000`) for headers and primary text.
*   **Success State**: Fluorescent Green (`#EBFF00` or similar) for "Going" badges.
*   **Shadows**: "Pop-art" style hard shadows or soft shadows with high offset (Black, bottom-right).

## 2. Layout & Structure
The page is organized into a vertical flow with a clean, centered container.

### A. Navigation Bar
*   **Logo**: Yellow book/card icon followed by "agora" in lowercase sans-serif.
*   **Menu Items**: "Home" (Active, Yellow), "Discover Events", "Organizers", "Stellar Ecosystem".
*   **Actions**: 
    *   "Create Your Event" button: Rounded, black border, includes a northeast arrow icon.
    *   Notification Bell: Outline style with a small red dot.
    *   User Profile: Circular avatar.

### B. "My Events" Section
*   **Header**: Large, bold "My Events".
*   **Tabs**: Pill-shaped switcher: "Upcoming" (active, rounded-peach background), "Hosting", "Past".
*   **Floating Chat**: A comment icon with a red notification bubble (1).
*   **Timeline View**:
    *   Vertical dotted line connecting specific dates (e.g., "6 Mar Friday").
    *   **Large Event Cards**:
        *   **Left Element**: A stylized "Ticket" stub (Purple for one, Black for another) with a barcode-like pattern at the bottom.
        *   **Middle Element**: Graphic/Poster area.
        *   **Right Element**: 
            *   Time (e.g., "1:00 PM WAT").
            *   Event Title (Bold, e.g., "Stellar developer and protocol meeting").
            *   Location Icon + Text.
            *   "Going" Badge (rounded, fluorescent green).
            *   Attendee Avatars: Stacked circular avatars with a count (e.g., +4).
            *   "View Event ->" link at the bottom right.

### C. "For You" (Discovery) Section
*   **Header**: Large, bold "For You".
*   **Tabs**: "Discover" (Active), "Following".
*   **Layout**: 2x3 Grid of smaller event cards.
*   **Small Event Cards**:
    *   Consistent background: Off-white/White with a dark bottom-right shadow.
    *   Ticket-stub icon on the left (various colors like Purple, Teal, Orange, Blue, Green).
    *   Content: Date/Time (small), Title (Bold), Location, Price (e.g., "Free" or "$8.90").
    *   "View Event ->" link in the bottom right corner of each card.
*   **Secondary Action**: Yellow "View All Events ->" and "View Discover Events ->" buttons at the bottom of respective sections.

## 3. Typography
*   **Font Family**: Clean, modern sans-serif (similar to Inter, Outfit, or Poppins).
*   **Headers**: Bold, high contrast (Black).
*   **Body/Labels**: Medium weight, slightly smaller font size for metadata (Time, Location).

## 4. Components & Interactive Elements
*   **Buttons**: Highly rounded (capsule shape) or rounded squares.
*   **Cards**: All cards use a "Ticket" metaphor, either through full card shape or left-side iconography.
*   **Hard Decor**: Subtle background illustrations (like the "agora" ticket graphic on the right side of the "For You" section).

## 5. Footer
*   **Background**: Dark grey/Black section.
*   **Graphic**: Central semi-circle/globe illustration with event thumbnails.
*   **Columns**:
    *   Column 1: Logo + Copyright.
    *   Column 2: "Discover Event", "About Us", "Pricing", "FAQs".
    *   Column 3 (Socials): Instagram, X, Mail, Stellar Ecosystem (all with corresponding icons).