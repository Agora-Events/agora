import { organizerApi } from '../organizerApi';
import AsyncStorage from '@react-native-async-storage/async-storage';

/**
 * Issue #1179 — organizer service API tests.
 *
 * Tests cover:
 *   - Request endpoints and body shape
 *   - Auth header inclusion when token exists
 *   - Error responses converted to thrown errors
 *   - scanTicket special handling (no throw on non-2xx)
 */

// ── Fixtures ──────────────────────────────────────────────────────────────────

const baseEventId = 'evt-1';
const baseGateId = 'gate-1';
const baseStaffWallet = 'GA7QYNF7SOWQ3GLR2BGMZEHXAVIRZA4KVWLTJJFC7MGXUA74P7UJVSGZ';

function jsonResponse(status: number, body: unknown): Response {
  return {
    ok: status >= 200 && status < 300,
    status,
    json: async () => body,
    headers: new Headers({ 'Content-Type': 'application/json' }),
  } as Response;
}

// ── Setup ─────────────────────────────────────────────────────────────────────

beforeEach(() => {
  jest.restoreAllMocks();
  global.fetch = jest.fn();

  // Clear auth token before each test
  jest.spyOn(AsyncStorage, 'getItem').mockResolvedValue(null);
});

describe('generateDelegationQR', () => {
  it('POSTs to /organizer/delegation with correct payload and auth header', async () => {
    const mockToken = 'mock-jwt-token';
    jest.spyOn(AsyncStorage, 'getItem').mockResolvedValue(mockToken);

    const mockResponse = {
      code: 'qr-deleg-abc123',
      event_id: baseEventId,
      gate_id: baseGateId,
      expires_at: '2026-09-26T12:00:00Z',
      permissions: ['scan'],
    };
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(200, mockResponse));

    const result = await organizerApi.generateDelegationQR(
      baseEventId,
      baseGateId,
      baseStaffWallet,
      480
    );

    expect(global.fetch).toHaveBeenCalledTimes(1);
    const [url, init] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toMatch(/\/organizer\/delegation$/);
    expect(init.method).toBe('POST');
    expect(init.headers.Authorization).toBe(`Bearer ${mockToken}`);
    expect(JSON.parse(init.body)).toMatchObject({
      event_id: baseEventId,
      gate_id: baseGateId,
      staff_wallet: baseStaffWallet,
      expires_in_minutes: 480,
    });
    expect(result).toEqual(mockResponse);
  });

  it('omits Authorization header when no auth token exists', async () => {
    const mockResponse = {
      code: 'qr-deleg-xyz',
      event_id: baseEventId,
      gate_id: baseGateId,
      expires_at: '2026-09-26T12:00:00Z',
      permissions: ['scan'],
    };
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(200, mockResponse));

    await organizerApi.generateDelegationQR(baseEventId, baseGateId, baseStaffWallet);

    const [, init] = (global.fetch as jest.Mock).mock.calls[0];
    expect(init.headers.Authorization).toBeUndefined();
  });

  it('throws an error with a useful message when the request fails', async () => {
    jest.spyOn(AsyncStorage, 'getItem').mockResolvedValue('token');
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(500, { error: 'Internal server error' }));

    await expect(
      organizerApi.generateDelegationQR(baseEventId, baseGateId, baseStaffWallet)
    ).rejects.toThrow('Failed to generate delegation QR code');
  });
});

describe('verifyDelegationQR', () => {
  it('POSTs to /organizer/delegation/verify with the QR code and auth header', async () => {
    const mockToken = 'mock-jwt-token';
    jest.spyOn(AsyncStorage, 'getItem').mockResolvedValue(mockToken);

    const mockResponse = {
      valid: true,
      event_id: baseEventId,
      gate_id: baseGateId,
      permissions: ['scan', 'broadcast'],
    };
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(200, mockResponse));

    const result = await organizerApi.verifyDelegationQR('qr-deleg-abc123');

    expect(global.fetch).toHaveBeenCalledTimes(1);
    const [url, init] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toMatch(/\/organizer\/delegation\/verify$/);
    expect(JSON.parse(init.body)).toMatchObject({ code: 'qr-deleg-abc123' });
    expect(init.headers.Authorization).toBe(`Bearer ${mockToken}`);
    expect(result).toEqual(mockResponse);
  });

  it('throws an error when verification fails', async () => {
    jest.spyOn(AsyncStorage, 'getItem').mockResolvedValue('token');
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(401, { error: 'Invalid QR code' }));

    await expect(organizerApi.verifyDelegationQR('invalid-code')).rejects.toThrow(
      'Failed to verify delegation QR code'
    );
  });
});

describe('createDoorTicket', () => {
  const baseRequest = {
    event_id: baseEventId,
    ticket_tier_id: 'tier-ga',
    quantity: 2,
    payment_method: 'usdc' as const,
    attendee_email: 'attendee@example.com',
    attendee_name: 'John Doe',
  };

  it('POSTs to /organizer/door-ticket with correct payload', async () => {
    const mockToken = 'mock-jwt-token';
    jest.spyOn(AsyncStorage, 'getItem').mockResolvedValue(mockToken);

    const mockResponse: {
      success: boolean;
      tickets: Array<{
        id: string;
        qr_code: string;
        ticket_tier_id: string;
        attendee_name: string;
      }>;
      transaction_id?: string;
    } = {
      success: true,
      tickets: [
        {
          id: 'ticket-1',
          qr_code: 'qr-abc123',
          ticket_tier_id: 'tier-ga',
          attendee_name: 'John Doe',
        },
        {
          id: 'ticket-2',
          qr_code: 'qr-def456',
          ticket_tier_id: 'tier-ga',
          attendee_name: 'John Doe',
        },
      ],
      transaction_id: 'txn-xyz789',
    };
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(200, mockResponse));

    const result = await organizerApi.createDoorTicket(baseRequest);

    expect(global.fetch).toHaveBeenCalledTimes(1);
    const [url, init] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toMatch(/\/organizer\/door-ticket$/);
    expect(JSON.parse(init.body)).toMatchObject(baseRequest);
    expect(init.headers.Authorization).toBe(`Bearer ${mockToken}`);
    expect(result).toEqual(mockResponse);
  });

  it('throws an error when ticket creation fails', async () => {
    jest.spyOn(AsyncStorage, 'getItem').mockResolvedValue('token');
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(400, { error: 'Invalid tier' }));

    await expect(organizerApi.createDoorTicket(baseRequest)).rejects.toThrow(
      'Failed to create door ticket'
    );
  });
});

describe('sendBroadcastAlert', () => {
  const baseAlert = {
    event_id: baseEventId,
    message: 'Please proceed to your gates',
    alert_type: 'info' as const,
  };

  it('POSTs to /organizer/broadcast with correct payload', async () => {
    const mockToken = 'mock-jwt-token';
    jest.spyOn(AsyncStorage, 'getItem').mockResolvedValue(mockToken);

    const mockResponse = { success: true, message: 'Alert sent to 150 attendees' };
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(200, mockResponse));

    const result = await organizerApi.sendBroadcastAlert(baseAlert);

    expect(global.fetch).toHaveBeenCalledTimes(1);
    const [url, init] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toMatch(/\/organizer\/broadcast$/);
    expect(JSON.parse(init.body)).toMatchObject(baseAlert);
    expect(result).toEqual(mockResponse);
  });

  it('throws an error when broadcast fails', async () => {
    jest.spyOn(AsyncStorage, 'getItem').mockResolvedValue('token');
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(403, { error: 'Not authorized' }));

    await expect(organizerApi.sendBroadcastAlert(baseAlert)).rejects.toThrow(
      'Failed to send broadcast alert'
    );
  });
});

describe('getEventGates', () => {
  it('GETs from /events/:eventId/gates with auth header', async () => {
    const mockToken = 'mock-jwt-token';
    jest.spyOn(AsyncStorage, 'getItem').mockResolvedValue(mockToken);

    const mockResponse = [
      {
        gate_id: 'gate-1',
        gate_name: 'Main Entrance',
        current_wait_time_minutes: 5,
        throughput_per_minute: 12,
        staff_count: 3,
        congestion_level: 'low' as const,
      },
    ];
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(200, mockResponse));

    const result = await organizerApi.getEventGates(baseEventId);

    expect(global.fetch).toHaveBeenCalledTimes(1);
    const [url, init] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toBe(`http://localhost:8080/api/v1/events/${baseEventId}/gates`);
    expect(init.method).toBe('GET');
    expect(init.headers.Authorization).toBe(`Bearer ${mockToken}`);
    expect(result).toEqual(mockResponse);
  });

  it('uses the default API_BASE when EXPO_PUBLIC_API_URL is not set', async () => {
    const originalEnv = process.env.EXPO_PUBLIC_API_URL;
    delete process.env.EXPO_PUBLIC_API_URL;

    jest.spyOn(AsyncStorage, 'getItem').mockResolvedValue('token');
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(200, []));

    await organizerApi.getEventGates(baseEventId);

    const [url] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toMatch(/^http:\/\/localhost:8080\/api\/v1/);

    // Restore
    process.env.EXPO_PUBLIC_API_URL = originalEnv;
  });

  it('throws an error when the request fails', async () => {
    jest.spyOn(AsyncStorage, 'getItem').mockResolvedValue('token');
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(500, { error: 'DB unavailable' }));

    await expect(organizerApi.getEventGates(baseEventId)).rejects.toThrow('Failed to fetch event gates');
  });
});

describe('getStaffAuthorizations', () => {
  it('GETs from /organizer/:eventId/staff with auth header', async () => {
    const mockToken = 'mock-jwt-token';
    jest.spyOn(AsyncStorage, 'getItem').mockResolvedValue(mockToken);

    const mockResponse = [
      {
        event_id: baseEventId,
        staff_wallet: 'GA1',
        gate_id: 'gate-1',
        authorized: true,
        timestamp: '2026-09-25T10:00:00Z',
      },
    ];
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(200, mockResponse));

    const result = await organizerApi.getStaffAuthorizations(baseEventId);

    expect(global.fetch).toHaveBeenCalledTimes(1);
    const [url] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toBe(`http://localhost:8080/api/v1/organizer/${baseEventId}/staff`);
    expect(result).toEqual(mockResponse);
  });

  it('throws an error when fetching staff authorizations fails', async () => {
    jest.spyOn(AsyncStorage, 'getItem').mockResolvedValue('token');
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(404, { error: 'Event not found' }));

    await expect(organizerApi.getStaffAuthorizations(baseEventId)).rejects.toThrow(
      'Failed to fetch staff authorizations'
    );
  });
});

describe('scanTicket', () => {
  const baseTicketId = 'ticket-123';
  const baseScanRequest = {
    payload: 'signed-payload-base64',
    signature: 'signature-base64',
    public_key: baseStaffWallet,
    mode: 'checkin' as const,
  };

  it('POSTs to /tickets/:ticketId/scan with auth header and returns status/result/message', async () => {
    const mockToken = 'mock-jwt-token';
    jest.spyOn(AsyncStorage, 'getItem').mockResolvedValue(mockToken);

    const mockResponse = {
      data: {
        valid: true,
        scanned: true,
        ticket_id: baseTicketId,
        ticket_status: 'checked_in',
        scanned_at: '2026-09-25T10:30:00Z',
        message: 'Ticket validated',
      },
    };
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(200, mockResponse));

    const result = await organizerApi.scanTicket(baseTicketId, baseScanRequest);

    expect(global.fetch).toHaveBeenCalledTimes(1);
    const [url, init] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toBe(`http://localhost:8080/api/v1/tickets/${baseTicketId}/scan`);
    expect(init.method).toBe('POST');
    expect(JSON.parse(init.body)).toMatchObject(baseScanRequest);
    expect(init.headers.Authorization).toBe(`Bearer ${mockToken}`);
    expect(result).toEqual({
      status: 200,
      result: mockResponse.data,
      message: 'Scan processed',
    });
  });

  it('handles a 409 duplicate scan without throwing', async () => {
    const mockToken = 'mock-jwt-token';
    jest.spyOn(AsyncStorage, 'getItem').mockResolvedValue(mockToken);

    const mockResponse = {
      data: null,
      message: 'Ticket already scanned',
    };
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(409, mockResponse));

    const result = await organizerApi.scanTicket(baseTicketId, baseScanRequest);

    // Should NOT throw, but return status 409 with null result
    expect(result.status).toBe(409);
    expect(result.result).toBeNull();
    expect(result.message).toBe('Ticket already scanned');
  });

  it('handles a 4xx error with invalid signature/expired payload', async () => {
    const mockToken = 'mock-jwt-token';
    jest.spyOn(AsyncStorage, 'getItem').mockResolvedValue(mockToken);

    const mockResponse = {
      data: null,
      message: 'Invalid signature',
    };
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(401, mockResponse));

    const result = await organizerApi.scanTicket(baseTicketId, baseScanRequest);

    expect(result.status).toBe(401);
    expect(result.result).toBeNull();
    expect(result.message).toBe('Invalid signature');
  });

  it('handles JSON parse errors gracefully', async () => {
    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: false,
      status: 500,
      json: async () => {
        throw new Error('JSON parse error');
      },
      headers: new Headers({ 'Content-Type': 'application/json' }),
    } as Response);

    const result = await organizerApi.scanTicket(baseTicketId, baseScanRequest);

    expect(result.status).toBe(500);
    expect(result.result).toBeNull();
    expect(result.message).toBe('Scan failed');
  });
});

describe('revokeStaffAuthorization', () => {
  it('POSTs to /organizer/:eventId/staff/revoke with correct payload', async () => {
    const mockToken = 'mock-jwt-token';
    jest.spyOn(AsyncStorage, 'getItem').mockResolvedValue(mockToken);

    const mockResponse = { success: true };
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(200, mockResponse));

    const result = await organizerApi.revokeStaffAuthorization(baseEventId, baseStaffWallet);

    expect(global.fetch).toHaveBeenCalledTimes(1);
    const [url, init] = (global.fetch as jest.Mock).mock.calls[0];
    expect(url).toBe(`http://localhost:8080/api/v1/organizer/${baseEventId}/staff/revoke`);
    expect(JSON.parse(init.body)).toMatchObject({ staff_wallet: baseStaffWallet });
    expect(result).toEqual(mockResponse);
  });

  it('throws an error when revocation fails', async () => {
    jest.spyOn(AsyncStorage, 'getItem').mockResolvedValue('token');
    (global.fetch as jest.Mock).mockResolvedValueOnce(jsonResponse(403, { error: 'Not authorized' }));

    await expect(
      organizerApi.revokeStaffAuthorization(baseEventId, baseStaffWallet)
    ).rejects.toThrow('Failed to revoke staff authorization');
  });
});
