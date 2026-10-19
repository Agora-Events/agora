# Real-Time Ticket Availability Feature

## Overview

The Real-Time Ticket Availability feature displays live ticket availability updates on the event detail page. Users see:

- Current number of available tickets
- Visual progress bar showing sold percentage
- Status messages (low stock warnings, sold out)
- Optional detailed breakdown (total, minted, available, sold percentage)

This feature polls the backend every 5 seconds by default and can optionally use Server-Sent Events (SSE) for instant updates.

## Architecture

### Component Hierarchy

```
EventDetailPage (Server Component)
  └─ RegistrationBox (Client Component)
      └─ TicketAvailabilityDisplay (Client Component)
          └─ useTicketAvailability Hook (Data Fetching)
              └─ GET /api/events/[id]/availability (API Endpoint)
                  └─ Prisma Event Query (Database)
```

### Data Flow

```
┌─────────────────────────────────────────────────────────┐
│ Event Detail Page                                        │
│ /events/[id]                                             │
└────────────────────────┬────────────────────────────────┘
                         │ (passes eventId)
                         ↓
┌─────────────────────────────────────────────────────────┐
│ RegistrationBox (Client Component)                       │
│ - Handles ticket quantity selection                      │
│ - Displays registration button                           │
└────────────────────────┬────────────────────────────────┘
                         │ (passes eventId)
                         ↓
┌─────────────────────────────────────────────────────────┐
│ TicketAvailabilityDisplay (Client Component)             │
│ - Shows availability status & progress bar               │
│ - Manages polling via hook                               │
└────────────────────────┬────────────────────────────────┘
                         │ (calls hook with eventId)
                         ↓
┌─────────────────────────────────────────────────────────┐
│ useTicketAvailability Hook                               │
│ - Fetches from /api/events/[id]/availability             │
│ - Handles polling (5s intervals)                         │
│ - Optional SSE support for real-time updates             │
│ - Returns: data, isLoading, error, refresh()             │
└────────────────────────┬────────────────────────────────┘
                         │ (HTTP requests every 5s)
                         ↓
┌─────────────────────────────────────────────────────────┐
│ API Endpoint                                             │
│ GET /api/events/[id]/availability                        │
│ - Queries Event model from Prisma                        │
│ - Calculates: available, sold%, isSoldOut                │
│ - Returns: TicketAvailabilityData (JSON)                 │
│ - Cache: 2s with SWR re-validate window                  │
└────────────────────────┬────────────────────────────────┘
                         │ (reads totalTickets, mintedTickets)
                         ↓
┌─────────────────────────────────────────────────────────┐
│ PostgreSQL Database                                      │
│ Event Model: id, totalTickets, mintedTickets             │
└─────────────────────────────────────────────────────────┘
```

## File Structure

```
apps/web/
├── hooks/
│   └── useTicketAvailability.ts          # Hook for fetching availability data
├── app/api/events/[id]/availability/
│   └── route.ts                          # API endpoint for availability
├── components/events/
│   ├── ticket-availability-display.tsx   # Display component
│   └── registration-box.tsx              # Updated with availability display
├── __tests__/
│   └── ticket-availability.test.tsx      # Tests for the feature
└── DOCS/
    └── REAL_TIME_TICKET_AVAILABILITY.md  # This file
```

## Components

### 1. useTicketAvailability Hook

**Location**: `hooks/useTicketAvailability.ts`

**Purpose**: Manages real-time ticket availability data fetching

**Features**:

- Automatic polling (configurable interval, default 5 seconds)
- SWR caching and deduplication
- Optional Server-Sent Events (SSE) for instant updates
- Fallback to polling if SSE fails
- Manual refresh capability
- Utility functions for status formatting

**Usage**:

```typescript
'use client';

import { useTicketAvailability, getAvailabilityStatus } from '@/hooks/useTicketAvailability';

export function MyComponent({ eventId }: { eventId: string }) {
  const { data, isLoading, error, refresh } = useTicketAvailability(eventId, {
    pollInterval: 5000,  // 5 seconds (default)
    pollOnBlur: true,    // Continue polling when tab not focused
  });

  if (isLoading) return <p>Loading...</p>;
  if (error) return <p>Error loading tickets</p>;
  if (!data) return null;

  const status = getAvailabilityStatus(data);

  return (
    <div>
      <p>{status}</p>
      <p>{data.availableTickets} / {data.totalTickets} available</p>
      <button onClick={() => refresh()}>Manual Refresh</button>
    </div>
  );
}
```

**Return Object**:

```typescript
{
  data?: TicketAvailabilityData;     // Ticket availability info
  isLoading: boolean;                // Fetching in progress
  error: Error | null;               // Fetch error
  refresh: () => void;               // Manual refresh function
  isUsingSSE: boolean;               // Whether using SSE
}
```

**TicketAvailabilityData Structure**:

```typescript
{
  totalTickets: number;        // Total tickets for event
  mintedTickets: number;       // Already purchased tickets
  availableTickets: number;    // Remaining available tickets
  isSoldOut: boolean;          // Whether event is sold out
  percentageSold: number;      // Percentage of tickets sold (0-100)
  isUsingSSE?: boolean;        // Whether using real-time SSE updates
}
```

### 2. TicketAvailabilityDisplay Component

**Location**: `components/events/ticket-availability-display.tsx`

**Purpose**: Display ticket availability with status and progress visualization

**Features**:

- Status message with contextual colors
- Visual progress bar
- Warning states (low stock, almost sold out, sold out)
- Optional detailed breakdown
- Live indicator for SSE connections
- Error handling

**Usage**:

```typescript
'use client';

import { TicketAvailabilityDisplay } from '@/components/events/ticket-availability-display';

export function EventCard() {
  return (
    <div>
      {/* Basic display */}
      <TicketAvailabilityDisplay eventId="evt_123" />

      {/* With details and custom polling */}
      <TicketAvailabilityDisplay
        eventId="evt_123"
        showDetails={true}
        pollInterval={3000}  // 3 seconds
        className="p-4 bg-blue-50 rounded-lg"
      />
    </div>
  );
}
```

**Props**:

```typescript
{
  eventId: string;              // Required: Event ID to monitor
  className?: string;           // Optional: CSS classes
  showDetails?: boolean;        // Optional: Show detailed breakdown (default: false)
  pollInterval?: number;        // Optional: Polling interval in ms (default: 5000)
}
```

**Visual States**:

| State           | Color  | Message                | Show Bar |
| --------------- | ------ | ---------------------- | -------- |
| Available       | Green  | "70 tickets available" | Yes      |
| Almost Sold Out | Yellow | "Almost Sold Out"      | Yes      |
| Low Stock (≤5)  | Orange | "Only 4 left!"         | Yes      |
| Sold Out        | Red    | "Sold Out"             | No       |

### 3. API Endpoint

**Location**: `app/api/events/[id]/availability/route.ts`

**Endpoint**: `GET /api/events/[id]/availability`

**Purpose**: Returns real-time ticket availability for an event

**Request**:

```bash
GET /api/events/evt_123/availability
```

**Response (200 OK)**:

```json
{
  "totalTickets": 100,
  "mintedTickets": 35,
  "availableTickets": 65,
  "isSoldOut": false,
  "percentageSold": 35
}
```

**Error Responses**:

```javascript
// Invalid event ID
GET /api/events/invalid/availability
// Response (400 Bad Request)
{ "error": "Invalid eventId" }

// Event not found
GET /api/events/nonexistent/availability
// Response (404 Not Found)
{ "error": "Event not found" }
```

**Cache Headers**:

```
Cache-Control: public, max-age=2, stale-while-revalidate=10
```

This allows:

- 2-second hard cache (browser caches response)
- 10-second stale-while-revalidate window (stale data served while fresh fetch happens)
- Reduces server load while maintaining relative freshness

## Integration with Registration Box

The `RegistrationBox` component now displays real-time ticket availability:

```typescript
// components/events/registration-box.tsx
'use client';

import { TicketAvailabilityDisplay } from './ticket-availability-display';

export function RegistrationBox({ event, host }) {
  const [quantity, setQuantity] = useState(1);
  const [isModalOpen, setIsModalOpen] = useState(false);

  return (
    <div className="bg-surface rounded-3xl p-6 sm:p-8">
      {/* Quantity selector */}
      <div className="flex justify-between items-center">
        {/* +/- buttons */}
      </div>

      {/* NEW: Real-time ticket availability display */}
      <div className="bg-gray-50 rounded-lg p-4 border border-gray-200">
        <TicketAvailabilityDisplay
          eventId={event.id.toString()}
          showDetails={false}
          pollInterval={5000}
        />
      </div>

      {/* Registration button and host info */}
      <div className="flex items-center justify-between">
        {/* Button and host */}
      </div>
    </div>
  );
}
```

## Configuration Options

### Polling Interval

Control how frequently the client checks for updates:

```typescript
// Check every 3 seconds (aggressive updates)
<TicketAvailabilityDisplay
  eventId="evt_123"
  pollInterval={3000}
/>

// Check every 10 seconds (less aggressive)
<TicketAvailabilityDisplay
  eventId="evt_123"
  pollInterval={10000}
/>

// Disable polling, only fetch on mount
<TicketAvailabilityDisplay
  eventId="evt_123"
  pollInterval={0}
/>
```

### Show Detailed Breakdown

Display detailed ticket statistics:

```typescript
// Basic view (default)
<TicketAvailabilityDisplay eventId="evt_123" />
// Shows: "70 tickets available"

// Detailed view
<TicketAvailabilityDisplay eventId="evt_123" showDetails={true} />
// Shows:
// Available: 70
// Minted: 30
// Total: 100
// Sold: 30%
```

## Performance Considerations

### 1. Cache Strategy

**Polling Caching**:

- SWR deduplicates requests within 5-minute window
- Browser cache respects 2-second max-age
- Stale-while-revalidate extends cache window

**Result**: Reduced server requests, minimal stale data

### 2. Request Frequency

**Default**: 5-second polling interval

**Impact**:

- 100 users on event detail page
- ~100 requests every 5 seconds = ~20 req/s
- Minimal database load (simple indexed query)

**Optimization**: Could reduce to 10s for low-traffic events

### 3. Server-Sent Events (Optional)

For real-time updates without polling:

```typescript
// Future enhancement - requires backend SSE support
<TicketAvailabilityDisplay
  eventId="evt_123"
  sseUrl="/api/events/events-stream"
/>
```

This would replace polling with persistent connection (more efficient for high-traffic events).

## Testing

### Unit Tests

**Location**: `__tests__/ticket-availability.test.tsx`

**Coverage**:

- `getAvailabilityStatus()` utility function
- `TicketAvailabilityDisplay` component rendering
- Loading states
- Error states
- Sold out display
- Low stock warnings
- Detailed breakdown visibility
- Progress bar calculations
- SSE indicator

**Running Tests**:

```bash
# Run specific test file
pnpm test ticket-availability.test.tsx

# Run with coverage
pnpm test:ci

# Watch mode
pnpm test -- --watch
```

### Manual Testing

**Steps**:

1. Navigate to any event detail page
2. Verify ticket availability displays correctly
3. Check availability message matches ticket count
4. Observe progress bar updates
5. Wait 5+ seconds and verify data refreshes
6. Try edge cases:
   - Sold out event
   - Event with <5 tickets
   - Event with 75%+ sold

**Test Events** (add to mockups if needed):

- Low stock: 2/100 tickets available
- Almost sold: 20/100 tickets available
- Sold out: 0/100 tickets available

## Common Issues & Solutions

### Issue: Availability not updating

**Possible Causes**:

1. Event ID not matching database
2. Polling interval too long
3. Browser cache preventing refresh

**Solutions**:

```typescript
// Ensure event ID is string
eventId={event.id.toString()}

// Use shorter interval for testing
pollInterval={1000}

// Manual refresh available
const { refresh } = useTicketAvailability(eventId);
refresh(); // Call to force update
```

### Issue: Performance degradation

**Possible Causes**:

1. Too many components polling simultaneously
2. Polling interval too aggressive
3. Missing API response caching

**Solutions**:

```typescript
// Increase polling interval
pollInterval={10000}

// Reuse same hook instance (SWR dedupes)
// Don't create new components unnecessarily

// API cache is automatic (handled in route.ts)
```

### Issue: SSE connection not working

**Possible Causes**:

1. Backend doesn't support SSE
2. Proxy/firewall blocking WebSocket-like connections

**Solutions**:

```typescript
// SSE is optional, falls back to polling automatically
// No action needed - hook handles fallback

// To check if using SSE:
const { isUsingSSE } = useTicketAvailability(eventId);
```

## Future Enhancements

### 1. Server-Sent Events (SSE)

Replace polling with persistent connection for real-time updates:

```typescript
// Requires backend implementation
sseUrl = "/api/events/stream";
```

### 2. Optimistic Updates

Update availability immediately after ticket purchase:

```typescript
// After successful mintTicket() call
const { refresh } = useTicketAvailability(eventId);
refresh(); // Re-fetch latest data
```

### 3. Notifications

Alert user when specific thresholds reached:

```typescript
if (data.availableTickets === 0) {
  notify("Event sold out!");
}
```

### 4. Global Ticket Counter

Show total tickets sold across all events:

```typescript
const { data: stats } = useTotalTicketStats();
return <p>Total sold today: {stats.total}</p>;
```

## API Modifications Required

The following endpoint is new and must be added to the backend:

**Endpoint**: `GET /api/events/[id]/availability`

**Handler**: Must query Prisma Event model and return:

```json
{
  "totalTickets": number,
  "mintedTickets": number,
  "availableTickets": number,
  "isSoldOut": boolean,
  "percentageSold": number
}
```

**Database Query**:

```typescript
const event = await prisma.event.findUnique({
  where: { id: eventId },
  select: {
    totalTickets: true,
    mintedTickets: true,
  },
});
```

This is a read-only query, no database changes needed.

## Deployment Notes

### Environment Variables

No new environment variables required.

### Database Schema

No schema changes needed. Uses existing `Event` model.

### Breaking Changes

None. Feature is additive and backward compatible.

### Rollback Procedure

If issues occur:

1. Remove `<TicketAvailabilityDisplay />` from registration-box.tsx
2. Delete the availability API endpoint
3. Remove hook and test files
4. Redeploy

## Monitoring & Analytics

### Metrics to Track

- API response times for availability endpoint
- Cache hit/miss rates
- Error rates (404, 500, etc.)
- Polling frequency impact on database load

### Error Tracking

- Log failed availability fetches
- Alert on high error rates
- Monitor API endpoint performance

## Related Documentation

- [TESTING_STRATEGY.md](./TESTING_STRATEGY.md) - Testing framework and patterns
- [FRONTEND_ARCHITECTURE.md](./FRONTEND_ARCHITECTURE.md) - Overall frontend design
- API Event Routes: `app/api/events/[id]/route.ts`
- Ticket Purchase: `app/api/payments/ticket/route.ts`
