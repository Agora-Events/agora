import React from 'react';
import {
  View,
  Text,
  StyleSheet,
  SafeAreaView,
  ScrollView,
} from 'react-native';
import { Button } from '../components/ui/Button';
import Colors from '../constants/Colors';
import { useAuth } from '../hooks/useAuth';

export interface SettingsScreenProps {
  onLogout?: () => void;
}

/**
 * SettingsScreen provides user configuration and session termination.
 * The Logout button uses the destructive (red) variant to prevent accidental taps
 * and clearly indicate a state-changing action.
 */
export const SettingsScreen: React.FC<SettingsScreenProps> = ({ onLogout }) => {
  const { user, logout } = useAuth();
  const handleLogout = onLogout || logout;

  return (
    <SafeAreaView style={styles.container}>
      <ScrollView contentContainerStyle={styles.content}>
        <View style={styles.section}>
          <Text style={styles.title}>Settings</Text>
          {user && (
            <View style={styles.card}>
              <Text style={styles.label}>Account</Text>
              <Text style={styles.value}>{user.name || user.email || 'Connected User'}</Text>
              {user.walletAddress && (
                <Text style={styles.walletText}>{user.walletAddress}</Text>
              )}
            </View>
          )}
        </View>

        <View style={styles.actions}>
          <Button
            title="Log Out"
            onPress={handleLogout}
            variant="destructive"
            style={styles.logoutButton}
            testID="logout-button"
            accessibilityLabel="Log Out"
            accessibilityHint="Logs out of your account"
          />
        </View>
      </ScrollView>
    </SafeAreaView>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: Colors.darkBackground,
  },
  content: {
    flexGrow: 1,
    padding: 24,
    justifyContent: 'space-between',
  },
  section: {
    marginBottom: 24,
  },
  title: {
    fontSize: 28,
    fontWeight: 'bold',
    color: Colors.primaryText,
    marginBottom: 16,
  },
  card: {
    backgroundColor: '#1E1E20',
    borderRadius: 12,
    padding: 16,
    borderWidth: 1,
    borderColor: '#2C2C2E',
  },
  label: {
    fontSize: 12,
    color: Colors.secondaryText,
    marginBottom: 4,
    textTransform: 'uppercase',
  },
  value: {
    fontSize: 16,
    fontWeight: '600',
    color: Colors.primaryText,
  },
  walletText: {
    fontSize: 13,
    color: Colors.secondaryText,
    marginTop: 8,
    fontFamily: 'monospace',
  },
  actions: {
    marginTop: 'auto',
    paddingTop: 24,
  },
  logoutButton: {
    marginTop: 12,
  },
});

export default SettingsScreen;
