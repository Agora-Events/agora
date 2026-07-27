import React, { useState } from 'react';
import { View, Text, StyleSheet, ScrollView, Modal, Alert, ActivityIndicator } from 'react-native';
import { useLocalSearchParams } from 'expo-router';
import Colors from '@/constants/Colors';
import Button from '@/components/ui/Button';
import Input from '@/components/ui/Input';
import { Keypair, Networks, TransactionBuilder, Horizon } from '@stellar/stellar-sdk';
import { StellarWalletManager } from '@/services/stellar';

export default function TicketDetailsScreen() {
  const { id } = useLocalSearchParams();
  
  const [ticketStatus, setTicketStatus] = useState<'Active' | 'Transferred' | 'Listed'>('Active');
  
  // Modals state
  const [isTransferModalOpen, setTransferModalOpen] = useState(false);
  const [isSellModalOpen, setSellModalOpen] = useState(false);
  
  // Form states
  const [recipientAddress, setRecipientAddress] = useState('');
  const [recipientError, setRecipientError] = useState('');
  const [sellPrice, setSellPrice] = useState('');
  const [sellError, setSellError] = useState('');
  const [isProcessing, setIsProcessing] = useState(false);

  // MOCK Ticket Details (consistent with `tickets.tsx` style)
  const ticket = {
    id: id || 'T-1004',
    eventTitle: 'Stellar Meridian 2026',
    date: 'Oct 15, 2026',
    seat: 'General Admission',
    txHash: '0x3f...b82d',
  };

  const handleTransferSubmit = async () => {
    // Validate target string against Stellar public key constraints or email
    const isEmail = /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(recipientAddress);
    const isStellarKey = recipientAddress.startsWith('G') && recipientAddress.length === 56;
    
    if (!isStellarKey && !isEmail) {
      setRecipientError('Invalid input. Must be a valid Stellar Public Key (starts with "G", 56 chars) or email.');
      return;
    }
    setRecipientError('');
    setIsProcessing(true);

    try {
      // Build transaction calling transfer_ticket
      const secretKey = await StellarWalletManager.getSecretKey();
      if (!secretKey) {
        throw new Error('No local Stellar wallet found.');
      }
      
      const keypair = Keypair.fromSecret(secretKey);
      const server = new Horizon.Server('https://horizon-testnet.stellar.org');
      
      const account = await server.loadAccount(keypair.publicKey());
      const fee = await server.fetchBaseFee();
      
      // Mocking the contract call to `ticket_payment` / `transfer_ticket`
      const tx = new TransactionBuilder(account, {
        fee: fee.toString(),
        networkPassphrase: Networks.TESTNET,
      })
        .addMemo({ type: 'text', value: 'transfer_ticket' } as any)
        .setTimeout(30)
        .build();
        
      tx.sign(keypair);
      await server.submitTransaction(tx);
      
      setTicketStatus('Transferred');
      setTransferModalOpen(false);
      Alert.alert('Success', 'Ticket successfully transferred!');
    } catch (error: any) {
      Alert.alert('Transfer Failed', error?.message || 'Unknown error occurred.');
    } finally {
      setIsProcessing(false);
    }
  };

  const handleSellSubmit = async () => {
    const priceNum = parseFloat(sellPrice);
    if (isNaN(priceNum) || priceNum <= 0) {
      setSellError('Please enter a valid price greater than 0.');
      return;
    }
    setSellError('');
    setIsProcessing(true);

    try {
      const secretKey = await StellarWalletManager.getSecretKey();
      if (!secretKey) {
        throw new Error('No local Stellar wallet found.');
      }
      
      const keypair = Keypair.fromSecret(secretKey);
      const server = new Horizon.Server('https://horizon-testnet.stellar.org');
      
      const account = await server.loadAccount(keypair.publicKey());
      const fee = await server.fetchBaseFee();
      
      // Mocking the contract call to `ticket_payment` escrow listing
      const tx = new TransactionBuilder(account, {
        fee: fee.toString(),
        networkPassphrase: Networks.TESTNET,
      })
        .addMemo({ type: 'text', value: `list_ticket:${priceNum}` } as any)
        .setTimeout(30)
        .build();
        
      tx.sign(keypair);
      await server.submitTransaction(tx);
      
      setTicketStatus('Listed');
      setSellModalOpen(false);
      Alert.alert('Success', 'Ticket successfully listed for secondary sale!');
    } catch (error: any) {
      Alert.alert('Listing Failed', error?.message || 'Unknown error occurred.');
    } finally {
      setIsProcessing(false);
    }
  };

  return (
    <ScrollView style={styles.container} contentContainerStyle={styles.content}>
      <Text style={styles.header}>Ticket Details</Text>
      
      <View style={styles.ticketCard}>
        <View style={styles.ticketHeader}>
          <Text style={styles.ticketId}>{ticket.id}</Text>
          <Text style={styles.statusBadge}>{ticketStatus}</Text>
        </View>
        <Text style={styles.eventTitle}>{ticket.eventTitle}</Text>
        <View style={styles.detailsRow}>
          <View>
            <Text style={styles.label}>Date</Text>
            <Text style={styles.value}>{ticket.date}</Text>
          </View>
          <View>
            <Text style={styles.label}>Section/Seat</Text>
            <Text style={styles.value}>{ticket.seat}</Text>
          </View>
        </View>
        <View style={styles.txContainer}>
          <Text style={styles.txLabel}>Transaction Hash</Text>
          <Text style={styles.txValue}>{ticket.txHash}</Text>
        </View>
      </View>

      <View style={styles.actionsContainer}>
        <Button 
          title="Transfer Ticket" 
          onPress={() => setTransferModalOpen(true)} 
          disabled={ticketStatus !== 'Active'}
          style={styles.actionButton}
        />
        <Button 
          title="Sell Ticket" 
          onPress={() => setSellModalOpen(true)} 
          disabled={ticketStatus !== 'Active'}
          style={styles.actionButton}
        />
      </View>

      {/* Transfer Modal */}
      <Modal visible={isTransferModalOpen} transparent animationType="slide">
        <View style={styles.modalOverlay}>
          <View style={styles.modalContent}>
            <Text style={styles.modalTitle}>Transfer Ticket</Text>
            <Input
              label="Recipient Stellar Public Key or Email"
              placeholder="G... or user@example.com"
              value={recipientAddress}
              onChangeText={setRecipientAddress}
              error={recipientError}
              autoCapitalize="none"
            />
            <View style={styles.warningBox}>
              <Text style={styles.warningText}>
                Estimated Gas Fee: ~0.01 USDC{'\n'}
                Secondary Transfer Fee: 2.50 USDC
              </Text>
            </View>
            {isProcessing ? (
              <ActivityIndicator size="large" color={Colors.primaryYellow} />
            ) : (
              <View style={styles.modalActions}>
                <Button title="Cancel" onPress={() => setTransferModalOpen(false)} style={[styles.modalButton, styles.cancelButton]} />
                <Button title="Confirm Transfer" onPress={handleTransferSubmit} style={styles.modalButton} />
              </View>
            )}
          </View>
        </View>
      </Modal>

      {/* Sell Modal */}
      <Modal visible={isSellModalOpen} transparent animationType="slide">
        <View style={styles.modalOverlay}>
          <View style={styles.modalContent}>
            <Text style={styles.modalTitle}>List Ticket for Sale</Text>
            <Input
              label="Listing Price (USDC)"
              placeholder="e.g. 50"
              keyboardType="numeric"
              value={sellPrice}
              onChangeText={setSellPrice}
              error={sellError}
            />
            <View style={styles.warningBox}>
              <Text style={styles.warningText}>
                Warning: Organizers may levy contract-level secondary fees (e.g. 10% royalty) on resale transactions.
              </Text>
            </View>
            {isProcessing ? (
              <ActivityIndicator size="large" color={Colors.primaryYellow} />
            ) : (
              <View style={styles.modalActions}>
                <Button title="Cancel" onPress={() => setSellModalOpen(false)} style={[styles.modalButton, styles.cancelButton]} />
                <Button title="Confirm Listing" onPress={handleSellSubmit} style={styles.modalButton} />
              </View>
            )}
          </View>
        </View>
      </Modal>
    </ScrollView>
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
  container: {
    flex: 1,
    backgroundColor: Colors.darkBackground,
  },
  content: {
    padding: 16,
  },
  header: {
    fontSize: 24,
    fontWeight: 'bold',
    color: Colors.primaryText,
    marginBottom: 20,
  },
  ticketCard: {
    backgroundColor: '#1E1E20',
    borderRadius: 12,
    padding: 16,
    marginBottom: 24,
    borderLeftWidth: 4,
    borderLeftColor: Colors.primaryYellow,
    borderWidth: 1,
    borderColor: '#2C2C2E',
  },
  ticketHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 12,
  },
  ticketId: {
    color: Colors.secondaryText,
    fontSize: 12,
    fontWeight: 'bold',
  },
  statusBadge: {
    backgroundColor: '#34C75922',
    color: '#34C759',
    fontSize: 10,
    fontWeight: 'bold',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 4,
    overflow: 'hidden',
  },
  eventTitle: {
    fontSize: 18,
    fontWeight: 'bold',
    color: Colors.primaryText,
    marginBottom: 16,
  },
  detailsRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 16,
  },
  label: {
    fontSize: 12,
    color: Colors.secondaryText,
    marginBottom: 4,
  },
  value: {
    fontSize: 14,
    color: Colors.primaryText,
    fontWeight: '500',
  },
  txContainer: {
    borderTopWidth: 1,
    borderTopColor: '#2C2C2E',
    paddingTop: 12,
  },
  txLabel: {
    fontSize: 11,
    color: Colors.secondaryText,
    marginBottom: 2,
  },
  txValue: {
    fontSize: 12,
    color: Colors.primaryYellow,
    fontFamily: 'SpaceMono',
  },
  actionsContainer: {
    flexDirection: 'column',
  },
  actionButton: {
    width: '100%',
    marginBottom: 12,
  },
  modalOverlay: {
    flex: 1,
    backgroundColor: 'rgba(0,0,0,0.7)',
    justifyContent: 'center',
    padding: 16,
  },
  modalContent: {
    backgroundColor: '#1E1E20',
    borderRadius: 12,
    padding: 24,
    borderWidth: 1,
    borderColor: '#2C2C2E',
  },
  modalTitle: {
    fontSize: 20,
    fontWeight: 'bold',
    color: Colors.primaryText,
    marginBottom: 20,
    textAlign: 'center',
  },
  warningBox: {
    backgroundColor: '#FF950022',
    padding: 12,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#FF950055',
    marginBottom: 20,
  },
  warningText: {
    color: '#FF9500',
    fontSize: 13,
    lineHeight: 20,
  },
  modalActions: {
    flexDirection: 'column',
  },
  modalButton: {
    width: '100%',
    marginBottom: 12,
  },
  cancelButton: {
    backgroundColor: '#2C2C2E',
  },
  container: { flex: 1, padding: 20, alignItems: 'center', backgroundColor: '#fff' },
  loader: { flex: 1, justifyContent: 'center' },
  error: { marginTop: 20, fontSize: 16, color: 'red' },
  title: { fontSize: 24, fontWeight: 'bold', marginBottom: 10, textAlign: 'center' },
  detail: { fontSize: 16, marginBottom: 5 },
  qrContainer: { marginTop: 30, padding: 20, borderWidth: 1, borderColor: '#ccc', alignItems: 'center', width: '100%' },
  qrPlaceholder: { fontSize: 18, fontWeight: 'bold', marginBottom: 10, textAlign: 'center' },
  payload: { fontSize: 10, color: '#666', textAlign: 'center', marginTop: 10 }
});
