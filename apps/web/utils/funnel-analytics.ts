export type FunnelEventType = "page_view" | "checkout_started";

export function trackFunnelEvent(type: FunnelEventType, eventId: string | number) {
  return fetch("/api/analytics", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ type, eventId: String(eventId) }),
    keepalive: true,
  }).catch(() => undefined);
}
