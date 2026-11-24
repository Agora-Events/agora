/**
 * Types for Interactive Seating Chart Engine & Spatial Renderer
 */

export type SeatStatus = 'available' | 'reserved' | 'selected' | 'vip' | 'locked';

export type SeatTier = 'standard' | 'vip' | 'accessible';

export interface SeatData {
  id: string;
  sectionId: string;
  sectionName: string;
  row: string;
  seatNumber: number;
  x: number;
  y: number;
  radius: number;
  tier: SeatTier;
  price: number;
  status: SeatStatus;
  lockTimer?: number; // seconds remaining on lock
  lockedBy?: string;
}

export interface SectionBounds {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
}

export interface SectionData {
  id: string;
  name: string;
  bounds: SectionBounds;
  color?: string;
}

export interface StageData {
  x: number;
  y: number;
  width: number;
  height: number;
  label: string;
}

export interface VenueMapData {
  width: number;
  height: number;
  stage: StageData;
  sections: SectionData[];
  seats: SeatData[];
}

export interface ViewportBounds {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
}

export interface SeatLockEvent {
  seatId: string;
  status: 'locked' | 'unlocked' | 'purchased';
  userId?: string;
  durationSeconds?: number;
}
