import React, { useEffect } from 'react';
import { Platform, StatusBar as RNStatusBar } from 'react-native';
import { StatusBar as ExpoStatusBar } from 'expo-status-bar';
import Colors from '../../constants/Colors';

/**
 * App-wide status bar styling. Keeps the status bar background matched to
 * the dark app background and its content light on both platforms, so it
 * reads consistently across every route rather than per-screen.
 */
export const AppStatusBar: React.FC = () => {
  useEffect(() => {
    if (Platform.OS === 'android') {
      RNStatusBar.setBackgroundColor(Colors.darkBackground);
      RNStatusBar.setTranslucent(false);
    }
  }, []);

  return <ExpoStatusBar style="light" backgroundColor={Colors.darkBackground} />;
};

export default AppStatusBar;
