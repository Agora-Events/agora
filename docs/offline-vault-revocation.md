# Offline Ticket Vault — Revocation Staleness Tradeoff

**Issue #1179** | Feature: Offline Ticket Vault

---

## The problem

The offline vault design is intentionally hermetic: the scanner device verifies
tickets using only an Ed25519 public key pre-synced before the event, with no
network call at gate time. This is a feature — it works when the venue has no
cellular signal and when the server is unreachable. But it creates a staleness
window for revoked or cancelled tickets.

### Concrete scenario

1. Attendee purchases a ticket at T=0. Their device stores the signing secret;
   the scanner receives the corresponding public key at sync time.
2. At T=2h, the organiser refunds and cancels the ticket server-side (e.g. the
   attendee filed a chargeback).
3. At T=3h, the event starts. The scanner has not re-synced since T=0.
4. The attendee presents their ticket. The scanner's public key still matches;
   the signature is valid; the timestamp is within the window.
5. **Result: the scanner admits the attendee despite the cancellation.**

The same scenario applies to:
- Tickets transferred on-chain after the scanner's last sync (old holder can
  still scan in with the pre-transfer key).
- Tickets flagged as fraudulent after a post-purchase review.
- Tickets invalidated because the buyer's payment failed or was charged back.

---

## Why this is inherent, not a bug

Full offline verification without a revocation channel is an unsolvable
tradeoff. You can pick at most two of:

| Property | This design | Server-round-trip design |
|---|---|---|
| Works offline | ✓ | ✗ |
| Instantly reflects revocations | ✗ | ✓ |
| Forgery-resistant | ✓ | ✓ |

HMAC (the alternative the issue initially mentioned) does not help: it has the
same staleness problem and additionally requires trusting each scanner with the
shared secret.

---

## Mitigations (implemented)

### 1. Scanner displays last-sync time
The scanner module exposes `getScanLog()` and `registeredTicketCount`. The
scanner UI **must** prominently display the timestamp of the last public-key
sync so gate staff can decide whether to accept borderline cases (e.g. "last
synced 6 hours ago — be cautious").

### 2. Rotating payload window
The 15-second rotating payload means a screenshot or recorded QR is only valid
for at most 75 seconds (15s window + 60s drift tolerance). This prevents
screenshot fraud but does not help with pre-sync revocation.

### 3. Per-ticket public keys
Each ticket has its own Ed25519 keypair. Revoking a specific ticket means
removing (or zeroing) its public key from the scanner's key map on the next
sync. After a sync, that ticket can no longer be admitted.

### 4. `addPublicKey` for partial syncs
`OfflineScanner.addPublicKey()` can be called at any time, even mid-event, if
the scanner gains connectivity and the app chooses to pull a partial update.
This can narrow the staleness window significantly in venues with intermittent
connectivity.

---

## Recommended operational practices

1. **Sync as close to event start as possible.** Run `addPublicKey` /
   full re-sync within 15 minutes of doors opening to minimise the window
   during which a post-sync revocation could be exploited.

2. **Display last-sync prominently.** Gate staff should see the sync time
   on the scanner screen at all times, not just in a settings menu.

3. **Re-sync at regular intervals if connectivity permits.** Even a 30-minute
   sync schedule during a 4-hour event would limit the staleness window to
   30 minutes for most revocations.

4. **Revocation list alongside key map.** At sync time, include an explicit
   revocation list (set of ticket IDs to reject regardless of signature
   validity). The `OfflineScanner` can be extended to check this list after
   signature verification but before recording the scan.

5. **Staff protocol for suspicious cases.** If a ticket was purchased very
   recently (e.g. within the last sync window), staff should verify the
   attendee's identity against the booking confirmation before admitting.

---

## What this design does NOT do

- It does not prevent a refunded attendee from entering if the scanner has not
  re-synced since the refund was processed.
- It does not handle multi-day events where the key map needs nightly rotation
  (extend `addPublicKey` + scheduled sync for this use case).
- It does not handle the case where the attendee's phone clock is set more than
  60 seconds off (SCANNER_CLOCK_DRIFT_S). In that case the payload will be
  rejected as `EXPIRED_TIMESTAMP`. Advise attendees to keep automatic time
  sync enabled.

---

*Last updated: 2026-08-18 (issue #1179)*
