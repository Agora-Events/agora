/**
 * ticket/[id].tsx — Issue #1179: Offline Ticket Vault
 *
 * Ticket detail screen with:
 *   - Biometric authentication gate before any vault decryption (issue #1179)
 *   - Rotating QR display via DynamicQrView (replaces the old ad-hoc approach)
 *   - Explicit biometric-unavailable and biometric-failure states
 *
 * Biometric flow:
 *   1. Screen mounts → useBiometricAuth checks hardware availability.
 *   2. If unavailable (no hardware / not enrolled): show an error banner;
 *      the QR section is never shown. Do NOT silently unlock.
 *   3. User taps "Show Ticket QR" → OS biometric prompt appears.
 *   4. On success: vault secret is read, keypair derived in memory, passed
 *      to DynamicQrView. The secretKey is held only in React state for the
 *      lifetime of the screen — it is cleared on unmount.
 *   5. On failure / cancel: explicit error message shown; QR stays hidden.
 *
 * Android note: expo-secure-store on Android does not self-prompt for
 * biometrics. We call useBiometricAuth().authenticate() first, and only call
 * getTicketSecret() after it resolves with success. On iOS the Keychain item
 * itself requires authentication, so the OS prompts automatically — the
 * explicit authenticate() call before reading is belt-and-suspenders.
 */

import React, { useState, useEffect, useCallback } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  Modal,
  Alert,
  ActivityIndicator,
  TouchableOpacity,
} from 'react-native';
import { useLocalSearchParams } from 'expo-router';
import Colors from '@/constants/Colors';
import Button from '@/components/ui/Button';
import Input from '@/components/ui/Input';
import { Keypair, Networks, TransactionBuilder, Horizon } from '@stellar/stellar-sdk';
import { StellarWalletManager } from '@/services/stellar';
import { useBiometricAuth } from '@/hooks/useBiometricAuth';
import { getTicketSecret } from '@/services/ticketVault';
import { deriveTicketKeyPair } from '@/lib/crypto';
import DynamicQrView from '@/components/ticket/DynamicQrView';

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

  // Vault state — secretKey held in memory only, never persisted to state storage.
  // Cleared when the screen unmounts.
  const [vaultSecretKey, setVaultSecretKey] = useState<Uint8Array | null>(null);
  const [vaultError, setVaultError] = useState<string | null>(null);
  const [vaultLoading, setVaultLoading] = useState(false);

  const biometric = useBiometricAuth();

  // Clear the in-memory key when the screen unmounts to minimise the window
  // during which the key is resident in JS heap.
  useEffect(() => {
    return () => {
      setVaultSecretKey(null);
    };
  }, []);

  // MOCK Ticket Details (consistent with `tickets.tsx` style)
  // TODO: Replace with real ticket lookup once the ticket store is wired up.
  const ticket = {
    id: id || 'T-1004',
    eventTitle: 'Stellar Meridian 2026',
    date: 'Oct 15, 2026',
    seat: 'General Admission',
    txHash: '0x3f...b82d',
    // paymentId corresponds to the payment ID used when the ticket was purchased.
    // In production this comes from the ticket store / on-chain lookup.
    paymentId: String(id || 'T-1004'),
  };

  /**
   * Gate: authenticate with biometrics, then read the vault.
   *
   * Called when the user taps "Show Ticket QR". Never called automatically
   * on mount — we require explicit user intent before prompting biometrics.
   */
  const handleShowQr = useCallback(async () => {
    if (biometric.state === 'unavailable') {
      // Already shown the banner — no point prompting again.
      return;
    }

    setVaultError(null);
    setVaultLoading(true);

    try {
      // Step 1: authenticate (required on Android; belt-and-suspenders on iOS).
      await biometric.authenticate();

      // If authentication failed, biometric.state will be 'failed' or 'error'.
      // We check after awaiting because authenticate() does not throw on failure.
      if (biometric.state !== 'success') {
        setVaultLoading(false);
        return;
      }

      // Step 2: read vault (OS may re-prompt on iOS if required).
      const secretBytes = await getTicketSecret(ticket.paymentId);
      if (!secretBytes) {
        setVaultError(
          'No ticket secret found for this ticket. This device may not be the original ' +
          'purchase device, or the ticket was purchased on a different account.'
        );
        setVaultLoading(false);
        return;
      }

      // Step 3: derive the signing keypair in memory.
      const { secretKey } = deriveTicketKeyPair(secretBytes);
      setVaultSecretKey(secretKey);
    } catch (err) {
      setVaultError(
        err instanceof Error ? err.message : 'Failed to access the ticket vault.'
      );
    } finally {
      setVaultLoading(false);
    }
  }, [biometric, ticket.paymentId]);

  /** Hide the QR and clear the in-memory key. */
  const handleHideQr = useCallback(() => {
    setVaultSecretKey(null);
    biometric.reset();
  }, [biometric]);

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

      {/* ── Biometric gate + Rotating QR ──────────────────────────────────── */}
      <View style={styles.qrSection}>
        <Text style={styles.sectionTitle}>Entry QR Code</Text>

        {/* Biometric unavailable — explicit error, no silent passthrough */}
        {biometric.state === 'unavailable' && (
          <View style={styles.warningBox} accessibilityRole="alert">
            <Text style={styles.warningText}>
              ⚠️ Biometric authentication is not available on this device.{'\n'}
              {biometric.errorMessage ?? 'Please enroll Face ID or Touch ID to display your ticket QR.'}
            </Text>
          </View>
        )}

        {/* Biometric failed — explicit error, no silent passthrough */}
        {(biometric.state === 'failed' || biometric.state === 'error') && !vaultSecretKey && (
          <View style={styles.errorBox} accessibilityRole="alert">
            <Text style={styles.errorText}>
              {biometric.errorMessage ?? 'Authentication failed. Please try again.'}
            </Text>
            <TouchableOpacity onPress={handleShowQr} style={styles.retryButton}>
              <Text style={styles.retryButtonText}>Try Again</Text>
            </TouchableOpacity>
          </View>
        )}

        {/* Vault error */}
        {vaultError && (
          <View style={styles.errorBox} accessibilityRole="alert">
            <Text style={styles.errorText}>{vaultError}</Text>
          </View>
        )}

        {/* Loading */}
        {vaultLoading && (
          <View style={styles.loadingBox}>
            <ActivityIndicator size="large" color={Colors.primaryYellow} />
            <Text style={styles.loadingText}>Verifying…</Text>
          </View>
        )}

        {/* QR unlocked and visible */}
        {vaultSecretKey && !vaultLoading && (
          <>
            <DynamicQrView
              ticketId={ticket.paymentId}
              secretKey={vaultSecretKey}
            />
            <TouchableOpacity onPress={handleHideQr} style={styles.hideButton}>
              <Text style={styles.hideButtonText}>Hide QR</Text>
            </TouchableOpacity>
          </>
        )}

        {/* Prompt to authenticate */}
        {!vaultSecretKey && !vaultLoading && biometric.state !== 'unavailable' && !vaultError && (
          <Button
            title="Show Ticket QR"
            onPress={handleShowQr}
            style={styles.actionButton}
          />
        )}
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
  // ── QR / biometric gate section ──────────────────────────────────────────
  qrSection: {
    marginBottom: 24,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: Colors.primaryText,
    marginBottom: 12,
  },
  loadingBox: {
    alignItems: 'center',
    padding: 20,
  },
  loadingText: {
    color: Colors.secondaryText,
    marginTop: 8,
    fontSize: 13,
  },
  errorBox: {
    backgroundColor: '#FF3B3022',
    padding: 14,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#FF3B3055',
    marginBottom: 12,
  },
  errorText: {
    color: '#FF3B30',
    fontSize: 13,
    lineHeight: 20,
  },
  retryButton: {
    marginTop: 8,
    alignSelf: 'flex-start',
  },
  retryButtonText: {
    color: Colors.primaryYellow,
    fontWeight: '600',
    fontSize: 13,
  },
  hideButton: {
    marginTop: 12,
    alignSelf: 'center',
  },
  hideButtonText: {
    color: Colors.secondaryText,
    fontSize: 13,
    textDecorationLine: 'underline',
  },
});

