/**
 * Issue #1789 — shared API base URL / apiUrl() helper tests.
 *
 * `EXPO_PUBLIC_API_URL` is documented as host-only (no `/api/v1` suffix).
 * `apiUrl()` must always join `/api/v1` and the given path safely, whether
 * or not the caller includes a leading slash.
 */

describe('apiUrl', () => {
  afterEach(() => {
    jest.resetModules();
  });

  it('joins the default base URL, /api/v1, and a leading-slash path', async () => {
    const { apiUrl } = await import('../api');
    expect(apiUrl('/profile/tickets')).toBe('http://localhost:8080/api/v1/profile/tickets');
  });

  it('joins the default base URL, /api/v1, and a path with no leading slash', async () => {
    const { apiUrl } = await import('../api');
    expect(apiUrl('profile/tickets')).toBe('http://localhost:8080/api/v1/profile/tickets');
  });

  it('does not duplicate /api/v1 when EXPO_PUBLIC_API_URL has a trailing slash', async () => {
    const originalEnv = process.env.EXPO_PUBLIC_API_URL;
    process.env.EXPO_PUBLIC_API_URL = 'http://localhost:9000/';

    jest.resetModules();
    const { apiUrl, API_BASE_URL } = await import('../api');

    expect(API_BASE_URL).toBe('http://localhost:9000');
    expect(apiUrl('/events/discover')).toBe('http://localhost:9000/api/v1/events/discover');

    process.env.EXPO_PUBLIC_API_URL = originalEnv;
  });

  it('respects a custom EXPO_PUBLIC_API_URL host', async () => {
    const originalEnv = process.env.EXPO_PUBLIC_API_URL;
    process.env.EXPO_PUBLIC_API_URL = 'http://10.0.2.2:3001';

    jest.resetModules();
    const { apiUrl } = await import('../api');

    expect(apiUrl('/organizer/delegation')).toBe('http://10.0.2.2:3001/api/v1/organizer/delegation');

    process.env.EXPO_PUBLIC_API_URL = originalEnv;
  });
});
