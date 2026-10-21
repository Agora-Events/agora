import { useState, useEffect, useMemo, useCallback, useRef } from 'react';
import {
  SeatData,
  VenueMapData,
  ViewportBounds,
  SeatLockEvent,
  SectionData,
} from '@/components/seating/types';
import { QuadTreeIndex } from '@/components/seating/SpatialIndex';

export interface UseSeatingLayoutOptions {
  eventId: string;
  initialVenueMap?: VenueMapData;
  mockSeatCount?: number;
  onSeatLockChange?: (event: SeatLockEvent) => void;
}

export function useSeatingLayout({
  eventId,
  initialVenueMap,
  mockSeatCount = 1200,
}: UseSeatingLayoutOptions) {
  // Generate sample high-performance venue map if none provided
  const venueData = useMemo<VenueMapData>(() => {
    if (initialVenueMap) return initialVenueMap;
    return generateVenueMap(mockSeatCount);
  }, [initialVenueMap, mockSeatCount]);

  const [seats, setSeats] = useState<SeatData[]>(venueData.seats);
  const [selectedSeatIds, setSelectedSeatIds] = useState<Set<string>>(new Set());
  const [focusedSeatIndex, setFocusedSeatIndex] = useState<number>(-1);

  // Initialize QuadTree spatial index for frustum culling
  const spatialIndex = useMemo(() => {
    const bounds: ViewportBounds = {
      minX: 0,
      minY: 0,
      maxX: venueData.width,
      maxY: venueData.height,
    };
    const index = new QuadTreeIndex(bounds, 32);
    index.build(seats);
    return index;
  }, [venueData, seats]);

  // Real-time lock countdown timer decrement
  useEffect(() => {
    const timer = setInterval(() => {
      setSeats((prevSeats) => {
        let changed = false;
        const updated = prevSeats.map((seat) => {
          if (seat.status === 'locked' && seat.lockTimer !== undefined) {
            changed = true;
            if (seat.lockTimer <= 1) {
              return { ...seat, status: 'available' as const, lockTimer: undefined };
            }
            return { ...seat, lockTimer: seat.lockTimer - 1 };
          }
          return seat;
        });
        return changed ? updated : prevSeats;
      });
    }, 1000);

    return () => clearInterval(timer);
  }, []);

  // WebSocket Live Seat Locking Listener / Simulated room updates
  const handleRemoteSeatLock = useCallback((event: SeatLockEvent) => {
    setSeats((prevSeats) =>
      prevSeats.map((seat) => {
        if (seat.id === event.seatId) {
          if (event.status === 'locked') {
            return {
              ...seat,
              status: 'locked' as const,
              lockTimer: event.durationSeconds || 120,
              lockedBy: event.userId,
            };
          } else if (event.status === 'unlocked') {
            return {
              ...seat,
              status: 'available' as const,
              lockTimer: undefined,
              lockedBy: undefined,
            };
          } else if (event.status === 'purchased') {
            return {
              ...seat,
              status: 'reserved' as const,
              lockTimer: undefined,
            };
          }
        }
        return seat;
      })
    );
  }, []);

  // Toggle seat selection
  const toggleSeatSelection = useCallback(
    (seat: SeatData) => {
      if (seat.status === 'reserved' || seat.status === 'locked') {
        return;
      }

      setSelectedSeatIds((prevSelected) => {
        const next = new Set(prevSelected);
        const isNowSelected = !next.has(seat.id);

        if (isNowSelected) {
          next.add(seat.id);
        } else {
          next.delete(seat.id);
        }

        // Update local seat status
        setSeats((prevSeats) =>
          prevSeats.map((s) => {
            if (s.id === seat.id) {
              return {
                ...s,
                status: isNowSelected ? ('selected' as const) : s.tier === 'vip' ? ('vip' as const) : ('available' as const),
              };
            }
            return s;
          })
        );

        return next;
      });
    },
    []
  );

  // Accessibility focus navigation traversal
  const focusNextSeat = useCallback(() => {
    setFocusedSeatIndex((prev) => {
      const nextIndex = prev + 1 < seats.length ? prev + 1 : 0;
      return nextIndex;
    });
  }, [seats.length]);

  const focusPrevSeat = useCallback(() => {
    setFocusedSeatIndex((prev) => {
      const prevIndex = prev - 1 >= 0 ? prev - 1 : seats.length - 1;
      return prevIndex;
    });
  }, [seats.length]);

  const selectFocusedSeat = useCallback(() => {
    if (focusedSeatIndex >= 0 && focusedSeatIndex < seats.length) {
      toggleSeatSelection(seats[focusedSeatIndex]);
    }
  }, [focusedSeatIndex, seats, toggleSeatSelection]);

  // Selected seats list & totals
  const selectedSeats = useMemo(() => {
    return seats.filter((s) => selectedSeatIds.has(s.id));
  }, [seats, selectedSeatIds]);

  const totalPrice = useMemo(() => {
    return selectedSeats.reduce((acc, s) => acc + s.price, 0);
  }, [selectedSeats]);

  // Query culled visible seats for current viewport
  const getVisibleSeats = useCallback(
    (viewport: ViewportBounds): SeatData[] => {
      return spatialIndex.queryVisibleSeats(viewport);
    },
    [spatialIndex]
  );

  return {
    venueData,
    seats,
    selectedSeats,
    totalPrice,
    selectedSeatIds,
    focusedSeatIndex,
    focusedSeat: focusedSeatIndex >= 0 ? seats[focusedSeatIndex] : null,
    toggleSeatSelection,
    handleRemoteSeatLock,
    focusNextSeat,
    focusPrevSeat,
    selectFocusedSeat,
    getVisibleSeats,
  };
}

/**
 * Procedurally generates a realistic stadium / theater venue map with thousands of seats
 */
function generateVenueMap(seatCount: number): VenueMapData {
  const width = 1200;
  const height = 900;
  const stage = {
    x: 400,
    y: 60,
    width: 400,
    height: 70,
    label: 'MAIN STAGE',
  };

  const sections: SectionData[] = [
    {
      id: 'sec-vip',
      name: 'VIP Floor (A)',
      bounds: { minX: 350, minY: 160, maxX: 850, maxY: 300 },
      color: '#F59E0B',
    },
    {
      id: 'sec-lower-left',
      name: 'Lower Tier Left (B)',
      bounds: { minX: 100, minY: 320, maxX: 550, maxY: 560 },
      color: '#3B82F6',
    },
    {
      id: 'sec-lower-right',
      name: 'Lower Tier Right (C)',
      bounds: { minX: 650, minY: 320, maxX: 1100, maxY: 560 },
      color: '#3B82F6',
    },
    {
      id: 'sec-balcony',
      name: 'Upper Balcony (D)',
      bounds: { minX: 180, minY: 600, maxX: 1020, maxY: 840 },
      color: '#8B5CF6',
    },
  ];

  const seats: SeatData[] = [];
  const rows = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M'];

  sections.forEach((sec) => {
    const isVipSection = sec.id === 'sec-vip';
    const numRows = isVipSection ? 6 : 8;
    const seatsPerRow = Math.floor((sec.bounds.maxX - sec.bounds.minX) / 26);

    for (let r = 0; r < numRows; r++) {
      const rowName = rows[r % rows.length];
      const y = sec.bounds.minY + 20 + r * 22;

      for (let s = 1; s <= seatsPerRow; s++) {
        if (seats.length >= seatCount) break;

        const x = sec.bounds.minX + 15 + (s - 1) * 24;
        const seatId = `${sec.id}-${rowName}-${s}`;

        // Some sample distribution of states
        let status: SeatData['status'] = 'available';
        let lockTimer: number | undefined;

        const rand = (s * 13 + r * 7) % 100;
        if (rand < 15) {
          status = 'reserved';
        } else if (rand === 22) {
          status = 'locked';
          lockTimer = 95;
        } else if (isVipSection) {
          status = 'vip';
        }

        seats.push({
          id: seatId,
          sectionId: sec.id,
          sectionName: sec.name,
          row: rowName,
          seatNumber: s,
          x,
          y,
          radius: 7,
          tier: isVipSection ? 'vip' : 'standard',
          price: isVipSection ? 150 : sec.id === 'sec-balcony' ? 45 : 75,
          status,
          lockTimer,
        });
      }
    }
  });

  return {
    width,
    height,
    stage,
    sections,
    seats,
  };
}

export default useSeatingLayout;
