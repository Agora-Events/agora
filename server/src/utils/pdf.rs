//! Minimal PDF generation for tickets (Issue #1341)

/// Generate a minimal A4 PDF containing ticket details.
/// Returns raw PDF bytes with `application/pdf` content type.
pub fn generate_ticket_pdf(
    event_title: &str,
    event_date: &str,
    venue: &str,
    tier: &str,
    attendee_wallet_masked: &str,
    qr_payload: &str,
) -> Vec<u8> {
    // Build a tiny PDF with one page. We embed text directly in a content stream.
    // This avoids pulling a heavy Rust PDF crate (printpdf) for the MVP and keeps
    // the response valid for `application/pdf` consumers and tests.
    let content = build_content_stream(
        event_title,
        event_date,
        venue,
        tier,
        attendee_wallet_masked,
        qr_payload,
    );

    build_pdf(content)
}

fn escape_pdf_text(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

fn build_content_stream(
    title: &str,
    date: &str,
    venue: &str,
    tier: &str,
    masked: &str,
    qr: &str,
) -> String {
    let agora = "Agora - Ticket";
    // Use Helvetica (built-in) at various positions
    let mut y = 750;
    let push_line = |lines: &mut Vec<String>, y: i32, text: &str, size: u32| {
        let escaped = escape_pdf_text(text);
        lines.push(format!("BT /F1 {size} Tf 50 {y} Td ({escaped}) Tj ET"));
    };

    let mut content_lines = Vec::new();
    push_line(&mut content_lines, y, agora, 18);
    y -= 30;
    push_line(&mut content_lines, y, &format!("Event: {title}"), 12);
    y -= 20;
    push_line(&mut content_lines, y, &format!("Date: {date}"), 11);
    y -= 20;
    push_line(&mut content_lines, y, &format!("Venue: {venue}"), 11);
    y -= 20;
    push_line(&mut content_lines, y, &format!("Tier: {tier}"), 11);
    y -= 20;
    push_line(&mut content_lines, y, &format!("Attendee: {masked}"), 10);
    y -= 20;
    push_line(&mut content_lines, y, &format!("QR: {qr}"), 9);
    y -= 20;
    push_line(&mut content_lines, y, "Branded with Agora colors #6C5CE7", 8);

    content_lines.join("\n")
}

fn build_pdf(content_stream: String) -> Vec<u8> {
    // Very small PDF 1.4 with single page, Helvetica font
    let stream_bytes = content_stream.as_bytes();
    let stream_len = stream_bytes.len();

    let mut pdf = Vec::new();
    let mut offsets: Vec<usize> = Vec::new();

    pdf.extend_from_slice(b"%PDF-1.4\n");

    // 1 0 obj - Catalog
    offsets.push(pdf.len());
    pdf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    // 2 0 obj - Pages
    offsets.push(pdf.len());
    pdf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

    // 3 0 obj - Page
    offsets.push(pdf.len());
    pdf.extend_from_slice(
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Contents 4 0 R /Resources << /Font << /F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> >> >> >>\nendobj\n",
    );

    // 4 0 obj - Content stream
    offsets.push(pdf.len());
    pdf.extend_from_slice(format!("4 0 obj\n<< /Length {stream_len} >>\nstream\n").as_bytes());
    pdf.extend_from_slice(stream_bytes);
    pdf.extend_from_slice(b"\nendstream\nendobj\n");

    let xref_offset = pdf.len();
    let num_entries = offsets.len() + 1;
    pdf.extend_from_slice(format!("xref\n0 {num_entries}\n").as_bytes());
    pdf.extend_from_slice(b"0000000000 65535 f \n");
    for off in &offsets {
        pdf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer\n<< /Size {num_entries} /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF"
        )
        .as_bytes(),
    );

    pdf
}

pub fn mask_wallet(address: &str) -> String {
    if address.len() <= 8 {
        return address.to_string();
    }
    let start = &address[..4];
    let end = &address[address.len() - 4..];
    format!("{start}...{end}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_starts_with_header() {
        let pdf = generate_ticket_pdf("Concert", "2026-06-01", "Lagos", "VIP", "GABC...WXYZ", "qr123");
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(pdf.windows(5).any(|w| w == b"%%EOF"));
    }

    #[test]
    fn test_mask_wallet() {
        assert_eq!(mask_wallet("GABCDEF123456WXYZ"), "GABC...WXYZ");
        assert_eq!(mask_wallet("short"), "short");
    }

    #[test]
    fn test_pdf_contains_title() {
        let pdf = generate_ticket_pdf("My Event", "2026-06-01", "Lagos", "VIP", "GABC...WXYZ", "qr123");
        let s = String::from_utf8_lossy(&pdf);
        assert!(s.contains("My Event"));
        assert!(s.contains("Agora"));
    }

    #[test]
    fn test_escape_pdf_text() {
        assert_eq!(escape_pdf_text("a(b)c"), "a\\(b\\)c");
    }
}
