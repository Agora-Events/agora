import React, { useCallback, useEffect, useRef, useState } from 'react';
import {
  LayoutChangeEvent,
  Modal,
  NativeScrollEvent,
  NativeSyntheticEvent,
  Pressable,
  ScrollView,
  Text,
  View,
} from 'react-native';
import Button from '@/components/ui/Button';
import styles from './TermsAgreementModal.styles';

const END_TOLERANCE = 1;

interface TermsAgreementModalProps {
  visible: boolean;
  termsText: string;
  onAgree: () => void;
  onRequestClose: () => void;
}

export default function TermsAgreementModal({
  visible,
  termsText,
  onAgree,
  onRequestClose,
}: TermsAgreementModalProps) {
  const [hasReachedEnd, setHasReachedEnd] = useState(false);
  const scrollOffset = useRef(0);
  const viewportHeight = useRef(0);
  const contentHeight = useRef(0);

  const updateEndState = useCallback(() => {
    if (
      viewportHeight.current > 0 &&
      contentHeight.current > 0 &&
      scrollOffset.current + viewportHeight.current >= contentHeight.current - END_TOLERANCE
    ) {
      setHasReachedEnd(true);
    }
  }, []);

  useEffect(() => {
    if (!visible) {
      setHasReachedEnd(false);
      scrollOffset.current = 0;
      viewportHeight.current = 0;
      contentHeight.current = 0;
    }
  }, [visible]);

  const handleScroll = ({ nativeEvent }: NativeSyntheticEvent<NativeScrollEvent>) => {
    scrollOffset.current = nativeEvent.contentOffset.y;
    viewportHeight.current = nativeEvent.layoutMeasurement.height;
    contentHeight.current = nativeEvent.contentSize.height;
    updateEndState();
  };

  const handleLayout = ({ nativeEvent }: LayoutChangeEvent) => {
    viewportHeight.current = nativeEvent.layout.height;
    updateEndState();
  };

  const handleContentSizeChange = (_width: number, height: number) => {
    contentHeight.current = height;
    updateEndState();
  };

  return (
    <Modal
      visible={visible}
      transparent
      animationType="fade"
      statusBarTranslucent
      onRequestClose={onRequestClose}
    >
      <View style={styles.overlay} accessibilityViewIsModal>
        <View style={styles.card}>
          <View style={styles.header}>
            <Text style={styles.title}>Terms of Service</Text>
            <Pressable
              accessibilityRole="button"
              accessibilityLabel="Close terms of service"
              hitSlop={12}
              onPress={onRequestClose}
            >
              <Text style={styles.close}>Close</Text>
            </Pressable>
          </View>

          <ScrollView
            testID="terms-agreement-scroll"
            style={styles.scrollView}
            contentContainerStyle={styles.scrollContent}
            onScroll={handleScroll}
            onLayout={handleLayout}
            onContentSizeChange={handleContentSizeChange}
            scrollEventThrottle={16}
          >
            <Text style={styles.terms}>{termsText}</Text>
          </ScrollView>

          <Text style={styles.hint}>
            {hasReachedEnd
              ? 'You have reached the end of the terms.'
              : 'Scroll to the bottom to continue.'}
          </Text>
          <Button
            testID="terms-agreement-confirm"
            title="Agree and Proceed"
            onPress={onAgree}
            disabled={!hasReachedEnd}
          />
        </View>
      </View>
    </Modal>
  );
}
