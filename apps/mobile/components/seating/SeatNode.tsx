import React, { memo } from 'react';
import { G, Circle, Rect, Text as SvgText, Defs, LinearGradient, Stop } from 'react-native-svg';
import { SeatData } from './types';

export interface SeatNodeProps {
  seat: SeatData;
  scale: number;
  onPress: (seat: SeatData) => void;
  isFocused?: boolean;
}

const STATUS_COLORS = {
  available: '#10B981', // Accent emerald
  reserved: '#4B5563',  // Muted gray
  selected: '#2563EB',  // Vibrant blue
  vip: '#F59E0B',       // Gold
  locked: '#EF4444',     // Red/orange locked
};

export const SeatNode = memo(({ seat, scale, onPress, isFocused = false }: SeatNodeProps) => {
  const { x, y, radius, status, seatNumber, tier, lockTimer } = seat;

  const isClickable = status === 'available' || status === 'selected' || status === 'vip';
  const color = STATUS_COLORS[status] || STATUS_COLORS.available;
  const isSelected = status === 'selected';
  const isVip = tier === 'vip' || status === 'vip';
  const isLocked = status === 'locked';

  // Show seat number if zoomed in sufficiently
  const showText = scale >= 1.6;

  return (
    <G
      x={x}
      y={y}
      onPress={() => {
        if (isClickable) {
          onPress(seat);
        }
      }}
      accessible={true}
      accessibilityRole="button"
      accessibilityLabel={`Section ${seat.sectionName}, Row ${seat.row}, Seat ${seatNumber}. Status: ${status}. Price: $${seat.price}`}
      accessibilityHint={isClickable ? 'Double tap to toggle seat selection' : 'Seat is not available'}
      accessibilityState={{
        selected: isSelected,
        disabled: !isClickable,
      }}
    >
      {/* Outer focus / selection ring */}
      {(isSelected || isFocused) && (
        <Circle
          r={radius + 3.5}
          fill="none"
          stroke={isSelected ? '#60A5FA' : '#FDDA23'}
          strokeWidth={2}
          strokeDasharray={isFocused ? '3, 2' : undefined}
          opacity={0.9}
        />
      )}

      {/* Main Seat Body */}
      {isVip ? (
        <Circle
          r={radius}
          fill="url(#vipGoldGradient)"
          stroke="#B45309"
          strokeWidth={0.8}
        />
      ) : (
        <Circle
          r={radius}
          fill={color}
          stroke={isSelected ? '#1D4ED8' : '#1F2937'}
          strokeWidth={0.6}
        />
      )}

      {/* Locked Timer / Badge */}
      {isLocked && lockTimer !== undefined && lockTimer > 0 && (
        <G y={-radius - 5}>
          <Rect
            x={-10}
            y={-7}
            width={20}
            height={10}
            rx={3}
            fill="#B91C1C"
          />
          <SvgText
            fontSize={7}
            fill="#FFFFFF"
            fontWeight="bold"
            textAnchor="middle"
            alignmentBaseline="middle"
          >
            {`${lockTimer}s`}
          </SvgText>
        </G>
      )}

      {/* Seat Number when zoomed in */}
      {showText && !isLocked && (
        <SvgText
          fontSize={radius * 0.9}
          fill={isSelected || status === 'reserved' ? '#FFFFFF' : '#000000'}
          fontWeight="600"
          textAnchor="middle"
          alignmentBaseline="central"
        >
          {seatNumber}
        </SvgText>
      )}
    </G>
  );
});

export default SeatNode;
