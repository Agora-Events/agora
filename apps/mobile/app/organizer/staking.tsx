import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  SafeAreaView,
  ScrollView,
  Pressable,
  Linking,
  ActivityIndicator,
} from 'react-native';
import Colors from '@/constants/Colors';
import Button from '@/components/ui/Button';
import Input from '@/components/ui/Input';
import { useAuth } from '@/hooks/useAuth';
import { useStaking } from '@/hooks/useStaking';
import { explorerUrl, formatDate, StakingTransaction } from '@/services/staking';
import { StellarWalletManager } from '@/services/stellar';

export default function OrganizerStakingScreen() {
  const { user } = useAuth();
  const [publicKey, setPublicKey] = useState<string | null>(user?.walletAddress || null);

  useEffect(() => {
    const fetchKey = async () => {
      const storedSecret = await StellarWalletManager.getSecretKey();
      if (storedSecret) {
        const key = StellarWalletManager.getPublicKeyFromSecret(storedSecret);
        setPublicKey(key);
      } else if (user?.walletAddress) {
        setPublicKey(user.walletAddress);
      }
    };
    fetchKey();
  }, [user?.walletAddress]);

  const {
    config,
    configLoading,
    stakingState,
    stateLoading,
    history,
    historyLoading,
    usdcBalance,
    handleStake,
    handleUnstake,
    handleClaim,
  } = useStaking(publicKey);

  const [stakeAmount, setStakeAmount] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [feedbackMessage, setFeedbackMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);

  const parsedAmount = parseFloat(stakeAmount) || 0;
  
  // Acceptance Criterion: If the user's wallet balance is lower than the amount they enter,
  // the stake submission button is disabled.
  const isBalanceInsufficient = parsedAmount > usdcBalance;
  const isStakeDisabled =
    parsedAmount <= 0 ||
    isBalanceInsufficient ||
    isSubmitting ||
    !publicKey;

  const onStakePress = async () => {
    if (isStakeDisabled) return;
    setFeedbackMessage(null);
    setIsSubmitting(true);
    try {
      const res = await handleStake(parsedAmount);
      if (res.success) {
        setFeedbackMessage({ type: 'success', text: res.message });
        setStakeAmount('');
      } else {
        setFeedbackMessage({ type: 'error', text: res.message });
      }
    } catch (err: any) {
      setFeedbackMessage({ type: 'error', text: err?.message || 'Transaction failed' });
    } finally {
      setIsSubmitting(false);
    }
  };

  const onUnstakePress = async () => {
    setFeedbackMessage(null);
    setIsSubmitting(true);
    try {
      const res = await handleUnstake();
      if (res.success) {
        setFeedbackMessage({ type: 'success', text: res.message });
      } else {
        setFeedbackMessage({ type: 'error', text: res.message });
      }
    } catch (err: any) {
      setFeedbackMessage({ type: 'error', text: err?.message || 'Transaction failed' });
    } finally {
      setIsSubmitting(false);
    }
  };

  const onClaimPress = async () => {
    setFeedbackMessage(null);
    setIsSubmitting(true);
    try {
      const res = await handleClaim();
      if (res.success) {
        setFeedbackMessage({ type: 'success', text: res.message });
      } else {
        setFeedbackMessage({ type: 'error', text: res.message });
      }
    } catch (err: any) {
      setFeedbackMessage({ type: 'error', text: err?.message || 'Transaction failed' });
    } finally {
      setIsSubmitting(false);
    }
  };

  const openExplorer = (txHash: string) => {
    const url = explorerUrl(txHash);
    Linking.openURL(url).catch(() => {
      // Fallback handling
    });
  };

  const isVerified = stakingState?.status === 'Verified';

  return (
    <SafeAreaView style={styles.container}>
      <ScrollView contentContainerStyle={styles.content}>
        {/* Header Section */}
        <View style={styles.headerContainer}>
          <Text style={styles.headerTitle}>Organizer Collateral Dashboard</Text>
          <Text style={styles.headerSubtitle}>
            Stake USDC collateral to verify your profile, build attendee trust, and earn rewards.
          </Text>
        </View>

        {/* Status Card */}
        <View style={styles.card}>
          <View style={styles.cardHeaderRow}>
            <Text style={styles.cardTitle}>Staking Status</Text>
            <View
              style={[
                styles.badge,
                isVerified ? styles.badgeVerified : styles.badgeUnverified,
              ]}
            >
              <Text
                style={[
                  styles.badgeText,
                  isVerified ? styles.badgeTextVerified : styles.badgeTextUnverified,
                ]}
              >
                {stateLoading ? 'Loading...' : stakingState?.status || 'Unverified'}
              </Text>
            </View>
          </View>

          <View style={styles.metricsGrid}>
            <View style={styles.metricBox}>
              <Text style={styles.metricLabel}>Staked Collateral</Text>
              <Text style={styles.metricValue}>
                {stateLoading ? '...' : `${stakingState?.stakedAmount ?? 0} USDC`}
              </Text>
            </View>
            <View style={styles.metricBox}>
              <Text style={styles.metricLabel}>Pending Rewards</Text>
              <Text style={styles.metricValue}>
                {stateLoading ? '...' : `${stakingState?.pendingRewards ?? 0} USDC`}
              </Text>
            </View>
          </View>

          <View style={styles.balanceInfoRow}>
            <Text style={styles.balanceInfoLabel}>Available USDC Balance:</Text>
            <Text style={styles.balanceInfoValue}>{usdcBalance} USDC</Text>
          </View>
        </View>

        {/* Contract Configuration */}
        <View style={styles.card}>
          <Text style={styles.cardTitle}>Contract Configuration</Text>
          <Text style={styles.configDescription}>Fetched from event_registry contract</Text>
          {configLoading ? (
            <ActivityIndicator color={Colors.primaryYellow} style={{ marginVertical: 10 }} />
          ) : (
            <View style={styles.configContainer}>
              <View style={styles.configRow}>
                <Text style={styles.configLabel}>Staking Token Address:</Text>
                <Text style={styles.configValue} numberOfLines={1} ellipsizeMode="middle">
                  {config?.tokenAddress || 'N/A'}
                </Text>
              </View>
              <View style={styles.configRow}>
                <Text style={styles.configLabel}>Minimum Required Stake:</Text>
                <Text style={styles.configValue}>
                  {config ? `${config.minimumStake} USDC` : 'N/A'}
                </Text>
              </View>
            </View>
          )}
        </View>

        {/* Feedback Message */}
        {feedbackMessage && (
          <View
            style={[
              styles.feedbackBox,
              feedbackMessage.type === 'success' ? styles.feedbackSuccess : styles.feedbackError,
            ]}
          >
            <Text
              style={[
                styles.feedbackText,
                feedbackMessage.type === 'success'
                  ? styles.feedbackTextSuccess
                  : styles.feedbackTextError,
              ]}
            >
              {feedbackMessage.text}
            </Text>
          </View>
        )}

        {/* Actions Form */}
        <View style={styles.card}>
          <Text style={styles.cardTitle}>Stake Collateral</Text>
          <Input
            label="Amount (USDC)"
            placeholder="e.g. 500"
            keyboardType="numeric"
            value={stakeAmount}
            onChangeText={setStakeAmount}
            error={
              isBalanceInsufficient
                ? `Insufficient balance (${usdcBalance} USDC available)`
                : undefined
            }
          />
          <Button
            title={isSubmitting ? 'Processing...' : 'Stake USDC'}
            onPress={onStakePress}
            disabled={isStakeDisabled}
            loading={isSubmitting}
            style={styles.actionBtn}
          />
        </View>

        <View style={styles.card}>
          <Text style={styles.cardTitle}>Manage Lockup & Rewards</Text>
          <View style={styles.manageButtonsRow}>
            <View style={{ flex: 1, marginRight: 8 }}>
              <Button
                title="Unstake Collateral"
                variant="outline"
                onPress={onUnstakePress}
                disabled={isSubmitting || (stakingState?.stakedAmount ?? 0) <= 0}
                style={styles.actionBtn}
              />
            </View>
            <View style={{ flex: 1, marginLeft: 8 }}>
              <Button
                title="Claim Rewards"
                variant="secondary"
                onPress={onClaimPress}
                disabled={isSubmitting || (stakingState?.pendingRewards ?? 0) <= 0}
                style={styles.actionBtn}
              />
            </View>
          </View>
        </View>

        {/* Transaction History List */}
        <View style={styles.card}>
          <Text style={styles.cardTitle}>Transaction History</Text>
          {historyLoading ? (
            <ActivityIndicator color={Colors.primaryYellow} style={{ marginVertical: 12 }} />
          ) : history.length === 0 ? (
            <Text style={styles.emptyHistoryText}>No staking transactions yet.</Text>
          ) : (
            history.map((item: StakingTransaction) => (
              <View key={item.id} style={styles.txRow}>
                <View style={styles.txMain}>
                  <Text style={styles.txType}>{item.type.toUpperCase()}</Text>
                  <Text style={styles.txDate}>{formatDate(item.date)}</Text>
                </View>
                <View style={styles.txRight}>
                  <Text style={styles.txAmount}>{item.amount} USDC</Text>
                  <Pressable onPress={() => openExplorer(item.txHash)}>
                    <Text style={styles.txHashLink} numberOfLines={1} ellipsizeMode="middle">
                      {item.txHash.substring(0, 10)}... 🔗
                    </Text>
                  </Pressable>
                </View>
              </View>
            ))
          )}
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
    padding: 16,
  },
  headerContainer: {
    marginBottom: 20,
  },
  headerTitle: {
    fontSize: 24,
    fontWeight: 'bold',
    color: Colors.primaryText,
    marginBottom: 6,
  },
  headerSubtitle: {
    fontSize: 14,
    color: Colors.secondaryText,
    lineHeight: 20,
  },
  card: {
    backgroundColor: '#1E1E20',
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
    borderWidth: 1,
    borderColor: '#2C2C2E',
  },
  cardHeaderRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 16,
  },
  cardTitle: {
    fontSize: 18,
    fontWeight: 'bold',
    color: Colors.primaryText,
    marginBottom: 12,
  },
  badge: {
    paddingHorizontal: 10,
    paddingVertical: 4,
    borderRadius: 12,
  },
  badgeVerified: {
    backgroundColor: '#34C75922',
  },
  badgeUnverified: {
    backgroundColor: '#FF950022',
  },
  badgeText: {
    fontSize: 12,
    fontWeight: 'bold',
  },
  badgeTextVerified: {
    color: '#34C759',
  },
  badgeTextUnverified: {
    color: '#FF9500',
  },
  metricsGrid: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 12,
  },
  metricBox: {
    flex: 1,
    backgroundColor: '#141416',
    padding: 12,
    borderRadius: 8,
    marginRight: 8,
  },
  metricLabel: {
    fontSize: 12,
    color: Colors.secondaryText,
    marginBottom: 4,
  },
  metricValue: {
    fontSize: 16,
    fontWeight: 'bold',
    color: Colors.primaryYellow,
  },
  balanceInfoRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingTop: 8,
    borderTopWidth: 1,
    borderTopColor: '#2C2C2E',
  },
  balanceInfoLabel: {
    fontSize: 13,
    color: Colors.secondaryText,
  },
  balanceInfoValue: {
    fontSize: 14,
    fontWeight: 'bold',
    color: Colors.primaryText,
  },
  configDescription: {
    fontSize: 12,
    color: Colors.secondaryText,
    marginBottom: 12,
  },
  configContainer: {
    backgroundColor: '#141416',
    padding: 12,
    borderRadius: 8,
  },
  configRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 8,
  },
  configLabel: {
    fontSize: 13,
    color: Colors.secondaryText,
  },
  configValue: {
    fontSize: 13,
    color: Colors.primaryText,
    fontFamily: 'SpaceMono',
    maxWidth: '55%',
  },
  feedbackBox: {
    padding: 12,
    borderRadius: 8,
    marginBottom: 16,
  },
  feedbackSuccess: {
    backgroundColor: '#34C75922',
    borderWidth: 1,
    borderColor: '#34C75944',
  },
  feedbackError: {
    backgroundColor: '#FF3B3022',
    borderWidth: 1,
    borderColor: '#FF3B3044',
  },
  feedbackText: {
    fontSize: 14,
  },
  feedbackTextSuccess: {
    color: '#34C759',
  },
  feedbackTextError: {
    color: Colors.accentRed,
  },
  actionBtn: {
    marginTop: 8,
  },
  manageButtonsRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
  },
  emptyHistoryText: {
    color: Colors.secondaryText,
    fontSize: 14,
    textAlign: 'center',
    marginVertical: 12,
  },
  txRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 10,
    borderBottomWidth: 1,
    borderBottomColor: '#2C2C2E',
  },
  txMain: {},
  txType: {
    fontSize: 14,
    fontWeight: 'bold',
    color: Colors.primaryText,
  },
  txDate: {
    fontSize: 12,
    color: Colors.secondaryText,
  },
  txRight: {
    alignItems: 'flex-end',
  },
  txAmount: {
    fontSize: 14,
    fontWeight: 'bold',
    color: Colors.primaryYellow,
  },
  txHashLink: {
    fontSize: 12,
    color: Colors.primaryYellow,
    textDecorationLine: 'underline',
    marginTop: 2,
  },
});
