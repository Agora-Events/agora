https://www.figma.com/design/cpRUhrSlBVxGElm18Fa2Uh/Agora-event?node-id=825-3084&t=iDIJUBtGHnMs5zvg-4

# Agora: Organizers Feature - Design Specifications (Empty State)

This document provides a detailed technical breakdown of the "Organizers" feature for the Agora platform, focusing on the empty state based on issue #249.

## 1. Organizers Main Screen (Empty State)
The empty state maintains the layout of the organizers page but replaces active subscription data with a placeholder.

### A. "My Organizer profile" Card (Persistent)
*   **Visuals**:
    *   **Header**: Vibrant abstract gradient background.
    *   **Avatar**: Circular profile image.
    *   **Info**: User name and handle in bold black text.
*   **Action**: Yellow capsule-shaped "Edit" button.

### B. "Subscribed Organizers" Section (Placeholder)
When the user has no active organization subscriptions:
*   **Container**: A large, centered, rounded-rectangle with a soft background.
*   **Illustration**:
    *   Stylized document/card icon in the center.
    *   Circular "0" badge indicating zero subscriptions.
*   **Text**: "Nothing Here, Yet" centered below the icon.
    *   **Typography**: Muted grey, medium weight sans-serif.

## 2. Global Styles & UI Motifs
*   **Background**: Soft cream-yellow (`#FDFCEE`).
*   **Typography**: Clean sans-serif.
*   **Color Palette**: Yellow accent (`#FFD900`), Deep Black (`#000000`).
*   **Shadows**: Subtle but consistent dark drop-shadows on cards and placeholders.
*   **Footer**: Standard dark Agora footer remains consistent.
