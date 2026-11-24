import React from 'react';
import { SafeAreaView, ScrollView, StyleSheet, Text, View } from 'react-native';
import Colors from '@/constants/Colors';
import CurrencySelector from '@/components/CurrencySelector';

export default function SettingsScreen() {
  return (
    <SafeAreaView style={styles.container}>
      <ScrollView contentContainerStyle={styles.content}>
        <View style={styles.card}>
          <Text style={styles.sectionTitle}>Display Currency</Text>
          <Text style={styles.description}>
            Prices are charged in USDC and shown in your preferred currency.
          </Text>
          <CurrencySelector />
        </View>
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: Colors.darkBackground,
  },
  content: {
    padding: 24,
  },
  card: {
    backgroundColor: Colors.dark.cardBackground,
    borderRadius: 12,
    padding: 16,
    borderWidth: 1,
    borderColor: Colors.dark.border,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: 'bold',
    color: Colors.primaryText,
    marginBottom: 4,
  },
  description: {
    fontSize: 13,
    color: Colors.secondaryText,
    marginBottom: 12,
  },
});
