// ==============================================================================
// Agora k6 Stress / Load Test — browse -> queue -> checkout -> scan (Issue #1178)
// ==============================================================================
// Note on file extension: the issue's target location names this
// `scripts/stress_test.rs`, but k6 scripts run on k6's embedded Goja
// JavaScript runtime — it cannot execute Rust. This is the equivalent
// modular k6 script at `scripts/stress_test.js`; the Rust side of the
// suite (criterion benchmarks, the chaos harness, and the re-org runner)
// lives under `server/benches/` and `server/tests/` per the rest of the
// issue's target locations.
//
// Simulates the realistic funnel a real Agora attendee walks through:
//   1. browse   — list/search events, view an event, view its ticket tiers
//   2. queue    — solve the SHA-256 proof-of-work gate and join the virtual
//                 waiting room (agora_server::handlers::waiting_room)
//   3. checkout — generate a signed QR payload for a ticket
//                 (agora_server::handlers::qr_payload). Actual on-chain
//                 payment happens client-side via the Soroban SDK against
//                 the `ticket_payment` contract, outside this REST surface,
//                 so it is out of scope for an HTTP load generator.
//   4. scan     — present the QR payload at the gate
//                 (POST /api/v1/tickets/:id/scan)
//
// Usage:
//   k6 run scripts/stress_test.js
//   BASE_URL=https://staging.agora.example VUS=50 DURATION=5m k6 run scripts/stress_test.js
//   k6 run --summary-export=summary.json scripts/stress_test.js
//
// Env vars:
//   BASE_URL   Agora API base URL          (default: http://localhost:3001)
//   VUS        peak virtual users          (default: 20)
//   DURATION   sustained load duration     (default: 2m)
//   RAMP_TIME  ramp up/down duration       (default: 30s)
// ==============================================================================

import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { sha256 } from 'k6/crypto';
import { Trend, Rate, Counter } from 'k6/metrics';
import { randomIntBetween } from 'https://jslib.k6.io/k6-utils/1.2.0/index.js';

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

const BASE_URL = __ENV.BASE_URL || 'http://localhost:3001';
const API = `${BASE_URL}/api/v1`;
const PEAK_VUS = Number(__ENV.VUS || 20);
const DURATION = __ENV.DURATION || '2m';
const RAMP_TIME = __ENV.RAMP_TIME || '30s';
// Safety valve for the proof-of-work solver — a wildly high difficulty
// (server misconfiguration) must not spin a VU forever.
const MAX_POW_ATTEMPTS = Number(__ENV.MAX_POW_ATTEMPTS || 2_000_000);

// ---------------------------------------------------------------------------
// Custom metrics — one Trend per funnel phase so CI can report p50/p95/p99
// per stage, not just an aggregate. `scripts/generate_perf_report.mjs`
// reads these back out of the k6 summary JSON.
// ---------------------------------------------------------------------------

const browseDuration = new Trend('agora_browse_duration', true);
const queueDuration = new Trend('agora_queue_duration', true);
const checkoutDuration = new Trend('agora_checkout_duration', true);
const scanDuration = new Trend('agora_scan_duration', true);
const powSolveDuration = new Trend('agora_pow_solve_duration', true);
const powAttempts = new Trend('agora_pow_attempts');
const funnelCompleted = new Counter('agora_funnel_completed_total');
const funnelErrors = new Rate('agora_funnel_error_rate');

// ---------------------------------------------------------------------------
// k6 scenario / threshold configuration
// ---------------------------------------------------------------------------

export const options = {
  scenarios: {
    browse_to_scan_funnel: {
      executor: 'ramping-vus',
      exec: 'fullFunnel',
      startVUs: 0,
      stages: [
        { duration: RAMP_TIME, target: PEAK_VUS },
        { duration: DURATION, target: PEAK_VUS },
        { duration: RAMP_TIME, target: 0 },
      ],
      gracefulRampDown: '10s',
    },
    // A lighter, browse-only scenario running the whole time to model
    // casual traffic (users who never queue) alongside the funnel above.
    browse_only: {
      executor: 'constant-vus',
      exec: 'browseOnly',
      vus: Math.max(2, Math.floor(PEAK_VUS / 4)),
      duration: DURATION,
      startTime: RAMP_TIME,
    },
  },
  thresholds: {
    http_req_failed: ['rate<0.05'],
    http_req_duration: ['p(95)<1500', 'p(99)<3000'],
    agora_browse_duration: ['p(95)<800'],
    agora_funnel_error_rate: ['rate<0.10'],
  },
};

// ---------------------------------------------------------------------------
// Proof-of-work solver — mirrors agora_server::services::pow::verify_pow
// exactly: SHA-256(challenge || nonce) must have `difficulty` leading hex
// zeros, where nonce is the decimal string form of an increasing counter
// (see server/src/services/pow.rs::solve_pow).
// ---------------------------------------------------------------------------

function solvePow(challenge, difficulty) {
  const target = '0'.repeat(difficulty);
  const start = Date.now();
  for (let nonce = 0; nonce < MAX_POW_ATTEMPTS; nonce++) {
    const digestHex = sha256(challenge + String(nonce), 'hex');
    if (digestHex.substring(0, difficulty) === target) {
      powSolveDuration.add(Date.now() - start);
      powAttempts.add(nonce + 1);
      return String(nonce);
    }
  }
  throw new Error(`pow: exhausted ${MAX_POW_ATTEMPTS} attempts at difficulty ${difficulty}`);
}

function uuidv4() {
  // RFC 4122 v4 via Math.random — fine for load-test payload uniqueness,
  // not for anything security-sensitive.
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
    const r = (Math.random() * 16) | 0;
    const v = c === 'x' ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });
}

// ---------------------------------------------------------------------------
// Phase 1: Browse
// ---------------------------------------------------------------------------

/// Returns the id of an event to carry into the queue/checkout phases, or
/// `null` when the environment has no events seeded (browse still ran and
/// still contributes latency data either way).
function browse() {
  const start = Date.now();
  let chosenEventId = null;

  group('browse', () => {
    const list = http.get(`${API}/events?limit=20`, { tags: { phase: 'browse' } });
    check(list, { 'browse: list events succeeds': (r) => r.status === 200 });

    const upcoming = http.get(`${API}/events/upcoming?limit=10`, { tags: { phase: 'browse' } });
    check(upcoming, { 'browse: upcoming events succeeds': (r) => r.status === 200 });

    const searchTerm = ['music', 'tech', 'art', 'sports'][randomIntBetween(0, 3)];
    const search = http.get(`${API}/events/search?q=${searchTerm}`, { tags: { phase: 'browse' } });
    check(search, { 'browse: search succeeds': (r) => r.status === 200 || r.status === 400 });

    chosenEventId = extractFirstEventId(list) || extractFirstEventId(upcoming);

    if (chosenEventId) {
      const detail = http.get(`${API}/events/${chosenEventId}`, { tags: { phase: 'browse' } });
      check(detail, { 'browse: event detail succeeds': (r) => r.status === 200 || r.status === 404 });

      const tiers = http.get(`${API}/events/${chosenEventId}/ticket-tiers`, {
        tags: { phase: 'browse' },
      });
      check(tiers, {
        'browse: ticket tiers succeeds': (r) => r.status === 200 || r.status === 404,
      });
    }
  });

  browseDuration.add(Date.now() - start);
  return chosenEventId;
}

function extractFirstEventId(res) {
  if (!res || res.status !== 200) return null;
  try {
    const body = res.json();
    const items = (body && body.data && (body.data.items || body.data.events || body.data)) || [];
    if (Array.isArray(items) && items.length > 0 && items[0] && items[0].id) {
      return items[0].id;
    }
  } catch (_e) {
    // Non-JSON or unexpected shape — treat as "no event available" rather
    // than failing the whole iteration.
  }
  return null;
}

// ---------------------------------------------------------------------------
// Phase 2: Queue (proof-of-work gate + virtual waiting room)
// ---------------------------------------------------------------------------

/// Returns true once the client is admitted (or the event has no queue
/// configured, i.e. the join call itself fails gracefully), false on a hard
/// failure worth counting against the error rate.
function queue(eventId, clientId) {
  if (!eventId) return true; // nothing to queue for in this environment

  const start = Date.now();
  let ok = true;

  group('queue', () => {
    const challengeRes = http.post(
      `${API}/waiting-room/challenge`,
      JSON.stringify({ event_id: eventId }),
      { headers: { 'Content-Type': 'application/json' }, tags: { phase: 'queue' } },
    );
    if (challengeRes.status !== 200) {
      // Waiting room may not be configured for this event/environment —
      // that's a valid non-error outcome for a load test, not a bug.
      return;
    }

    let challenge, difficulty;
    try {
      const body = challengeRes.json();
      challenge = body.data.challenge;
      difficulty = body.data.difficulty;
    } catch (_e) {
      ok = false;
      return;
    }

    const nonce = solvePow(challenge, difficulty);

    const joinRes = http.post(
      `${API}/waiting-room/join`,
      JSON.stringify({ event_id: eventId, client_id: clientId, challenge, nonce }),
      { headers: { 'Content-Type': 'application/json' }, tags: { phase: 'queue' } },
    );
    ok = check(joinRes, {
      'queue: join succeeds or reports sold out': (r) =>
        r.status === 200 || r.status === 409 || r.status === 422,
    });

    if (joinRes.status === 200) {
      const statusRes = http.get(
        `${API}/waiting-room/status?event_id=${eventId}&client_id=${clientId}`,
        { tags: { phase: 'queue' } },
      );
      check(statusRes, { 'queue: status check succeeds': (r) => r.status === 200 || r.status === 404 });
    }
  });

  queueDuration.add(Date.now() - start);
  return ok;
}

// ---------------------------------------------------------------------------
// Phase 3: Checkout (signed QR issuance — the platform-side half of a
// purchase; the on-chain leg runs through the Soroban `ticket_payment`
// contract via the client's wallet, not this REST API).
// ---------------------------------------------------------------------------

function checkout(eventId) {
  const start = Date.now();
  const ticketId = uuidv4();

  const res = http.post(
    `${API}/qr/generate`,
    JSON.stringify({
      qr_type: 'ticket',
      data: { event_id: eventId, ticket_id: ticketId },
      ticket_id: ticketId,
      expires_in_seconds: 3600,
    }),
    { headers: { 'Content-Type': 'application/json' }, tags: { phase: 'checkout' } },
  );

  checkoutDuration.add(Date.now() - start);

  const ok = check(res, { 'checkout: qr generation succeeds': (r) => r.status === 200 });
  if (!ok) return null;

  try {
    const body = res.json();
    return { ticketId, qr: body.data };
  } catch (_e) {
    return null;
  }
}

// ---------------------------------------------------------------------------
// Phase 4: Scan (gate check-in)
// ---------------------------------------------------------------------------

function scan(ticketId, qr) {
  if (!qr) return;

  const start = Date.now();
  const res = http.post(
    `${API}/tickets/${ticketId}/scan`,
    JSON.stringify({
      payload: qr.payload,
      signature: qr.signature,
      public_key: qr.public_key,
    }),
    { headers: { 'Content-Type': 'application/json' }, tags: { phase: 'scan' } },
  );
  scanDuration.add(Date.now() - start);

  // A synthetic ticket_id was never actually minted on-chain, so a 404/422
  // here is expected under load-test conditions — only a 5xx indicates the
  // scan endpoint itself is unhealthy under load.
  check(res, {
    'scan: endpoint does not 5xx under load': (r) => r.status < 500,
  });
}

// ---------------------------------------------------------------------------
// Scenario entry points
// ---------------------------------------------------------------------------

export function fullFunnel() {
  const clientId = `k6-vu-${__VU}-iter-${__ITER}`;
  let succeeded = true;

  const eventId = browse();
  sleep(randomIntBetween(1, 3));

  succeeded = queue(eventId, clientId) && succeeded;
  sleep(randomIntBetween(1, 2));

  if (eventId) {
    const checkoutResult = checkout(eventId);
    sleep(randomIntBetween(1, 2));
    if (checkoutResult) {
      scan(checkoutResult.ticketId, checkoutResult.qr);
      funnelCompleted.add(1);
    } else {
      succeeded = false;
    }
  }

  funnelErrors.add(!succeeded);
  sleep(randomIntBetween(1, 4));
}

export function browseOnly() {
  browse();
  sleep(randomIntBetween(2, 6));
}
