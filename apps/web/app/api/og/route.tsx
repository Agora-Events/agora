import { ImageResponse } from "@vercel/og";

export const runtime = "edge";

const SITE_URL = process.env.NEXT_PUBLIC_SITE_URL || "https://agora.events";
const API_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:3001";

async function fetchEvent(eventId: string) {
  try {
    const res = await fetch(`${API_URL}/api/v1/events/${eventId}`);
    if (!res.ok) return null;
    const json = await res.json();
    // The server wraps responses in { data, ... } for many endpoints; try both.
    return json.data || json;
  } catch (e) {
    return null;
  }
}

export async function GET(req: Request) {
  const { searchParams } = new URL(req.url);
  const eventId = searchParams.get("eventId");

  let event = null;
  if (eventId) {
    event = await fetchEvent(eventId);
  }

  // Fallback defaults
  const title = event?.title ?? "Agora Event";
  const host = event?.organizerName ?? event?.organizer?.name ?? "Agora";
  const date = event?.date ?? "TBA";
  const location = event?.location ?? "Online";
  const cover = event?.imageUrl ? (event.imageUrl.startsWith("http") ? event.imageUrl : `${SITE_URL}${event.imageUrl}`) : `${SITE_URL}/og-fallback.jpg`;

  try {
    return new ImageResponse(
      (
        <div
          style={{
            width: "1200px",
            height: "630px",
            display: "flex",
            background: "#fff",
            border: "18px solid #000",
            boxSizing: "border-box",
            fontFamily: "Inter, system-ui, Arial, sans-serif",
          }}
        >
          <div style={{ width: "45%", height: "100%", overflow: "hidden", background: "#0B1221" }}>
            <img src={cover} alt="cover" style={{ width: "100%", height: "100%", objectFit: "cover" }} />
          </div>
          <div style={{ padding: 48, width: "55%", display: "flex", flexDirection: "column", justifyContent: "space-between" }}>
            <div>
              <div style={{ background: "#FFE34D", display: "inline-block", padding: "6px 12px", fontWeight: 700, borderRadius: 4 }}>Agora</div>
              <h1 style={{ fontSize: 56, marginTop: 24, marginBottom: 12, lineHeight: 1.02, color: "#000", fontWeight: 800 }}>{title}</h1>
              <p style={{ fontSize: 22, color: "#111", margin: 0 }}>{host} • {date}</p>
            </div>

            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
              <div style={{ display: "flex", gap: 12, alignItems: "center" }}>
                <div style={{ width: 44, height: 44, borderRadius: 8, background: "#000" }} />
                <div style={{ fontSize: 20, fontWeight: 700 }}>{location}</div>
              </div>
              <div style={{ fontSize: 16, color: "#333" }}>agora.events</div>
            </div>
          </div>
        </div>
      ),
      {
        width: 1200,
        height: 630,
      },
    );
  } catch (e) {
    return new Response("Failed to generate image", { status: 500 });
  }
}
