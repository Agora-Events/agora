import { renderHook, waitFor } from '@testing-library/react';
import { useEventDetails } from '../hooks/useEventDetails';
import { describe, it, expect, beforeAll, afterEach, afterAll, vi } from 'vitest';
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';
import { SWRConfig } from 'swr';
import React from 'react';

const mockEvent = {
  id: '1',
  name: 'Test Event',
};

const server = setupServer(
  http.get('/api/v1/events/1', () => {
    return HttpResponse.json(mockEvent);
  }),
  http.get('/api/v1/events/2', () => {
    return new HttpResponse(null, { status: 500 });
  })
);

beforeAll(() => server.listen());
afterEach(() => {
  server.resetHandlers();
});
afterAll(() => server.close());

const createWrapper = () => {
  return ({ children }: { children: React.ReactNode }) => (
    <SWRConfig value={{ provider: () => new Map(), dedupingInterval: 0, errorRetryInterval: 0 }}>
      {children}
    </SWRConfig>
  );
};

describe('useEventDetails', () => {
  it('should return pending state initially', async () => {
    const { result } = renderHook(() => useEventDetails('1'), {
      wrapper: createWrapper(),
    });
    expect(result.current.isLoading).toBe(true);
    expect(result.current.event).toBeUndefined();
  });

  it('should fetch event data successfully', async () => {
    const { result } = renderHook(() => useEventDetails('1'), {
      wrapper: createWrapper(),
    });
    
    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });
    
    expect(result.current.event).toEqual(mockEvent);
    expect(result.current.isError).toBeUndefined();
  });

  it('should return failure with retry on error', async () => {
    let callCount = 0;
    server.use(
      http.get('/api/v1/events/2', () => {
        callCount++;
        return new HttpResponse(null, { status: 500 });
      })
    );

    const { result } = renderHook(() => useEventDetails('2'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isError).toBeDefined();
    }, { timeout: 4000 });

    // SWR retries errors by default
    expect(callCount).toBeGreaterThan(1);
    expect(result.current.event).toBeUndefined();
  });
});
