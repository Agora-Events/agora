# [Design] Search for Events: UI/UX Design #538

## Summary
The platform currently lacks a dedicated search experience for events. This design implements a comprehensive search interface that aligns with Agora's visual language, covering the search input, filters, results layout, and all essential states (empty, loading, and error).

### Key Requirements
- **Search Bar & Input**: Prominent search input with "Search events, organizers, locations…" placeholder, clear/reset button, and search icon.
- **Filters & Sorting**: Horizontal scrollable category chips, date range picker (Today, Weekend, Month, Custom), location and price filters, and a sort dropdown.
- **Search Results**: Results count label, results grid (reusing GridEventCard), and a list/grid toggle.
- **Empty & Edge States**: No results illustration/message, loading skeletons, and error states for API failures.
- **Mobile Responsiveness**: Full-screen search overlay and filter bottom sheet for mobile devices.

## Figma Design Link
[Agora Event - Figma Design](https://www.figma.com/design/qnXwOxdJD4SBC6yjdwsilN/Agora-Design-%E2%80%93-Search-for-Events--UI-UX-Design?node-id=20-2&t=9O1DHo1X04wcJdra-1)
