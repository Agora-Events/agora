https://www.figma.com/design/cpRUhrSlBVxGElm18Fa2Uh/Agora-event?node-id=813-2163&t=iDIJUBtGHnMs5zvg-4

# Agora: Upcoming Events (Empty State) - Design

This document provides a detailed technical breakdown of the "Upcoming Events (Empty State)" designed for the Agora platform, based on issue #245.

## 1. Overview
The "Empty State" of the Upcoming Events screen maintains the same global layout and theme as the "Filled State" but replaces the chronological timeline with a placeholder component in the "My Events" section.

## 2. Empty State Placeholder (My Events)
When no upcoming events are scheduled, the timeline is replaced by a centered illustration area:
*   **Container**: A large, rounded rectangle with a subtle background color (`#FFF7E6` approx) that is slightly more saturated than the page background.
*   **Illustration**: 
    *   A stylized icon representing a stack of documents or cards.
    *   A circular "0" badge attached to the top-right corner of the icon, indicating a count of zero.
*   **Text**: "Nothing Here, Yet" is positioned below the illustration.
    *   **Typography**: Medium weight, sans-serif, in a muted grey color to indicate a secondary/empty status.

## 3. Persistent Elements (Identical to Filled State)
The rest of the screen remains consistent with the global design system:

### A. Global Header
*   **Logo**: "agora" with the yellow book icon. 
*   **Nav**: "Home" (Active), "Discover Events", "Organizers", "Stellar Ecosystem".
*   **Actions**: "Create Your Event" (black pill button), Notification bell, and User profile.

### B. "For You" Section
Even when "My Events" is empty, the "For You" discovery section is fully populated to encourage exploration:
*   **Layout**: 2x3 grid of standard event cards.
*   **Content**: "Stellar developer and protocol meeting", "BitDevs Lagos", "2026 Wannabod Real Estate Outlook", etc.
*   **Actions**: "Discover" tab active, with a "View Discover Event ->" yellow button at the bottom.

### C. Footer
*   Standard dark footer with the logo, central globe graphic, navigation links, and social media icons (Instagram, X, Mail).

## 4. Visual Consistency
*   **Background**: `#FDFCEE` (Cream).
*   **Accent**: `#FFD900` (Yellow).
*   **Typography**: Clean sans-serif throughout.
*   **Shadows**: Subtle black drop-shadows on all cards and the empty state container.