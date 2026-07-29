import { renderHook, waitFor } from '@testing-library/react';
import { useEventDetails, EventApiError } from '../hooks/useEventDetails';
import { describe, it, expect, beforeAll, afterEach, afterAll, vi } from 'vitest';
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';
import { SWRConfig } from 'swr';
import React, { Component, type ReactNode } from 'react';

const mockEvent = {
  id: '1',
  name: 'Test Event',
};

const server = setupServer(
  http.get('/api/v1/events/1', () => {
    return HttpResponse.json(mockEvent);
  }),
  http.get('/api/v1/events/not-found', () => {
    return new HttpResponse(null, { status: 404 });
  }),
  http.get('/api/v1/events/server-error', () => {
    return new HttpResponse(null, { status: 500 });
  }),
);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

// Suppress React's console.error for expected thrown errors in tests
beforeAll(() => {
  vi.spyOn(console, 'error').mockImplementation(() => {});
});
afterAll(() => {
  vi.restoreAllMocks();
});

const createWrapper = () => {
  return ({ children }: { children: ReactNode }) => (
    <SWRConfig
      value={{
        provider: () => new Map(),
        dedupingInterval: 0,
        errorRetryInterval: 0,
        // Disable SWR's own retry so 500 tests don't loop
        shouldRetryOnError: false,
      }}
    >
      {children}
    </SWRConfig>
  );
};

/** Minimal error boundary to catch errors thrown by the hook under test. */
class ErrorBoundary extends Component<
  { children: ReactNode; onError: (e: Error) => void },
  { caught: boolean }
> {
  state = { caught: false };
  componentDidCatch(error: Error) {
    this.props.onError(error);
    this.setState({ caught: true });
  }
  render() {
    return this.state.caught ? null : this.props.children;
  }
}

describe('useEventDetails', () => {
  it('returns pending state initially', async () => {
    const { result } = renderHook(() => useEventDetails('1'), {
      wrapper: createWrapper(),
    });
    expect(result.current.isLoading).toBe(true);
    expect(result.current.event).toBeUndefined();
  });

  it('fetches event data successfully', async () => {
    const { result } = renderHook(() => useEventDetails('1'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.event).toEqual(mockEvent);
    expect(result.current.isError).toBeUndefined();
  });

  it('exposes a retry function', async () => {
    const { result } = renderHook(() => useEventDetails('1'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(typeof result.current.retry).toBe('function');
  });

  it('returns isError (EventApiError with status 404) for not-found — does not throw', async () => {
    const { result } = renderHook(() => useEventDetails('not-found'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isError).toBeDefined();
    }, { timeout: 4000 });

    expect(result.current.isError).toBeInstanceOf(EventApiError);
    expect((result.current.isError as EventApiError).status).toBe(404);
    expect(result.current.event).toBeUndefined();
  });

  it('throws EventApiError with status 500 so it reaches the error boundary', async () => {
    let caughtError: Error | null = null;

    const Wrapper = ({ children }: { children: ReactNode }) => (
      <SWRConfig
        value={{
          provider: () => new Map(),
          dedupingInterval: 0,
          errorRetryInterval: 0,
          shouldRetryOnError: false,
        }}
      >
        <ErrorBoundary onError={(e) => { caughtError = e; }}>
          {children}
        </ErrorBoundary>
      </SWRConfig>
    );

    renderHook(() => useEventDetails('server-error'), { wrapper: Wrapper });

    await waitFor(() => {
      expect(caughtError).not.toBeNull();
    }, { timeout: 4000 });

    expect(caughtError).toBeInstanceOf(EventApiError);
    expect((caughtError as unknown as EventApiError).status).toBe(500);
  });
});
