import React, { useEffect, useState, useCallback } from 'react';
import {
  View,
  Text,
  StyleSheet,
  SafeAreaView,
  ScrollView,
} from 'react-native';
import { useRouter } from 'expo-router';
import * as Clipboard from 'expo-clipboard';
import Colors from '@/constants/Colors';
import Button from '@/components/ui/Button';
import Input from '@/components/ui/Input';
import AvatarEdit from '@/components/AvatarEdit';
import { useAuth } from '@/hooks/useAuth';
import { StellarWalletManager, StellarBalances } from '@/services/stellar';

export default function ProfileScreen() {
  const router = useRouter();
  const { user, logout, updateWalletAddress } = useAuth();
  const [secretInput, setSecretInput] = useState('');
  const [publicKey, setPublicKey] = useState('');
  const [balances, setBalances] = useState<StellarBalances | null>(null);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [loadingBalances, setLoadingBalances] = useState(false);

  const isValidPublicKey = (value: string) => /^G[A-Z2-7]{55}$/.test(value);

  const loadBalances = useCallback(async (address: string) => {
    setLoadingBalances(true);
    setErrorMessage(null);
    setStatusMessage(null);

    try {
      const result = await StellarWalletManager.getBalances(address);
      setBalances(result);

      if (!result.accountExists) {
        setErrorMessage('This account does not exist on the Stellar Testnet.');
      } else if (!result.hasUSDCTrustline) {
        setErrorMessage('USDC trustline is not active for this account.');
      }
    } catch {
      setErrorMessage('Unable to load Stellar balances.');
      setBalances(null);
    } finally {
      setLoadingBalances(false);
    }
  }, []);

  useEffect(() => {
    const restoreKey = async () => {
      const storedSecret = await StellarWalletManager.getSecretKey();
      if (storedSecret) {
        const key = StellarWalletManager.getPublicKeyFromSecret(storedSecret);
        setPublicKey(key);
        updateWalletAddress(key);
      }
    };
    restoreKey();
  }, [updateWalletAddress]);

  useEffect(() => {
    if (publicKey) {
      loadBalances(publicKey);
    }
  }, [publicKey, loadBalances]);

  useEffect(() => {
    if (user?.walletAddress && isValidPublicKey(user.walletAddress)) {
      setPublicKey(user.walletAddress);
    }
  }, [user?.walletAddress]);

  const handleGenerateWallet = async () => {
    const { secretKey, publicKey: newPublicKey } = StellarWalletManager.generateKeypair();

    await StellarWalletManager.saveSecretKey(secretKey);
    setPublicKey(newPublicKey);
    updateWalletAddress(newPublicKey);
    setSecretInput('');
    setStatusMessage('New Stellar wallet generated and stored securely.');
    setErrorMessage(null);
  };

  const handleImportWallet = async () => {
    const trimmedSecret = secretInput.trim();
    if (!trimmedSecret) {
      setErrorMessage('Enter a valid Stellar secret key.');
      return;
    }

    try {
      const { publicKey: importedPublicKey } = await StellarWalletManager.importSecretKey(trimmedSecret);
      setPublicKey(importedPublicKey);
      updateWalletAddress(importedPublicKey);
      setSecretInput('');
      setStatusMessage('Stellar secret key imported securely.');
      setErrorMessage(null);
    } catch {
      setErrorMessage('Invalid Stellar secret key.');
    }
  };

  const handleCopyPublicKey = async () => {
    if (!publicKey) return;
    await Clipboard.setStringAsync(publicKey);
    setStatusMessage('Public key copied to clipboard.');
    setErrorMessage(null);
  };

  const handleAddTrustline = async () => {
    if (!publicKey) return;
    setStatusMessage(null);
    setErrorMessage(null);

    const result = await StellarWalletManager.submitUSDCTrustline();
    if (result.success) {
      setStatusMessage(result.message);
      await loadBalances(publicKey);
    } else {
      setErrorMessage(result.message);
    }
  };

  return (
    <SafeAreaView style={styles.container}>
      <ScrollView contentContainerStyle={styles.content}>
        <View style={styles.profileHeader}>
          <AvatarEdit 
            onImageSelected={(uri) => console.log('New Avatar:', uri)} 
            userName={user?.name ?? undefined}
          />
          <Text style={styles.nameText}>{user?.name || 'Agora User'}</Text>
          <Text style={styles.emailText}>{user?.email || 'user@agora.events'}</Text>
        </View>

        <View style={styles.detailsCard}>
          <Text style={styles.sectionTitle}>Wallet Information</Text>

          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>Stellar Address</Text>
            <Text style={styles.infoValue} numberOfLines={1} ellipsizeMode="middle">
              {publicKey || user?.walletAddress || 'GDAGORA...'}
            </Text>
          </View>

          <View style={styles.balanceRow}>
            <View style={styles.balanceItem}>
              <Text style={styles.balanceLabel}>XLM Balance</Text>
              <Text style={styles.balanceValue}>
                {loadingBalances ? 'Loading...' : balances?.xlmBalance ?? '---'}
              </Text>
            </View>
            <View style={styles.balanceItem}>
              <Text style={styles.balanceLabel}>USDC Balance</Text>
              <Text style={styles.balanceValue}>
                {loadingBalances ? 'Loading...' : balances?.usdcBalance ?? '---'}
              </Text>
            </View>
          </View>

          {errorMessage ? <Text style={styles.warningText}>{errorMessage}</Text> : null}
          {statusMessage ? <Text style={styles.statusText}>{statusMessage}</Text> : null}

          <Input
            label="Import Stellar Secret Key"
            placeholder="S..."
            value={secretInput}
            onChangeText={setSecretInput}
            secureTextEntry
          />

          <Button title="Import Wallet" onPress={handleImportWallet} variant="secondary" style={styles.actionButton} />
          <Button title="Generate New Wallet" onPress={handleGenerateWallet} style={styles.actionButton} />
          <Button title="Copy Public Key" onPress={handleCopyPublicKey} variant="outline" style={styles.actionButton} />

          <Button
            title="Organizer Staking Dashboard"
            onPress={() => router.push('/organizer/staking')}
            variant="primary"
            style={styles.actionButton}
          />

          {balances?.accountExists && !balances.hasUSDCTrustline ? (
            <Button
              title="Add USDC Trustline"
              onPress={handleAddTrustline}
              variant="primary"
              style={styles.actionButton}
            />
          ) : null}
        </View>

        <Button
          title="Log Out"
          onPress={logout}
          variant="outline"
          style={styles.logoutButton}
        />
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
    flex: 1,
    padding: 24,
    justifyContent: 'space-between',
  },
  profileHeader: {
    alignItems: 'center',
    marginTop: 20,
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
  balanceRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginTop: 16,
  },
  balanceItem: {
    flex: 1,
    marginRight: 12,
  },
  balanceItemLast: {
    marginRight: 0,
  },
  balanceLabel: {
    fontSize: 12,
    color: Colors.secondaryText,
    marginBottom: 4,
  },
  balanceValue: {
    fontSize: 16,
    color: Colors.primaryText,
    fontFamily: 'SpaceMono',
  },
  warningText: {
    color: Colors.accentRed,
    fontSize: 13,
    marginTop: 12,
  },
  statusText: {
    color: Colors.primaryYellow,
    fontSize: 13,
    marginTop: 12,
  },
  actionButton: {
    marginTop: 12,
  },
  logoutButton: {
    borderColor: Colors.accentRed,
    color: Colors.accentRed,
    marginBottom: 20,
  },
});
