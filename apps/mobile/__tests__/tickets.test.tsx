import React from 'react';
import { render, fireEvent, waitFor, act } from '@testing-library/react-native';
import NetInfo from '@react-native-community/netinfo';
import TicketsScreen from '../app/(tabs)/tickets';
import * as ticketService from '../services/ticketService';

// Mock NetInfo
jest.mock('@react-native-community/netinfo', () => {
  let listener: ((state: { isConnected: boolean; isInternetReachable: boolean }) => void) | null = null;
  return {
    fetch: jest.fn().mockResolvedValue({ isConnected: true, isInternetReachable: true }),
    addEventListener: jest.fn((callback) => {
      listener = callback;
      return () => {
        listener = null;
      };
    }),
    __triggerNetworkChange: (state: { isConnected: boolean; isInternetReachable: boolean }) => {
      if (listener) {
        listener(state);
      }
    },
  };
});

// Mock ticketService
jest.mock('../services/ticketService', () => {
  const actual = jest.requireActual('../services/ticketService');
  return {
    ...actual,
    fetchAttendeeTickets: jest.fn(),
    getCachedAttendeeTickets: jest.fn(),
  };
});

const MOCK_TICKETS: ticketService.AttendeeTicket[] = [
  {
    id: 'T-2001',
    eventTitle: 'Decentralized Identity Summit',
    date: 'Nov 12, 2026',
    seat: 'VIP Pass',
    txHash: '0xabc...1234',
  },
  {
    id: 'T-2002',
    eventTitle: 'Rust in Production',
    date: 'Dec 05, 2026',
    seat: 'General Admission',
    txHash: '0xdef...5678',
  },
];

describe('TicketsScreen (Attendee Tickets Tab)', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    (ticketService.fetchAttendeeTickets as jest.Mock).mockResolvedValue(MOCK_TICKETS);
    (ticketService.getCachedAttendeeTickets as jest.Mock).mockResolvedValue(null);
  });

  it('renders tickets list on successful load', async () => {
    const { getByText, findByText, getAllByTestId } = render(<TicketsScreen />);

    expect(getByText('My Tickets')).toBeTruthy();
    expect(await findByText('Decentralized Identity Summit')).toBeTruthy();
    expect(getByText('Rust in Production')).toBeTruthy();

    const ticketCards = getAllByTestId('ticket-card');
    expect(ticketCards.length).toBe(2);
  });

  it('triggers pull-to-refresh and updates tickets', async () => {
    const updatedTickets: ticketService.AttendeeTicket[] = [
      {
        id: 'T-3001',
        eventTitle: 'Updated Stellar Meridian',
        date: 'Oct 20, 2026',
        seat: 'General Admission',
        txHash: '0x999...8888',
      },
    ];

    const { getByTestId, findByText } = render(<TicketsScreen />);
    await findByText('Decentralized Identity Summit');

    (ticketService.fetchAttendeeTickets as jest.Mock).mockResolvedValueOnce(updatedTickets);

    const scrollView = getByTestId('tickets-scroll-view');
    const { refreshControl } = scrollView.props;

    await act(async () => {
      await refreshControl.props.onRefresh();
    });

    expect(ticketService.fetchAttendeeTickets).toHaveBeenCalledTimes(2);
    expect(await findByText('Updated Stellar Meridian')).toBeTruthy();
  });

  it('displays network error retry banner when fetch fails with network error', async () => {
    (ticketService.fetchAttendeeTickets as jest.Mock).mockRejectedValueOnce(
      new ticketService.NetworkError('Network error. Unable to load tickets.')
    );

    const { findByTestId, getByText } = render(<TicketsScreen />);

    const banner = await findByTestId('network-error-banner');
    expect(banner).toBeTruthy();
    expect(getByText('Network error. Unable to load tickets.')).toBeTruthy();
    expect(getByText('Retry')).toBeTruthy();
  });

  it('retries fetching tickets when retry button is pressed and clears the banner', async () => {
    (ticketService.fetchAttendeeTickets as jest.Mock).mockRejectedValueOnce(
      new ticketService.NetworkError('Network error. Unable to load tickets.')
    );

    const { findByTestId, queryByTestId, findByText } = render(<TicketsScreen />);

    const retryButton = await findByTestId('retry-button');
    expect(retryButton).toBeTruthy();

    // Now mock subsequent retry success
    (ticketService.fetchAttendeeTickets as jest.Mock).mockResolvedValueOnce(MOCK_TICKETS);

    await act(async () => {
      fireEvent.press(retryButton);
    });

    await waitFor(() => {
      expect(queryByTestId('network-error-banner')).toBeNull();
    });

    expect(await findByText('Decentralized Identity Summit')).toBeTruthy();
  });

  it('shows network error banner when device transitions to offline', async () => {
    const { findByText, queryByTestId, findByTestId } = render(<TicketsScreen />);
    await findByText('Decentralized Identity Summit');

    expect(queryByTestId('network-error-banner')).toBeNull();

    // Simulate going offline
    const netinfoMock = NetInfo as unknown as {
      __triggerNetworkChange: (state: { isConnected: boolean; isInternetReachable: boolean }) => void;
    };

    act(() => {
      netinfoMock.__triggerNetworkChange({
        isConnected: false,
        isInternetReachable: false,
      });
    });

    const banner = await findByTestId('network-error-banner');
    expect(banner).toBeTruthy();
    expect(await findByText('You are currently offline. Check your network connection.')).toBeTruthy();
  });
});
