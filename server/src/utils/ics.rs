//! ICS calendar file generation (Issue #1342)

use chrono::{DateTime, Utc};

/// Generate an .ics calendar invite for an event.
pub fn generate_ics(
    event_title: &str,
    location: &str,
    description: &str,
    dtstart: DateTime<Utc>,
    dtend: DateTime<Utc>,
    event_link: &str,
) -> String {
    let dtstamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let start = dtstart.format("%Y%m%dT%H%M%SZ").to_string();
    let end = dtend.format("%Y%m%dT%H%M%SZ").to_string();
    let uid = format!("{}@agora.events", uuid::Uuid::new_v4());
    let desc = format!("{description}\\n{event_link}");

    // Escape commas/semicolons per RFC5545 for safety (minimal)
    let summary = escape_ics_text(event_title);
    let loc = escape_ics_text(location);
    let desc_escaped = escape_ics_text(&desc);

    format!(
        "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Agora//Event//EN\r\n\
CALSCALE:GREGORIAN\r\n\
METHOD:PUBLISH\r\n\
BEGIN:VEVENT\r\n\
UID:{uid}\r\n\
DTSTAMP:{dtstamp}\r\n\
DTSTART:{start}\r\n\
DTEND:{end}\r\n\
SUMMARY:{summary}\r\n\
LOCATION:{loc}\r\n\
DESCRIPTION:{desc_escaped}\r\n\
STATUS:CONFIRMED\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n"
    )
}

fn escape_ics_text(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_ics_contains_required_fields() {
        let start = Utc.with_ymd_and_hms(2026, 6, 1, 10, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 6, 1, 12, 0, 0).unwrap();
        let ics = generate_ics("My Concert", "Lagos", "Great show", start, end, "https://agora.events/e/1");
        assert!(ics.contains("DTSTART:20260601T100000Z"));
        assert!(ics.contains("DTEND:20260601T120000Z"));
        assert!(ics.contains("SUMMARY:My Concert"));
        assert!(ics.contains("LOCATION:Lagos"));
        assert!(ics.contains("DESCRIPTION:"));
        assert!(ics.contains("BEGIN:VCALENDAR"));
        assert!(ics.contains("END:VCALENDAR"));
        assert!(ics.contains("BEGIN:VEVENT"));
    }

    #[test]
    fn test_ics_escaping() {
        assert_eq!(escape_ics_text("a,b;c"), "a\\,b\\;c");
    }
}
