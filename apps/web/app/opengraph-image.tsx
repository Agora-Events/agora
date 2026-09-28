import { ImageResponse } from "next/og";

export const runtime = "edge";

export const size = { width: 1200, height: 630 };
export const contentType = "image/png";
export const alt = "Agora — Plan Events. Bring People Together. Grow Communities.";

export default async function OpenGraphImage() {
  return new ImageResponse(
    (
      <div
        style={{
          width: "100%",
          height: "100%",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          background: "#FFFBE9",
          border: "18px solid #060606",
          boxSizing: "border-box",
          fontFamily: "Inter, system-ui, Arial, sans-serif",
          padding: "64px 80px",
          position: "relative",
        }}
      >
        {/* Brand badge */}
        <div
          style={{
            background: "#FDDA23",
            display: "inline-block",
            padding: "8px 20px",
            borderRadius: 8,
            fontSize: 28,
            fontWeight: 800,
            letterSpacing: "0.04em",
            color: "#060606",
            marginBottom: 32,
          }}
        >
          AGORA
        </div>

        {/* Tagline */}
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            gap: 6,
          }}
        >
          <span
            style={{
              fontSize: 52,
              fontWeight: 700,
              lineHeight: 1.15,
              color: "#060606",
              textAlign: "center",
            }}
          >
            Plan Events. Bring People
          </span>
          <span
            style={{
              fontSize: 52,
              fontWeight: 700,
              lineHeight: 1.15,
              color: "#CAAE1C",
              textAlign: "center",
            }}
          >
            Together. Grow Communities.
          </span>
        </div>

        {/* Domain */}
        <div
          style={{
            position: "absolute",
            bottom: 40,
            fontSize: 18,
            color: "#747475",
            letterSpacing: "0.02em",
          }}
        >
          agora.events
        </div>
      </div>
    ),
    { width: 1200, height: 630 },
  );
}