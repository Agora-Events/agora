import React, { useEffect, useRef } from 'react';
import { View, StyleSheet, Dimensions, Text } from 'react-native';
import { Svg, Path, Circle } from 'react-native-svg';
import Animated, {
  useAnimatedStyle,
  useSharedValue,
  withTiming,
  Easing,
} from 'react-native-reanimated';
import Colors from '@/constants/Colors';

const { width: SCREEN_WIDTH } = Dimensions.get('window');
const CHART_WIDTH = SCREEN_WIDTH - 32;
const CHART_HEIGHT = 200;
const PADDING = 20;

interface DataPoint {
  timestamp: string;
  value: number;
}

interface ScanVelocityChartProps {
  data: DataPoint[];
  color?: string;
  showGrid?: boolean;
}

export default function ScanVelocityChart({
  data,
  color = Colors.primaryYellow,
  showGrid = true,
}: ScanVelocityChartProps) {
  const animatedValues = useRef(
    data.map(() => Math.random())
  ).current;

  const progress = useSharedValue(0);

  useEffect(() => {
    progress.value = withTiming(1, {
      duration: 1000,
      easing: Easing.out(Easing.cubic),
    });
  }, [data]);

  const maxValue = Math.max(...data.map((d) => d.value), 1);
  const minValue = Math.min(...data.map((d) => d.value), 0);
  const range = maxValue - minValue || 1;

  const points = data.map((point, index) => {
    const x = PADDING + (index / (data.length - 1 || 1)) * (CHART_WIDTH - 2 * PADDING);
    const y =
      CHART_HEIGHT -
      PADDING -
      ((point.value - minValue) / range) * (CHART_HEIGHT - 2 * PADDING);
    return { x, y, value: point.value };
  });

  // Create path for the line
  const pathData = points.reduce((acc, point, index) => {
    if (index === 0) {
      return `M ${point.x} ${point.y}`;
    }
    // Smooth curve using bezier
    const prev = points[index - 1];
    const cp1x = prev.x + (point.x - prev.x) / 2;
    const cp1y = prev.y;
    const cp2x = prev.x + (point.x - prev.x) / 2;
    const cp2y = point.y;
    return `${acc} C ${cp1x} ${cp1y}, ${cp2x} ${cp2y}, ${point.x} ${point.y}`;
  }, '');

  // Create area path (fill below the line)
  const areaPath = `${pathData} L ${points[points.length - 1].x} ${CHART_HEIGHT} L ${points[0].x} ${CHART_HEIGHT} Z`;

  const animatedStyle = useAnimatedStyle(() => ({
    opacity: progress.value,
    transform: [{ scaleY: progress.value }],
  }));

  return (
    <View style={styles.container}>
      <View style={[styles.chartContainer, { height: CHART_HEIGHT }]}>
        {showGrid && (
          <View style={styles.gridContainer}>
            {[0, 0.25, 0.5, 0.75, 1].map((fraction) => (
              <View
                key={fraction}
                style={[
                  styles.gridLine,
                  {
                    top: fraction * (CHART_HEIGHT - 2 * PADDING) + PADDING,
                  },
                ]}
              />
            ))}
          </View>
        )}

        {/* Area fill */}
        <Animated.View style={[styles.area, animatedStyle]}>
          <Svg width={CHART_WIDTH} height={CHART_HEIGHT} style={styles.svg}>
            <Path d={areaPath} fill={`${color}22`} stroke="none" />
          </Svg>
        </Animated.View>

        {/* Line */}
        <Animated.View style={[styles.line, animatedStyle]}>
          <Svg width={CHART_WIDTH} height={CHART_HEIGHT} style={styles.svg}>
            <Path
              d={pathData}
              fill="none"
              stroke={color}
              strokeWidth={3}
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </Svg>
        </Animated.View>

        {/* Data points */}
        {points.map((point, index) => (
          <Animated.View
            key={index}
            style={[
              styles.dataPoint,
              {
                left: point.x - 4,
                top: point.y - 4,
                backgroundColor: color,
              },
              animatedStyle,
            ]}
          />
        ))}
      </View>

      {/* X-axis labels */}
      <View style={styles.xAxis}>
        {data.map((point, index) => {
          if (index % Math.ceil(data.length / 5) === 0) {
            const time = new Date(point.timestamp).toLocaleTimeString('en-US', {
              hour: '2-digit',
              minute: '2-digit',
            });
            return (
              <View
                key={index}
                style={[
                  styles.xAxisLabel,
                  {
                    left:
                      PADDING +
                      (index / (data.length - 1 || 1)) * (CHART_WIDTH - 2 * PADDING),
                  },
                ]}
              >
                <Text style={styles.xAxisText}>
                  {time}
                </Text>
              </View>
            );
          }
          return null;
        })}
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    width: CHART_WIDTH,
    backgroundColor: '#1E1E20',
    borderRadius: 12,
    padding: 16,
    borderWidth: 1,
    borderColor: '#2C2C2E',
  },
  chartContainer: {
    position: 'relative',
    overflow: 'hidden',
  },
  gridContainer: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
  },
  gridLine: {
    position: 'absolute',
    left: PADDING,
    right: PADDING,
    height: 1,
    backgroundColor: '#2C2C2E',
  },
  svg: {
    position: 'absolute',
    top: 0,
    left: 0,
  },
  area: {
    position: 'absolute',
    top: 0,
    left: 0,
  },
  line: {
    position: 'absolute',
    top: 0,
    left: 0,
  },
  dataPoint: {
    position: 'absolute',
    width: 8,
    height: 8,
    borderRadius: 4,
    borderWidth: 2,
    borderColor: '#1E1E20',
  },
  xAxis: {
    position: 'relative',
    height: 24,
    marginTop: 8,
  },
  xAxisLabel: {
    position: 'absolute',
    transform: [{ translateX: -25 }],
  },
  xAxisText: {
    fontSize: 10,
    color: Colors.secondaryText,
    width: 50,
    textAlign: 'center',
  },
});
