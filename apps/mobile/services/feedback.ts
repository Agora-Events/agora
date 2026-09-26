import * as Haptics from 'expo-haptics';

/**
 * Centralized feedback service for haptic alerts and feedback
 * Wraps expo-haptics with fallback support for devices without haptic capability
 */

/**
 * Triggers double notification style haptic feedback (success feedback)
 * Falls back gracefully on devices without haptic support
 */
export const triggerSuccess = async (): Promise<void> => {
    try {
        await Haptics.notificationAsync(Haptics.NotificationFeedbackType.Success);
    } catch (error) {
        // Silently ignore on devices without haptic support
        console.debug('Haptic feedback unavailable:', error);
    }
};

/**
 * Triggers heavy warning style haptic feedback (error feedback)
 * Falls back gracefully on devices without haptic support
 */
export const triggerError = async (): Promise<void> => {
    try {
        await Haptics.notificationAsync(Haptics.NotificationFeedbackType.Warning);
    } catch (error) {
        // Silently ignore on devices without haptic support
        console.debug('Haptic feedback unavailable:', error);
    }
};

/**
 * Triggers light select tick feedback (selection change feedback)
 * Falls back gracefully on devices without haptic support
 */
export const triggerSelectionChange = async (): Promise<void> => {
    try {
        await Haptics.selectionAsync();
    } catch (error) {
        // Silently ignore on devices without haptic support
        console.debug('Haptic feedback unavailable:', error);
    }
};
