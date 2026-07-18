import React from 'react';
import { View, Text, StyleSheet, SafeAreaView } from 'react-native';
import Colors from '@/constants/Colors';
import Button from '@/components/ui/Button';
import { useAuth } from '@/hooks/useAuth';

export default function ProfileScreen() {
  const { user, logout } = useAuth();

  return (
    <SafeAreaView style={styles.container}>
      <View style={styles.content}>
        <View style={styles.profileHeader}>
          <View style={styles.avatarPlaceholder}>
            <Text style={styles.avatarText}>
              {user?.name ? user.name.charAt(0).toUpperCase() : 'A'}
            </Text>
          </View>
          <Text style={styles.nameText}>{user?.name || 'Agora User'}</Text>
          <Text style={styles.emailText}>{user?.email || 'user@agora.events'}</Text>
        </View>

        <View style={styles.detailsCard}>
          <Text style={styles.sectionTitle}>Wallet Information</Text>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>Stellar Address</Text>
            <Text style={styles.infoValue} numberOfLines={1} ellipsizeMode="middle">
              {user?.walletAddress || 'GDAGORA...'}
            </Text>
          </View>
        </View>

        <Button
          title="Log Out"
          onPress={logout}
          variant="outline"
          style={styles.logoutButton}
        />
      </View>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: Colors.darkBackground,
  },
  content: {
    flex: 1,
    padding: 24,
    justifyContent: 'space-between',
  },
  profileHeader: {
    alignItems: 'center',
    marginTop: 20,
  },
  avatarPlaceholder: {
    width: 90,
    height: 90,
    borderRadius: 45,
    backgroundColor: Colors.primaryYellow,
    justifyContent: 'center',
    alignItems: 'center',
    marginBottom: 16,
  },
  avatarText: {
    fontSize: 32,
    fontWeight: 'bold',
    color: Colors.darkBackground,
  },
  nameText: {
    fontSize: 22,
    fontWeight: 'bold',
    color: Colors.primaryText,
    marginBottom: 4,
  },
  emailText: {
    fontSize: 14,
    color: Colors.secondaryText,
  },
  detailsCard: {
    backgroundColor: '#1E1E20',
    borderRadius: 12,
    padding: 16,
    borderWidth: 1,
    borderColor: '#2C2C2E',
    marginVertical: 32,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: 'bold',
    color: Colors.primaryText,
    marginBottom: 12,
  },
  infoRow: {
    borderTopWidth: 1,
    borderTopColor: '#2C2C2E',
    paddingTop: 12,
  },
  infoLabel: {
    fontSize: 12,
    color: Colors.secondaryText,
    marginBottom: 4,
  },
  infoValue: {
    fontSize: 14,
    color: Colors.primaryYellow,
    fontFamily: 'SpaceMono',
  },
  logoutButton: {
    borderColor: Colors.accentRed,
    color: Colors.accentRed,
    marginBottom: 20,
  },
});
