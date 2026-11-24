import React, { useState, useMemo, useCallback } from 'react';
import {
  StyleSheet,
  View,
  Text,
  TouchableOpacity,
  Dimensions,
  LayoutChangeEvent,
} from 'react-native';
import { GestureDetector, Gesture } from 'react-native-gesture-handler';
import Animated, {
  useSharedValue,
  useAnimatedStyle,
  withSpring,
  runOnJS,
} from 'react-native-reanimated';
import Svg, {
  G,
  Rect,
  Text as SvgText,
  Defs,
  LinearGradient,
  Stop,
} from 'react-native-svg';
import { Ionicons } from '@expo/vector-icons';
import { useThemeContext } from '@/context/ThemeContext';
import { Colors } from '@/constants/Colors';
import { SeatData, ViewportBounds, VenueMapData } from './types';
import { SeatNode } from './SeatNode';
import { useSeatingLayout } from '@/hooks/useSeatingLayout';

export interface SeatingCanvasProps {
  eventId: string;
  venueMap?: VenueMapData;
  onProceedToCheckout?: (selectedSeats: SeatData[], totalPrice: number) => void;
}

const MIN_SCALE = 0.5;
const MAX_SCALE = 8.0;

export function SeatingCanvas({
  eventId,
  venueMap,
  onProceedToCheckout,
}: SeatingCanvasProps) {
  const { colorScheme } = useThemeContext();
  const isDark = colorScheme === 'dark';

  const [canvasDimensions, setCanvasDimensions] = useState({
    width: Dimensions.get('window').width,
    height: 520,
  });

  const {
    venueData,
    selectedSeats,
    totalPrice,
    focusedSeat,
    toggleSeatSelection,
    focusNextSeat,
    focusPrevSeat,
    selectFocusedSeat,
    getVisibleSeats,
  } = useSeatingLayout({ eventId, initialVenueMap: venueMap });

  // Reanimated transformation values
  const scale = useSharedValue(0.8);
  const savedScale = useSharedValue(0.8);
  const translateX = useSharedValue(20);
  const savedTranslateX = useSharedValue(20);
  const translateY = useSharedValue(30);
  const savedTranslateY = useSharedValue(30);

  // JS state for driving frustum culling query
  const [viewportState, setViewportState] = useState({
    scale: 0.8,
    translateX: 20,
    translateY: 30,
  });

  const updateViewportOnJS = useCallback((s: number, tx: number, ty: number) => {
    setViewportState({ scale: s, translateX: tx, translateY: ty });
  }, []);

  // Gestures: Pinch to zoom & 2D Pan with decay
  const pinchGesture = Gesture.Pinch()
    .onUpdate((e) => {
      const newScale = Math.min(
        Math.max(savedScale.value * e.scale, MIN_SCALE),
        MAX_SCALE
      );
      scale.value = newScale;
    })
    .onEnd(() => {
      savedScale.value = scale.value;
      runOnJS(updateViewportOnJS)(scale.value, translateX.value, translateY.value);
    });

  const panGesture = Gesture.Pan()
    .onUpdate((e) => {
      translateX.value = savedTranslateX.value + e.translationX;
      translateY.value = savedTranslateY.value + e.translationY;
    })
    .onEnd(() => {
      savedTranslateX.value = translateX.value;
      savedTranslateY.value = translateY.value;
      runOnJS(updateViewportOnJS)(scale.value, translateX.value, translateY.value);
    });

  const composedGesture = Gesture.Simultaneous(panGesture, pinchGesture);

  const animatedCanvasStyle = useAnimatedStyle(() => {
    return {
      transform: [
        { translateX: translateX.value },
        { translateY: translateY.value },
        { scale: scale.value },
      ],
    };
  });

  // Calculate visible frustum bounds in venue coordinate space
  const visibleFrustum: ViewportBounds = useMemo(() => {
    const s = viewportState.scale;
    const minX = -viewportState.translateX / s;
    const minY = -viewportState.translateY / s;
    const maxX = minX + canvasDimensions.width / s;
    const maxY = minY + canvasDimensions.height / s;

    return { minX, minY, maxX, maxY };
  }, [viewportState, canvasDimensions]);

  // Query culled visible seats from QuadTree spatial index
  const visibleSeats = useMemo(() => {
    return getVisibleSeats(visibleFrustum);
  }, [getVisibleSeats, visibleFrustum]);

  const handleLayout = (e: LayoutChangeEvent) => {
    const { width, height } = e.nativeEvent.layout;
    if (width > 0 && height > 0) {
      setCanvasDimensions({ width, height });
    }
  };

  const handleResetView = () => {
    scale.value = withSpring(0.8);
    savedScale.value = 0.8;
    translateX.value = withSpring(20);
    savedTranslateX.value = 20;
    translateY.value = withSpring(30);
    savedTranslateY.value = 30;
    setViewportState({ scale: 0.8, translateX: 20, translateY: 30 });
  };

  const handleZoom = (factor: number) => {
    const newScale = Math.min(Math.max(scale.value * factor, MIN_SCALE), MAX_SCALE);
    scale.value = withSpring(newScale);
    savedScale.value = newScale;
    setViewportState((prev) => ({ ...prev, scale: newScale }));
  };

  const bg = Colors[colorScheme].background;
  const cardBg = Colors[colorScheme].cardBackground;
  const textColor = isDark ? '#FFFFFF' : '#111827';
  const subtextColor = isDark ? '#9CA3AF' : '#6B7280';
  const borderColor = Colors[colorScheme].border;

  return (
    <View style={[styles.container, { backgroundColor: bg }]}>
      {/* Legend & Stats Bar */}
      <View style={[styles.legendBar, { backgroundColor: cardBg, borderColor }]}>
        <View style={styles.legendItem}>
          <View style={[styles.legendDot, { backgroundColor: '#10B981' }]} />
          <Text style={[styles.legendText, { color: subtextColor }]}>Available</Text>
        </View>
        <View style={styles.legendItem}>
          <View style={[styles.legendDot, { backgroundColor: '#F59E0B' }]} />
          <Text style={[styles.legendText, { color: subtextColor }]}>VIP</Text>
        </View>
        <View style={styles.legendItem}>
          <View style={[styles.legendDot, { backgroundColor: '#2563EB' }]} />
          <Text style={[styles.legendText, { color: subtextColor }]}>Selected</Text>
        </View>
        <View style={styles.legendItem}>
          <View style={[styles.legendDot, { backgroundColor: '#4B5563' }]} />
          <Text style={[styles.legendText, { color: subtextColor }]}>Reserved</Text>
        </View>
        <View style={styles.legendItem}>
          <View style={[styles.legendDot, { backgroundColor: '#EF4444' }]} />
          <Text style={[styles.legendText, { color: subtextColor }]}>Locked</Text>
        </View>
      </View>

      {/* Interactive Spatial Canvas */}
      <View style={styles.canvasContainer} onLayout={handleLayout}>
        <GestureDetector gesture={composedGesture}>
          <Animated.View style={[styles.svgWrapper, animatedCanvasStyle]}>
            <Svg
              width={venueData.width}
              height={venueData.height}
              viewBox={`0 0 ${venueData.width} ${venueData.height}`}
            >
              <Defs>
                <LinearGradient id="vipGoldGradient" x1="0%" y1="0%" x2="100%" y2="100%">
                  <Stop offset="0%" stopColor="#FBBF24" />
                  <Stop offset="100%" stopColor="#D97706" />
                </LinearGradient>
                <LinearGradient id="stageGradient" x1="0%" y1="0%" x2="0%" y2="100%">
                  <Stop offset="0%" stopColor="#374151" />
                  <Stop offset="100%" stopColor="#1F2937" />
                </LinearGradient>
              </Defs>

              {/* Stage Element */}
              <G>
                <Rect
                  x={venueData.stage.x}
                  y={venueData.stage.y}
                  width={venueData.stage.width}
                  height={venueData.stage.height}
                  rx={10}
                  fill="url(#stageGradient)"
                  stroke="#4B5563"
                  strokeWidth={1.5}
                />
                <SvgText
                  x={venueData.stage.x + venueData.stage.width / 2}
                  y={venueData.stage.y + venueData.stage.height / 2}
                  fontSize={14}
                  fontWeight="bold"
                  fill="#FFFFFF"
                  textAnchor="middle"
                  alignmentBaseline="central"
                >
                  {venueData.stage.label}
                </SvgText>
              </G>

              {/* Section Outlines & Labels */}
              {venueData.sections.map((sec) => (
                <G key={sec.id}>
                  <Rect
                    x={sec.bounds.minX}
                    y={sec.bounds.minY}
                    width={sec.bounds.maxX - sec.bounds.minX}
                    height={sec.bounds.maxY - sec.bounds.minY}
                    rx={12}
                    fill="none"
                    stroke={isDark ? '#374151' : '#E5E7EB'}
                    strokeWidth={1}
                    strokeDasharray="4, 4"
                  />
                  <SvgText
                    x={(sec.bounds.minX + sec.bounds.maxX) / 2}
                    y={sec.bounds.minY + 12}
                    fontSize={11}
                    fontWeight="600"
                    fill={isDark ? '#9CA3AF' : '#6B7280'}
                    textAnchor="middle"
                  >
                    {sec.name}
                  </SvgText>
                </G>
              ))}

              {/* Culled Visible Seats */}
              {visibleSeats.map((seat) => (
                <SeatNode
                  key={seat.id}
                  seat={seat}
                  scale={viewportState.scale}
                  onPress={toggleSeatSelection}
                  isFocused={focusedSeat?.id === seat.id}
                />
              ))}
            </Svg>
          </Animated.View>
        </GestureDetector>

        {/* Floating Zoom & Reset Controls */}
        <View style={styles.floatingControls}>
          <TouchableOpacity
            style={[styles.controlBtn, { backgroundColor: cardBg, borderColor }]}
            onPress={() => handleZoom(1.3)}
            accessibilityRole="button"
            accessibilityLabel="Zoom in"
          >
            <Ionicons name="add" size={18} color={textColor} />
          </TouchableOpacity>
          <TouchableOpacity
            style={[styles.controlBtn, { backgroundColor: cardBg, borderColor }]}
            onPress={() => handleZoom(0.7)}
            accessibilityRole="button"
            accessibilityLabel="Zoom out"
          >
            <Ionicons name="remove" size={18} color={textColor} />
          </TouchableOpacity>
          <TouchableOpacity
            style={[styles.controlBtn, { backgroundColor: cardBg, borderColor }]}
            onPress={handleResetView}
            accessibilityRole="button"
            accessibilityLabel="Reset map view"
          >
            <Ionicons name="scan-outline" size={16} color={textColor} />
          </TouchableOpacity>
        </View>

        {/* Viewport Info Pill */}
        <View style={styles.statsPill}>
          <Text style={styles.statsText}>
            {`Rendered: ${visibleSeats.length}/${venueData.seats.length} seats`}
          </Text>
        </View>
      </View>

      {/* Screen-Reader Accessibility Grid Navigation Bar */}
      <View style={[styles.a11yBar, { backgroundColor: cardBg, borderColor }]}>
        <View style={styles.a11yInfo}>
          <Text style={[styles.a11yTitle, { color: textColor }]}>
            {focusedSeat
              ? `${focusedSeat.sectionName} • Row ${focusedSeat.row} • #${focusedSeat.seatNumber}`
              : 'Keyboard / Screen-reader Navigation'}
          </Text>
          <Text style={[styles.a11ySub, { color: subtextColor }]}>
            {focusedSeat
              ? `Status: ${focusedSeat.status} • $${focusedSeat.price}`
              : 'Traverse grid across sections and rows'}
          </Text>
        </View>
        <View style={styles.a11yActions}>
          <TouchableOpacity
            style={[styles.a11yBtn, { borderColor }]}
            onPress={focusPrevSeat}
            accessibilityRole="button"
            accessibilityLabel="Previous seat"
          >
            <Ionicons name="chevron-back" size={18} color={textColor} />
          </TouchableOpacity>
          <TouchableOpacity
            style={[styles.a11yBtn, styles.a11ySelectBtn]}
            onPress={selectFocusedSeat}
            accessibilityRole="button"
            accessibilityLabel="Toggle selected seat"
          >
            <Text style={styles.a11ySelectText}>Toggle</Text>
          </TouchableOpacity>
          <TouchableOpacity
            style={[styles.a11yBtn, { borderColor }]}
            onPress={focusNextSeat}
            accessibilityRole="button"
            accessibilityLabel="Next seat"
          >
            <Ionicons name="chevron-forward" size={18} color={textColor} />
          </TouchableOpacity>
        </View>
      </View>

      {/* Bottom Selection & Checkout Drawer */}
      {selectedSeats.length > 0 && (
        <View style={[styles.checkoutDrawer, { backgroundColor: cardBg, borderColor }]}>
          <View style={styles.checkoutInfo}>
            <Text style={[styles.checkoutCount, { color: textColor }]}>
              {`${selectedSeats.length} ${selectedSeats.length === 1 ? 'Seat' : 'Seats'} Selected`}
            </Text>
            <Text style={[styles.checkoutTotal, { color: Colors.primaryYellow }]}>
              {`Total: $${totalPrice.toFixed(2)}`}
            </Text>
          </View>
          <TouchableOpacity
            style={styles.proceedBtn}
            onPress={() => onProceedToCheckout?.(selectedSeats, totalPrice)}
            accessibilityRole="button"
            accessibilityLabel={`Proceed to checkout for ${selectedSeats.length} seats totaling $${totalPrice.toFixed(2)}`}
          >
            <Text style={styles.proceedBtnText}>Checkout</Text>
            <Ionicons name="arrow-forward" size={16} color="#000000" />
          </TouchableOpacity>
        </View>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  legendBar: {
    flexDirection: 'row',
    justifyContent: 'space-around',
    paddingVertical: 10,
    paddingHorizontal: 8,
    borderBottomWidth: 1,
  },
  legendItem: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
  },
  legendDot: {
    width: 8,
    height: 8,
    borderRadius: 4,
  },
  legendText: {
    fontSize: 11,
    fontWeight: '500',
  },
  canvasContainer: {
    flex: 1,
    overflow: 'hidden',
    position: 'relative',
  },
  svgWrapper: {
    width: '100%',
    height: '100%',
  },
  floatingControls: {
    position: 'absolute',
    right: 16,
    top: 16,
    gap: 8,
  },
  controlBtn: {
    width: 38,
    height: 38,
    borderRadius: 19,
    borderWidth: 1,
    alignItems: 'center',
    justifyContent: 'center',
    shadowColor: '#000',
    shadowOpacity: 0.15,
    shadowOffset: { width: 0, height: 2 },
    shadowRadius: 4,
    elevation: 4,
  },
  statsPill: {
    position: 'absolute',
    left: 14,
    top: 16,
    backgroundColor: 'rgba(0,0,0,0.6)',
    paddingVertical: 4,
    paddingHorizontal: 8,
    borderRadius: 12,
  },
  statsText: {
    color: '#E5E7EB',
    fontSize: 10,
    fontWeight: '600',
  },
  a11yBar: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingHorizontal: 16,
    paddingVertical: 10,
    borderTopWidth: 1,
  },
  a11yInfo: {
    flex: 1,
  },
  a11yTitle: {
    fontSize: 12,
    fontWeight: '700',
  },
  a11ySub: {
    fontSize: 11,
  },
  a11yActions: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
  },
  a11yBtn: {
    padding: 6,
    borderRadius: 8,
    borderWidth: 1,
    alignItems: 'center',
    justifyContent: 'center',
  },
  a11ySelectBtn: {
    backgroundColor: Colors.primaryYellow,
    borderWidth: 0,
    paddingHorizontal: 10,
    paddingVertical: 6,
  },
  a11ySelectText: {
    color: '#000000',
    fontSize: 11,
    fontWeight: '700',
  },
  checkoutDrawer: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingHorizontal: 20,
    paddingVertical: 14,
    borderTopWidth: 1,
    elevation: 8,
  },
  checkoutInfo: {
    flex: 1,
  },
  checkoutCount: {
    fontSize: 13,
    fontWeight: '600',
  },
  checkoutTotal: {
    fontSize: 17,
    fontWeight: '800',
  },
  proceedBtn: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
    backgroundColor: Colors.primaryYellow,
    paddingVertical: 10,
    paddingHorizontal: 18,
    borderRadius: 12,
  },
  proceedBtnText: {
    color: '#000000',
    fontSize: 14,
    fontWeight: '700',
  },
});

export default SeatingCanvas;
