import { describe, it, expect } from "vitest";
import {
  escapeIcsText,
  formatToUtcString,
  buildIcsFile,
  buildGoogleCalendarUrl,
} from "../utils/calendar";

describe("calendar utils", () => {
  describe("escapeIcsText", () => {
    it("escapes backslashes, semicolons, commas, and newlines per RFC 5545", () => {
      const input = "Hello, world; line 1\nline 2 \\ test";
      const escaped = escapeIcsText(input);
      expect(escaped).toBe("Hello\\, world\\; line 1\\nline 2 \\\\ test");
    });

    it("returns empty string for empty input", () => {
      expect(escapeIcsText("")).toBe("");
    });
  });

  describe("formatToUtcString", () => {
    it("formats dates to YYYYMMDDTHHMMSSZ", () => {
      const date = new Date(Date.UTC(2026, 10, 17, 18, 30, 0));
      expect(formatToUtcString(date)).toBe("20261117T183000Z");
    });
  });

  describe("buildIcsFile", () => {
    it("emits a spec-compliant VCALENDAR and VEVENT", () => {
      const event = {
        id: 123,
        title: "Stellar Asado, Buenos Aires",
        description: "Builder kickoff & code\nJoin us!",
        location: "Buenos Aires, Argentina",
        startsAt: "2026-11-17T18:00:00Z",
      };

      const ics = buildIcsFile(event);
      expect(ics).toContain("BEGIN:VCALENDAR");
      expect(ics).toContain("VERSION:2.0");
      expect(ics).toContain("BEGIN:VEVENT");
      expect(ics).toContain("SUMMARY:Stellar Asado\\, Buenos Aires");
      expect(ics).toContain("DESCRIPTION:Builder kickoff & code\\nJoin us!");
      expect(ics).toContain("LOCATION:Buenos Aires\\, Argentina");
      expect(ics).toContain("DTSTART:20261117T180000Z");
      // Default end time should be start + 2 hours (20:00:00Z)
      expect(ics).toContain("DTEND:20261117T200000Z");
      expect(ics).toContain("END:VEVENT");
      expect(ics).toContain("END:VCALENDAR");
    });

    it.each([0, "event-123"])("keeps the UID when event %s is rescheduled", (id) => {
      const event = { id, title: "Conference", startsAt: "2026-11-17T10:00:00Z" };
      const original = buildIcsFile(event);
      const rescheduled = buildIcsFile({ ...event, startsAt: "2026-11-18T10:00:00Z" });

      expect(original.split("\r\n")).toContain(`UID:${id}@agora.events`);
      expect(rescheduled.split("\r\n")).toContain(`UID:${id}@agora.events`);
      expect(original).toContain("DTSTART:20261117T100000Z");
      expect(rescheduled).toContain("DTSTART:20261118T100000Z");
    });

    it("uses title and start time to distinguish events without IDs", () => {
      const event = { title: "Morning workshop", startsAt: "2026-11-17T10:00:00Z" };
      const getUid = (ics: string) => ics.split("\r\n").find((line) => line.startsWith("UID:"));
      const uid = getUid(buildIcsFile(event));

      expect(uid).toBeDefined();
      expect(getUid(buildIcsFile(event))).toBe(uid);
      expect(getUid(buildIcsFile({ ...event, title: "Builder meetup" }))).not.toBe(uid);
      expect(getUid(buildIcsFile({ ...event, startsAt: "2026-11-17T11:00:00Z" }))).not.toBe(uid);
    });

    it("uses explicit end time when provided", () => {
      const event = {
        title: "Conference",
        startsAt: "2026-11-17T10:00:00Z",
        endsAt: "2026-11-17T15:00:00Z",
      };

      const ics = buildIcsFile(event);
      expect(ics).toContain("DTSTART:20261117T100000Z");
      expect(ics).toContain("DTEND:20261117T150000Z");
    });
  });

  describe("buildGoogleCalendarUrl", () => {
    it("generates valid prefilled Google Calendar URL", () => {
      const event = {
        title: "Stellar Asado",
        location: "Buenos Aires",
        description: "Kickoff event",
        startsAt: "2026-11-17T18:00:00Z",
      };

      const url = buildGoogleCalendarUrl(event);
      expect(url).toContain("https://calendar.google.com/calendar/render?action=TEMPLATE");
      expect(url).toContain("text=Stellar+Asado");
      expect(url).toContain("location=Buenos+Aires");
      expect(url).toContain("details=Kickoff+event");
      expect(url).toContain("dates=20261117T180000Z%2F20261117T200000Z");
    });
  });
});
