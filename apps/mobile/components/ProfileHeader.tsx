import React, { useState, useRef, useEffect, useCallback } from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  StyleSheet,
  Animated,
  Platform,
  StyleProp,
  ViewStyle,
} from 'react-native';
import * as Clipboard from 'expo-clipboard';
import { Ionicons } from '@expo/vector-icons';
import Colors from '@/constants/Colors';
import AvatarEdit from '@/components/AvatarEdit';

export interface ProfileHeaderProps {
  /** User's display name or username */
  username?: string | null;
  /** User's email address */
  email?: string | null;
  /** Stellar public key address (e.g. G...) */
  publicKey?: string | null;
  /** Initial avatar image URI */
  avatarUrl?: string | null;
  /** Callback when user selects a new avatar image */
  onAvatarChange?: (uri: string) => void;
  /** Whether the avatar can be tapped to pick an image */
  editableAvatar?: boolean;
  /** Optional callback fired after public key is copied */
  onCopy?: (key: string) => void;
  /** Optional custom toast message */
  toastMessage?: string;
  /** Duration in milliseconds that the toast remains visible */
  toastDuration?: number;
  /** Optional container style */
  style?: StyleProp<ViewStyle>;
  /** Optional testID */
  testID?: string;
}

/**
 * Truncates a Stellar public key (or cryptographic address) to the format: `GD47...7STN`
 */
export function truncatePublicKey(key?: string | null, startChars = 4, endChars = 4): string {
  if (!key) return '';
  const trimmed = key.trim();
  if (trimmed.length <= startChars + endChars) return trimmed;
  return `${trimmed.slice(0, startChars)}...${trimmed.slice(-endChars)}`;
}

export const ProfileHeader: React.FC<ProfileHeaderProps> = ({
  username,
  email,
  publicKey,
  avatarUrl,
  onAvatarChange,
  editableAvatar = true,
  onCopy,
  toastMessage = 'Public key copied to clipboard!',
  toastDuration = 2500,
  style,
  testID = 'profile-header',
}) => {
  const [toastVisible, setToastVisible] = useState(false);
  const [activeToastText, setActiveToastText] = useState(toastMessage);
  const fadeAnim = useRef(new Animated.Value(0)).current;
  const slideAnim = useRef(new Animated.Value(-6)).current;
  const timerRef = useRef<NodeJS.Timeout | null>(null);

  const displayName = username || 'Agora User';
  const displayEmail = email || 'user@agora.events';
  const truncatedKey = publicKey ? truncatePublicKey(publicKey) : 'No address';

  const hideToast = useCallback(() => {
    Animated.parallel([
      Animated.timing(fadeAnim, {
        toValue: 0,
        duration: 200,
        useNativeDriver: Platform.OS !== 'web',
      }),
      Animated.timing(slideAnim, {
        toValue: -6,
        duration: 200,
        useNativeDriver: Platform.OS !== 'web',
      }),
    ]).start(() => {
      setToastVisible(false);
    });
  }, [fadeAnim, slideAnim]);

  const showToast = useCallback(
    (msg: string) => {
      if (timerRef.current) {
        clearTimeout(timerRef.current);
      }
      setActiveToastText(msg);
      setToastVisible(true);

      Animated.parallel([
        Animated.timing(fadeAnim, {
          toValue: 1,
          duration: 200,
          useNativeDriver: Platform.OS !== 'web',
        }),
        Animated.timing(slideAnim, {
          toValue: 0,
          duration: 200,
          useNativeDriver: Platform.OS !== 'web',
        }),
      ]).start();

      timerRef.current = setTimeout(() => {
        hideToast();
      }, toastDuration);
    },
    [fadeAnim, slideAnim, toastDuration, hideToast]
  );

  useEffect(() => {
    return () => {
      if (timerRef.current) {
        clearTimeout(timerRef.current);
      }
    };
  }, []);

  const handleCopy = async () => {
    if (!publicKey) return;
    try {
      await Clipboard.setStringAsync(publicKey);
      showToast(toastMessage);
      onCopy?.(publicKey);
    } catch {
      showToast('Failed to copy');
    }
  };

  return (
    <View style={[styles.container, style]} testID={testID}>
      {/* 1. Avatar with placeholder support */}
      <View style={styles.avatarWrapper}>
        <AvatarEdit
          initialImage={avatarUrl}
          onImageSelected={onAvatarChange ?? (() => {})}
          userName={displayName}
        />
      </View>

      {/* 2. Username and Email */}
      <Text style={styles.nameText} numberOfLines={1} testID="profile-username">
        {displayName}
      </Text>
      <Text style={styles.emailText} numberOfLines={1} testID="profile-email">
        {displayEmail}
      </Text>

      {/* 3. Truncated Stellar Public Key & 4. Copy Button */}
      <View style={styles.keyContainer} testID="public-key-widget">
        <Text style={styles.keyText} numberOfLines={1} testID="truncated-public-key">
          {truncatedKey}
        </Text>
        <TouchableOpacity
          onPress={handleCopy}
          disabled={!publicKey}
          style={[styles.copyButton, !publicKey && styles.copyButtonDisabled]}
          accessibilityRole="button"
          accessibilityLabel="Copy public key"
          testID="copy-public-key-button"
          activeOpacity={0.7}
          hitSlop={{ top: 8, bottom: 8, left: 8, right: 8 }}
        >
          <Ionicons
            name="copy-outline"
            size={16}
            color={publicKey ? Colors.primaryYellow : Colors.secondaryText}
          />
        </TouchableOpacity>
      </View>

      {/* Toast Notification */}
      <View style={styles.toastAnchor}>
        {toastVisible && (
          <Animated.View
            style={[
              styles.toast,
              {
                opacity: fadeAnim,
                transform: [{ translateY: slideAnim }],
              },
            ]}
            accessibilityRole="alert"
            accessibilityLiveRegion="polite"
            testID="toast-message"
          >
            <Ionicons name="checkmark-circle" size={15} color="#34C759" style={styles.toastIcon} />
            <Text style={styles.toastText} numberOfLines={1}>
              {activeToastText}
            </Text>
          </Animated.View>
        )}
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    alignItems: 'center',
    width: '100%',
    paddingVertical: 12,
  },
  avatarWrapper: {
    marginBottom: 4,
  },
  nameText: {
    fontSize: 22,
    fontWeight: '700',
    color: Colors.primaryText,
    marginBottom: 4,
    textAlign: 'center',
    maxWidth: '90%',
  },
  emailText: {
    fontSize: 14,
    color: Colors.secondaryText,
    marginBottom: 12,
    textAlign: 'center',
    maxWidth: '90%',
  },
  keyContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#1E1E20',
    borderWidth: 1,
    borderColor: '#2C2C2E',
    borderRadius: 20,
    paddingVertical: 6,
    paddingHorizontal: 12,
    maxWidth: '90%',
  },
  keyText: {
    fontSize: 13,
    color: Colors.primaryYellow,
    fontFamily: 'SpaceMono',
    letterSpacing: 0.5,
    marginRight: 8,
  },
  copyButton: {
    padding: 4,
    borderRadius: 6,
    justifyContent: 'center',
    alignItems: 'center',
  },
  copyButtonDisabled: {
    opacity: 0.4,
  },
  toastAnchor: {
    minHeight: 28,
    marginTop: 8,
    justifyContent: 'center',
    alignItems: 'center',
  },
  toast: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#242426',
    borderWidth: 1,
    borderColor: '#38383A',
    borderRadius: 16,
    paddingVertical: 4,
    paddingHorizontal: 12,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.2,
    shadowRadius: 4,
    elevation: 3,
  },
  toastIcon: {
    marginRight: 6,
  },
  toastText: {
    fontSize: 12,
    fontWeight: '500',
    color: Colors.primaryText,
  },
});

export default ProfileHeader;
