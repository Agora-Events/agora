import React, { useState, useMemo } from 'react';
import {
  Modal,
  StyleSheet,
  Text,
  View,
  TouchableOpacity,
  ScrollView,
  ActivityIndicator,
  Alert,
} from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import * as Clipboard from 'expo-clipboard';
import { useThemeContext } from '@/context/ThemeContext';
import { Colors } from '@/constants/Colors';
import {
  SUPPORTED_WALLETS,
  WalletType,
  WalletMetadata,
  DecodedXDR,
  decodeXDR,
} from '@/services/walletAdapter';
import { useWallet } from '@/hooks/useWallet';

export interface WalletConnectModalProps {
  visible: boolean;
  onClose: () => void;
  /** Optional transaction XDR to preview and approve */
  pendingXdr?: string;
  onConfirmSign?: (signedXdr: string) => void;
}

export function WalletConnectModal({
  visible,
  onClose,
  pendingXdr,
  onConfirmSign,
}: WalletConnectModalProps) {
  const { colorScheme } = useThemeContext();
  const isDark = colorScheme === 'dark';
  const {
    walletType,
    publicKey,
    status,
    isConnecting,
    error,
    connect,
    disconnect,
    signTransaction,
  } = useWallet();

  const [connectingType, setConnectingType] = useState<WalletType | null>(null);
  const [signing, setSigning] = useState(false);
  const [copied, setCopied] = useState(false);

  const decodedTx: DecodedXDR | null = useMemo(() => {
    if (!pendingXdr) return null;
    return decodeXDR(pendingXdr);
  }, [pendingXdr]);

  const handleSelectWallet = async (wallet: WalletMetadata) => {
    try {
      setConnectingType(wallet.id);
      await connect(wallet.id);
    } catch (err: any) {
      Alert.alert('Connection Error', err?.message || 'Could not connect to wallet.');
    } finally {
      setConnectingType(null);
    }
  };

  const handleCopyPublicKey = async () => {
    if (publicKey) {
      await Clipboard.setStringAsync(publicKey);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const handleApproveSign = async () => {
    if (!pendingXdr) return;
    try {
      setSigning(true);
      const signed = await signTransaction(pendingXdr);
      if (onConfirmSign) {
        onConfirmSign(signed);
      }
      onClose();
    } catch (err: any) {
      Alert.alert('Signing Failed', err?.message || 'Failed to sign transaction.');
    } finally {
      setSigning(false);
    }
  };

  const formatAddress = (addr: string) => {
    if (addr.length <= 12) return addr;
    return `${addr.slice(0, 6)}...${addr.slice(-6)}`;
  };

  const bg = Colors[colorScheme].background;
  const cardBg = Colors[colorScheme].cardBackground;
  const textColor = isDark ? '#FFFFFF' : '#111827';
  const subtextColor = isDark ? '#9CA3AF' : '#6B7280';
  const borderColor = Colors[colorScheme].border;

  return (
    <Modal
      visible={visible}
      animationType="slide"
      transparent
      onRequestClose={onClose}
    >
      <View style={styles.overlay}>
        <View style={[styles.modalCard, { backgroundColor: bg, borderColor }]}>
          {/* Header */}
          <View style={[styles.header, { borderBottomColor: borderColor }]}>
            <View style={styles.titleRow}>
              <Ionicons
                name={pendingXdr ? 'receipt-outline' : 'wallet-outline'}
                size={22}
                color={Colors.primaryYellow}
              />
              <Text style={[styles.title, { color: textColor }]}>
                {pendingXdr ? 'Review & Sign Transaction' : 'Connect Stellar Wallet'}
              </Text>
            </View>
            <TouchableOpacity
              onPress={onClose}
              style={styles.closeBtn}
              accessibilityRole="button"
              accessibilityLabel="Close wallet modal"
            >
              <Ionicons name="close" size={22} color={subtextColor} />
            </TouchableOpacity>
          </View>

          <ScrollView style={styles.body} contentContainerStyle={styles.bodyContent}>
            {/* If there is a pending XDR to review */}
            {pendingXdr && decodedTx ? (
              <View style={styles.xdrContainer}>
                <View style={[styles.badge, { backgroundColor: '#3B82F622' }]}>
                  <Text style={styles.badgeText}>Soroban Smart Contract Invocation</Text>
                </View>

                {decodedTx.functionName && (
                  <View style={[styles.txField, { backgroundColor: cardBg, borderColor }]}>
                    <Text style={[styles.fieldLabel, { color: subtextColor }]}>Function Call</Text>
                    <Text style={[styles.fieldValueBold, { color: Colors.primaryYellow }]}>
                      {decodedTx.functionName}
                    </Text>
                  </View>
                )}

                {decodedTx.contractAddress && (
                  <View style={[styles.txField, { backgroundColor: cardBg, borderColor }]}>
                    <Text style={[styles.fieldLabel, { color: subtextColor }]}>Contract ID</Text>
                    <Text style={[styles.fieldValueMono, { color: textColor }]} numberOfLines={1}>
                      {decodedTx.contractAddress}
                    </Text>
                  </View>
                )}

                <View style={styles.detailsRow}>
                  <View style={[styles.detailBox, { backgroundColor: cardBg, borderColor }]}>
                    <Text style={[styles.fieldLabel, { color: subtextColor }]}>Max Fee</Text>
                    <Text style={[styles.detailValue, { color: textColor }]}>
                      {decodedTx.fee} stroops
                    </Text>
                  </View>
                  <View style={[styles.detailBox, { backgroundColor: cardBg, borderColor }]}>
                    <Text style={[styles.fieldLabel, { color: subtextColor }]}>Operations</Text>
                    <Text style={[styles.detailValue, { color: textColor }]}>
                      {decodedTx.operationCount}
                    </Text>
                  </View>
                </View>

                {decodedTx.resourceLimits && (
                  <View style={[styles.txField, { backgroundColor: cardBg, borderColor }]}>
                    <Text style={[styles.fieldLabel, { color: subtextColor }]}>Resource Limits</Text>
                    <Text style={[styles.fieldValue, { color: subtextColor }]}>
                      CPU Instructions: {decodedTx.resourceLimits.instructions?.toLocaleString() || 'N/A'}
                    </Text>
                  </View>
                )}

                {/* Signing Actions */}
                <View style={styles.signActionContainer}>
                  <TouchableOpacity
                    style={[styles.actionBtn, styles.approveBtn]}
                    onPress={handleApproveSign}
                    disabled={signing}
                    accessibilityRole="button"
                    accessibilityLabel="Approve and Sign Transaction"
                  >
                    {signing ? (
                      <ActivityIndicator size="small" color="#000000" />
                    ) : (
                      <Text style={styles.approveBtnText}>Approve & Sign</Text>
                    )}
                  </TouchableOpacity>

                  <TouchableOpacity
                    style={[styles.actionBtn, styles.cancelBtn, { borderColor }]}
                    onPress={onClose}
                    disabled={signing}
                    accessibilityRole="button"
                    accessibilityLabel="Reject Transaction"
                  >
                    <Text style={[styles.cancelBtnText, { color: subtextColor }]}>Reject</Text>
                  </TouchableOpacity>
                </View>
              </View>
            ) : (
              /* Wallet Connection Flow */
              <View>
                {publicKey ? (
                  // Connected State
                  <View style={[styles.connectedCard, { backgroundColor: cardBg, borderColor }]}>
                    <View style={styles.connectedRow}>
                      <View style={styles.connectedStatusDot} />
                      <Text style={[styles.connectedLabel, { color: subtextColor }]}>
                        Connected with {walletType ? walletType.toUpperCase() : 'Wallet'}
                      </Text>
                    </View>

                    <TouchableOpacity
                      onPress={handleCopyPublicKey}
                      style={styles.addressPill}
                      accessibilityRole="button"
                      accessibilityLabel="Copy public address"
                    >
                      <Text style={[styles.addressText, { color: textColor }]}>
                        {formatAddress(publicKey)}
                      </Text>
                      <Ionicons
                        name={copied ? 'checkmark-circle' : 'copy-outline'}
                        size={16}
                        color={copied ? '#10B981' : subtextColor}
                      />
                    </TouchableOpacity>

                    <TouchableOpacity
                      style={styles.disconnectBtn}
                      onPress={disconnect}
                      accessibilityRole="button"
                      accessibilityLabel="Disconnect wallet"
                    >
                      <Ionicons name="log-out-outline" size={16} color="#EF4444" />
                      <Text style={styles.disconnectText}>Disconnect Wallet</Text>
                    </TouchableOpacity>
                  </View>
                ) : (
                  // List of Providers
                  <View style={styles.walletList}>
                    <Text style={[styles.subtitle, { color: subtextColor }]}>
                      Select a wallet to sign Soroban transactions and manage your NFT event tickets.
                    </Text>

                    {error && (
                      <View style={styles.errorBanner}>
                        <Ionicons name="alert-circle-outline" size={16} color="#DC2626" />
                        <Text style={styles.errorBannerText}>{error}</Text>
                      </View>
                    )}

                    {SUPPORTED_WALLETS.map((w) => {
                      const isBusy = isConnecting && connectingType === w.id;
                      return (
                        <TouchableOpacity
                          key={w.id}
                          style={[styles.walletItem, { backgroundColor: cardBg, borderColor }]}
                          onPress={() => handleSelectWallet(w)}
                          disabled={isConnecting}
                          accessibilityRole="button"
                          accessibilityLabel={`Connect with ${w.name}`}
                        >
                          <View style={styles.walletIconCircle}>
                            <Ionicons name={w.icon as any} size={22} color={Colors.primaryYellow} />
                          </View>
                          <View style={styles.walletInfo}>
                            <Text style={[styles.walletName, { color: textColor }]}>{w.name}</Text>
                            <Text style={[styles.walletDesc, { color: subtextColor }]} numberOfLines={1}>
                              {w.description}
                            </Text>
                          </View>
                          {isBusy ? (
                            <ActivityIndicator size="small" color={Colors.primaryYellow} />
                          ) : (
                            <Ionicons name="chevron-forward" size={18} color={subtextColor} />
                          )}
                        </TouchableOpacity>
                      );
                    })}
                  </View>
                )}
              </View>
            )}
          </ScrollView>
        </View>
      </View>
    </Modal>
  );
}

const styles = StyleSheet.create({
  overlay: {
    flex: 1,
    backgroundColor: 'rgba(0, 0, 0, 0.7)',
    justifyContent: 'flex-end',
  },
  modalCard: {
    borderTopLeftRadius: 24,
    borderTopRightRadius: 24,
    borderTopWidth: 1,
    borderLeftWidth: 1,
    borderRightWidth: 1,
    maxHeight: '85%',
    minHeight: 380,
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingHorizontal: 20,
    paddingVertical: 16,
    borderBottomWidth: 1,
  },
  titleRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 10,
  },
  title: {
    fontSize: 17,
    fontWeight: '700',
  },
  closeBtn: {
    padding: 4,
  },
  body: {
    flex: 1,
  },
  bodyContent: {
    padding: 20,
    paddingBottom: 36,
  },
  subtitle: {
    fontSize: 14,
    lineHeight: 20,
    marginBottom: 16,
  },
  errorBanner: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
    backgroundColor: '#FEE2E2',
    padding: 10,
    borderRadius: 8,
    marginBottom: 14,
  },
  errorBannerText: {
    color: '#991B1B',
    fontSize: 13,
    flexShrink: 1,
  },
  walletList: {
    gap: 10,
  },
  walletItem: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 14,
    borderRadius: 14,
    borderWidth: 1,
  },
  walletIconCircle: {
    width: 40,
    height: 40,
    borderRadius: 20,
    backgroundColor: 'rgba(253, 218, 35, 0.12)',
    alignItems: 'center',
    justifyContent: 'center',
    marginRight: 12,
  },
  walletInfo: {
    flex: 1,
  },
  walletName: {
    fontSize: 15,
    fontWeight: '600',
    marginBottom: 2,
  },
  walletDesc: {
    fontSize: 12,
  },
  connectedCard: {
    padding: 18,
    borderRadius: 16,
    borderWidth: 1,
    alignItems: 'center',
    gap: 14,
  },
  connectedRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  connectedStatusDot: {
    width: 8,
    height: 8,
    borderRadius: 4,
    backgroundColor: '#10B981',
  },
  connectedLabel: {
    fontSize: 13,
    fontWeight: '500',
  },
  addressPill: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
    backgroundColor: 'rgba(253, 218, 35, 0.1)',
    paddingVertical: 8,
    paddingHorizontal: 16,
    borderRadius: 20,
  },
  addressText: {
    fontSize: 15,
    fontWeight: '700',
    fontFamily: 'monospace',
  },
  disconnectBtn: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
    paddingVertical: 6,
    paddingHorizontal: 12,
  },
  disconnectText: {
    color: '#EF4444',
    fontSize: 13,
    fontWeight: '600',
  },
  xdrContainer: {
    gap: 12,
  },
  badge: {
    alignSelf: 'flex-start',
    paddingVertical: 4,
    paddingHorizontal: 10,
    borderRadius: 8,
    marginBottom: 4,
  },
  badgeText: {
    color: '#3B82F6',
    fontSize: 12,
    fontWeight: '700',
  },
  txField: {
    padding: 12,
    borderRadius: 12,
    borderWidth: 1,
  },
  fieldLabel: {
    fontSize: 11,
    fontWeight: '600',
    textTransform: 'uppercase',
    marginBottom: 4,
  },
  fieldValueBold: {
    fontSize: 15,
    fontWeight: '700',
  },
  fieldValueMono: {
    fontSize: 13,
    fontFamily: 'monospace',
  },
  fieldValue: {
    fontSize: 13,
  },
  detailsRow: {
    flexDirection: 'row',
    gap: 10,
  },
  detailBox: {
    flex: 1,
    padding: 12,
    borderRadius: 12,
    borderWidth: 1,
  },
  detailValue: {
    fontSize: 15,
    fontWeight: '700',
  },
  signActionContainer: {
    marginTop: 10,
    gap: 10,
  },
  actionBtn: {
    paddingVertical: 14,
    borderRadius: 12,
    alignItems: 'center',
    justifyContent: 'center',
  },
  approveBtn: {
    backgroundColor: Colors.primaryYellow,
  },
  approveBtnText: {
    color: '#000000',
    fontSize: 15,
    fontWeight: '700',
  },
  cancelBtn: {
    borderWidth: 1,
    backgroundColor: 'transparent',
  },
  cancelBtnText: {
    fontSize: 14,
    fontWeight: '600',
  },
});

export default WalletConnectModal;
