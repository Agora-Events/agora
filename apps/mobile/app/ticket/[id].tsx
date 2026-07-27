import React, { useEffect, useState, useCallback, useRef } from 'react';
import { View, Text, StyleSheet, ActivityIndicator } from 'react-native';
import { useLocalSearchParams } from 'expo-router';
import * as SecureStore from 'expo-secure-store';
import { Keypair } from '@stellar/stellar-sdk';

interface Ticket {
  id: string;
  eventName: string;
  date: string;
  venue: string;
  seat: string;
}

export default function TicketDetail() {
  const { id } = useLocalSearchParams<{ id: string }>();
  const [ticket, setTicket] = useState<Ticket | null>(null);
  const [loading, setLoading] = useState(true);
  const [qrPayload, setQrPayload] = useState<string | null>(null);
  const timerRef = useRef<NodeJS.Timeout | null>(null);

  const fetchTicket = useCallback(async () => {
    try {
      // Offline support: try to load from cache first
      const cached = await SecureStore.getItemAsync('ticket_cache');
      let tickets: Ticket[] = cached ? JSON.parse(cached) : [];

      // Try to fetch from API if online
      try {
        const response = await fetch('/api/profile');
        if (response.ok) {
          const data = await response.json();
          tickets = data.tickets || [];
          await SecureStore.setItemAsync('ticket_cache', JSON.stringify(tickets));
        }
      } catch (e) {
        // Offline: continue with cached tickets
      }

      const found = tickets.find((t) => t.id === id);
      if (found) {
        setTicket(found);
      }
    } catch (e) {
      console.error('Error fetching ticket', e);
    } finally {
      setLoading(false);
    }
  }, [id]);

  const generateDynamicPayload = useCallback(async () => {
    if (!id) return;
    try {
      // Fetch private key from secure store ONLY for the signature process
      const privateKey = await SecureStore.getItemAsync('privateKey');
      if (!privateKey) {
        console.error('No private key found');
        return;
      }

      // Use system time (works offline)
      const timestamp = Math.floor(Date.now() / 1000);
      const payload = {
        ticketId: id,
        timestamp,
      };

      const payloadString = JSON.stringify(payload);
      const keypair = Keypair.fromSecret(privateKey);
      
      // Cryptographic signature
      const signatureBuffer = keypair.sign(Buffer.from(payloadString));
      const signature = signatureBuffer.toString('base64');

      setQrPayload(JSON.stringify({
        ...payload,
        signature
      }));
      
      // Explicitly clear from memory by not keeping it in component state
      // Garbage collection will handle the local variable cleanup.
    } catch (e) {
      console.error('Error generating payload', e);
    }
  }, [id]);

  useEffect(() => {
    fetchTicket();
  }, [fetchTicket]);

  useEffect(() => {
    if (ticket) {
      // Apply a high screen brightness configuration when displaying the QR code
      // Note: Mocked because no screen brightness library could be introduced per constraints.
      console.log('Setting screen brightness to maximum');
      
      generateDynamicPayload();
      timerRef.current = setInterval(() => {
        generateDynamicPayload();
      }, 15000);

      return () => {
        if (timerRef.current) clearInterval(timerRef.current);
        console.log('Restoring screen brightness');
      };
    }
  }, [ticket, generateDynamicPayload]);

  if (loading) {
    return <ActivityIndicator style={styles.loader} />;
  }

  if (!ticket) {
    return <Text style={styles.error}>Ticket not found.</Text>;
  }

  return (
    <View style={styles.container}>
      <Text style={styles.title}>{ticket.eventName}</Text>
      <Text style={styles.detail}>Date: {ticket.date}</Text>
      <Text style={styles.detail}>Venue: {ticket.venue}</Text>
      <Text style={styles.detail}>Seat: {ticket.seat}</Text>
      
      <View style={styles.qrContainer}>
        {/* Placeholder for visual QR Code, as a library could not be introduced per constraints */}
        <Text style={styles.qrPlaceholder}>[QR Code Placeholder]</Text>
        <Text style={styles.payload}>{qrPayload}</Text>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, padding: 20, alignItems: 'center', backgroundColor: '#fff' },
  loader: { flex: 1, justifyContent: 'center' },
  error: { marginTop: 20, fontSize: 16, color: 'red' },
  title: { fontSize: 24, fontWeight: 'bold', marginBottom: 10, textAlign: 'center' },
  detail: { fontSize: 16, marginBottom: 5 },
  qrContainer: { marginTop: 30, padding: 20, borderWidth: 1, borderColor: '#ccc', alignItems: 'center', width: '100%' },
  qrPlaceholder: { fontSize: 18, fontWeight: 'bold', marginBottom: 10, textAlign: 'center' },
  payload: { fontSize: 10, color: '#666', textAlign: 'center', marginTop: 10 }
});
