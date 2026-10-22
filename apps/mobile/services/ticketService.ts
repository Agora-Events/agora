import AsyncStorage from '@react-native-async-storage/async-storage';
import NetInfo from '@react-native-community/netinfo';

export interface AttendeeTicket {
  id: string;
  eventTitle: string;
  date: string;
  seat: string;
  txHash: string;
  status?: string;
}

export const FALLBACK_TICKETS: AttendeeTicket[] = [
  {
    id: 'T-1004',
    eventTitle: 'Stellar Meridian 2026',
    date: 'Oct 15, 2026',
    seat: 'General Admission',
    txHash: '0x3f...b82d',
    status: 'valid',
  },
];

export const TICKETS_STORAGE_KEY = 'agora.attendee.tickets.cache';
const API_BASE = process.env.EXPO_PUBLIC_API_URL || 'http://localhost:3001/api/v1';

export class NetworkError extends Error {
  constructor(message = 'Network error. Please check your connection.') {
    super(message);
    this.name = 'NetworkError';
  }
}

export async function fetchAttendeeTickets(): Promise<AttendeeTicket[]> {
  // Check network connectivity first
  const netState = await NetInfo.fetch();
  if (netState.isConnected === false || netState.isInternetReachable === false) {
    throw new NetworkError('You are currently offline. Please check your connection.');
  }

  const token = await AsyncStorage.getItem('auth_token');
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  };
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  try {
    const res = await fetch(`${API_BASE}/profile/tickets`, {
      method: 'GET',
      headers,
    });

    if (!res.ok) {
      if (res.status >= 500) {
        throw new NetworkError(`Server responded with error (${res.status})`);
      }
      throw new Error(`Failed to fetch tickets (${res.status})`);
    }

    const body = await res.json();
    const rawTickets = body?.data?.upcoming || body?.data || body?.tickets || [];

    if (Array.isArray(rawTickets) && rawTickets.length > 0) {
      const tickets: AttendeeTicket[] = rawTickets.map((t: any) => ({
        id: t.id || `T-${Math.floor(1000 + Math.random() * 9000)}`,
        eventTitle: t.event_title || t.eventTitle || 'Untitled Event',
        date: t.event_start_time
          ? new Date(t.event_start_time).toLocaleDateString('en-US', {
              month: 'short',
              day: 'numeric',
              year: 'numeric',
            })
          : t.date || 'TBD',
        seat: t.ticket_tier_name || t.seat || 'General Admission',
        txHash: t.transaction_hash || t.txHash || '0x...stellar',
        status: t.status || 'valid',
      }));

      await AsyncStorage.setItem(TICKETS_STORAGE_KEY, JSON.stringify(tickets));
      return tickets;
    }

    return rawTickets.length === 0 ? [] : FALLBACK_TICKETS;
  } catch (err: any) {
    if (err instanceof NetworkError) {
      throw err;
    }
    if (err?.name === 'TypeError' || err?.message?.includes('Network') || err?.message?.includes('fetch')) {
      throw new NetworkError('Network request failed. Please check your connection.');
    }
    throw err;
  }
}

export async function getCachedAttendeeTickets(): Promise<AttendeeTicket[] | null> {
  try {
    const cached = await AsyncStorage.getItem(TICKETS_STORAGE_KEY);
    if (cached) {
      return JSON.parse(cached);
    }
  } catch {
    // Ignore cache parse errors
  }
  return null;
}
