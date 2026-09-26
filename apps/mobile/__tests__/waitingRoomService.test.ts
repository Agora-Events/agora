import {
  fetchPowChallenge,
  getWaitingRoomStatus,
  joinWaitingRoom,
  openWaitingRoomStream,
  solvePow,
  WaitingRoomError,
} from '@/services/waitingRoom';
import { sha256Hex } from '@/utils/sha256';

const EVENT_ID = '550e8400-e29b-41d4-a716-446655440000';
const CLIENT_ID = 'GCLIENTWALLET';

describe('waiting room API client', () => {
  const originalFetch = global.fetch;

  afterEach(() => {
    global.fetch = originalFetch;
  });

  function mockFetchJson(status: number, body: unknown) {
    global.fetch = jest.fn().mockResolvedValue({
      ok: status >= 200 && status < 300,
      status,
      json: async () => body,
    } as any);
  }

  it('fetchPowChallenge unwraps the server { success, data, message } envelope', async () => {
    mockFetchJson(200, {
      success: true,
      data: { challenge: 'deadbeef', difficulty: 4, expires_in: 300 },
      message: 'Proof-of-work challenge issued',
    });

    const challenge = await fetchPowChallenge(EVENT_ID);
    expect(challenge).toEqual({ challenge: 'deadbeef', difficulty: 4, expires_in: 300 });
  });

  it('joinWaitingRoom posts the PoW solution', async () => {
    mockFetchJson(200, {
      success: true,
      data: {
        status: 'waiting',
        position: 142,
        queue_size: 1000,
        estimated_wait_seconds: 142,
        grant_token: null,
      },
      message: 'Joined the queue',
    });

    const status = await joinWaitingRoom({
      event_id: EVENT_ID,
      client_id: CLIENT_ID,
      challenge: 'deadbeef',
      nonce: '12345',
    });
    expect(status.status).toBe('waiting');
    expect(status.position).toBe(142);

    const [, init] = (global.fetch as jest.Mock).mock.calls[0];
    expect(JSON.parse((init as any).body)).toEqual({
      event_id: EVENT_ID,
      client_id: CLIENT_ID,
      challenge: 'deadbeef',
      nonce: '12345',
    });
  });

  it('surfaces server error messages with their status', async () => {
    mockFetchJson(422, { code: 422, message: 'Proof-of-work solution is incorrect' });

    await expect(
      joinWaitingRoom({ event_id: EVENT_ID, client_id: CLIENT_ID, challenge: 'c', nonce: 'n' })
    ).rejects.toMatchObject({
      name: 'WaitingRoomError',
      status: 422,
      message: 'Proof-of-work solution is incorrect',
    });
  });

  it('getWaitingRoomStatus throws WaitingRoomError (404) when not in queue', async () => {
    mockFetchJson(404, { code: 404, message: 'You are not in the queue for this event' });

    await expect(getWaitingRoomStatus(EVENT_ID, CLIENT_ID)).rejects.toBeInstanceOf(WaitingRoomError);
  });

  it('solvePow produces a nonce the server-side rule accepts', () => {
    const challenge = 'cafebabe'.repeat(4);
    const nonce = solvePow(challenge, 2, 100_000);
    expect(sha256Hex(`${challenge}${nonce}`).startsWith('00')).toBe(true);
  });
});

describe('openWaitingRoomStream (SSE via XHR)', () => {
  class FakeXHR {
    responseText = '';
    onprogress: (() => void) | null = null;
    onload: (() => void) | null = null;
    onerror: (() => void) | null = null;
    aborted = false;
    sent = false;
    sentUrl = '';

    open(_method: string, url: string) {
      this.sentUrl = url;
    }
    setRequestHeader() {}
    send() {
      this.sent = true;
    }
    abort() {
      this.aborted = true;
    }

    pushChunk(chunk: string) {
      this.responseText += chunk;
      this.onprogress?.();
    }
  }

  const originalXHR = global.XMLHttpRequest;

  afterEach(() => {
    (global as any).XMLHttpRequest = originalXHR;
  });

  function mockXhr(fake: FakeXHR) {
    (global as any).XMLHttpRequest = jest.fn(() => fake);
  }

  it('parses position frames and closes with the admitted grant', () => {
    const fake = new FakeXHR();
    mockXhr(fake);

    const onPosition = jest.fn();
    const onAdmitted = jest.fn();
    const onClosed = jest.fn();
    const onError = jest.fn();

    openWaitingRoomStream(EVENT_ID, CLIENT_ID, {
      onPosition,
      onAdmitted,
      onError,
      onClosed,
    });

    expect(fake.sent).toBe(true);
    expect(fake.sentUrl).toContain('/api/v1/waiting-room/stream');

    fake.pushChunk(
      'data: {"type":"position","position":142,"queue_size":1000,"estimated_wait_seconds":45}\n\n'
    );
    expect(onPosition).toHaveBeenCalledWith(142, 1000, 45);

    fake.pushChunk('data: {"type":"admitted","grant_token":"signed-grant-jwt"}\n\n');
    expect(onAdmitted).toHaveBeenCalledWith('signed-grant-jwt');
    expect(fake.aborted).toBe(true);
    expect(onClosed).toHaveBeenCalled();
    expect(onError).not.toHaveBeenCalled();
  });

  it('ignores keepalive comments and split frames', () => {
    const fake = new FakeXHR();
    mockXhr(fake);

    const onPosition = jest.fn();
    openWaitingRoomStream(EVENT_ID, CLIENT_ID, {
      onPosition,
      onAdmitted: jest.fn(),
      onError: jest.fn(),
      onClosed: jest.fn(),
    });

    // Keepalive comment has no `data:` line.
    fake.pushChunk(': keepalive\n\n');
    expect(onPosition).not.toHaveBeenCalled();

    // A frame arriving across two progress ticks is still parsed.
    fake.pushChunk('data: {"type":"position","position":1,"queue_size":2,"esti');
    expect(onPosition).not.toHaveBeenCalled();
    fake.pushChunk('mated_wait_seconds":3}\n\n');
    expect(onPosition).toHaveBeenCalledWith(1, 2, 3);
  });
});
