/**
 * resale.tsx — secondary market screen for a single ticket (issue #1184).
 *
 * Route: `/ticket/resale?paymentId=<on-chain payment id>&eventId=<event id>`
 *
 * The screen covers the seller's whole journey in one place, because the steps
 * are not independent — listing, watching offers, and handing over the ticket
 * key are one obligation, and dropping out halfway leaves a buyer with a
 * ticket they cannot use.
 *
 *   1. **Cap feedback.** The ceiling is read from the contract before the form
 *      is usable, so the seller sees the maximum up front and gets live
 *      validation while typing rather than a rejected transaction later.
 *   2. **Listing management.** List and cancel, mirroring each confirmed
 *      on-chain action to the backend so the listing is discoverable.
 *   3. **Offers.** Buyers publish an X25519 key with their offer. Accepting one
 *      seals this ticket's check-in secret to that key and uploads the
 *      envelope; the server relays ciphertext it cannot read.
 *
 * Settlement itself happens on the buyer's device — `purchase_resale_ticket`
 * moves money and ownership atomically — which is why the seller's "accept"
 * action is a key handover rather than a transfer.
 */

import React, { useCallback, useEffect, useMemo, useState } from 'react';
import {
  ActivityIndicator,
  Alert,
  RefreshControl,
  ScrollView,
  StyleSheet,
  Text,
  View,
} from 'react-native';
import { useLocalSearchParams } from 'expo-router';

import Colors from '@/constants/Colors';
import Button from '@/components/ui/Button';
import Input from '@/components/ui/Input';
import { useAuth } from '@/hooks/useAuth';
import { useSaleNotifications } from '@/hooks/useSaleNotifications';
import {
  fetchMaxResalePrice,
  fetchOnChainListing,
  submitCancelResaleListing,
  submitListForResale,
  type OnChainResaleListing,
} from '@/services/resaleContract';
import {
  fetchOffers,
  publishListing,
  uploadKeyEnvelope,
  withdrawListing,
  type ResaleOffer,
} from '@/services/marketplaceApi';
import { clearTicketSecret, getTicketSecret, sealTicketSecret } from '@/services/resaleCrypto';
import { describePurchaseFailure, stroopsToUsdc, usdcToStroops } from '@/services/ticketPaymentContract';

/** How often the offer book refreshes while the screen is open. */
const OFFER_POLL_INTERVAL_MS = 20_000;

type ScreenState = 'loading' | 'ready' | 'error';

export default function TicketResaleScreen() {
  const { paymentId, eventId } = useLocalSearchParams<{
    paymentId?: string;
    eventId?: string;
  }>();
  const { user } = useAuth();

  const [screenState, setScreenState] = useState<ScreenState>('loading');
  const [loadError, setLoadError] = useState<string | null>(null);

  const [maxPriceStroops, setMaxPriceStroops] = useState<bigint | null>(null);
  const [listing, setListing] = useState<OnChainResaleListing | null>(null);
  const [offers, setOffers] = useState<ResaleOffer[]>([]);

  const [priceInput, setPriceInput] = useState('');
  const [isSubmitting, setSubmitting] = useState(false);
  const [isRefreshing, setRefreshing] = useState(false);
  /** Wallet of the offer currently being sealed, so only its row shows a spinner. */
  const [sealingFor, setSealingFor] = useState<string | null>(null);

  // Ask for push permission on arrival: the seller is about to take on an
  // obligation that only they can discharge.
  const notifications = useSaleNotifications(Boolean(paymentId));

  const isListed = listing?.status === 'Active';

  // ── Loading ───────────────────────────────────────────────────────────────

  const loadChainState = useCallback(async () => {
    if (!paymentId) {
      setLoadError('No ticket was specified for resale.');
      setScreenState('error');
      return;
    }

    try {
      const [cap, existing] = await Promise.all([
        fetchMaxResalePrice(paymentId),
        fetchOnChainListing(paymentId),
      ]);

      setMaxPriceStroops(cap);
      setListing(existing);
      // Prefill with the live listing price, or the cap for a fresh listing —
      // the cap is the useful anchor, since the seller is choosing under it.
      setPriceInput(
        stroopsToUsdc(existing?.status === 'Active' ? existing.price : cap).toFixed(2)
      );
      setScreenState('ready');
      setLoadError(null);
    } catch (error) {
      setLoadError(describePurchaseFailure(error));
      setScreenState('error');
    }
  }, [paymentId]);

  const loadOffers = useCallback(async () => {
    if (!paymentId) return;
    try {
      setOffers(await fetchOffers(paymentId));
    } catch {
      // The offer book is supplementary; a failure here must not take down the
      // listing controls, which are the part the seller actually needs.
    }
  }, [paymentId]);

  useEffect(() => {
    loadChainState();
  }, [loadChainState]);

  useEffect(() => {
    if (!isListed) return;

    loadOffers();
    const timer = setInterval(loadOffers, OFFER_POLL_INTERVAL_MS);
    return () => clearInterval(timer);
  }, [isListed, loadOffers]);

  const onRefresh = useCallback(async () => {
    setRefreshing(true);
    await Promise.all([loadChainState(), loadOffers()]);
    setRefreshing(false);
  }, [loadChainState, loadOffers]);

  // ── Live price validation ─────────────────────────────────────────────────

  /**
   * Validates the typed price against the cap and derives what the seller
   * would actually take home. Recomputed on every keystroke so the feedback
   * tracks the input rather than the last submission.
   */
  const priceFeedback = useMemo(() => {
    if (maxPriceStroops == null) return null;

    const maxUsdc = stroopsToUsdc(maxPriceStroops);
    const parsed = Number(priceInput);

    if (priceInput.trim() === '') {
      return { error: null as string | null, maxUsdc, royaltyUsdc: 0, proceedsUsdc: 0 };
    }
    if (!Number.isFinite(parsed) || parsed <= 0) {
      return {
        error: 'Enter a price greater than 0.',
        maxUsdc,
        royaltyUsdc: 0,
        proceedsUsdc: 0,
      };
    }
    if (usdcToStroops(parsed) > maxPriceStroops) {
      return {
        error: `Above the ${maxUsdc.toFixed(2)} USDC cap for this ticket. The organizer limits resale markup to protect buyers.`,
        maxUsdc,
        royaltyUsdc: 0,
        proceedsUsdc: 0,
      };
    }

    // Mirrors `resale::split_proceeds` — royalty floors, dust to the seller.
    const royaltyBps = listing?.royaltyBps ?? 500;
    const royaltyUsdc = Math.floor((parsed * royaltyBps) / 100) / 100;

    return {
      error: null,
      maxUsdc,
      royaltyUsdc,
      proceedsUsdc: parsed - royaltyUsdc,
    };
  }, [priceInput, maxPriceStroops, listing?.royaltyBps]);

  const canSubmitPrice = Boolean(
    priceFeedback && !priceFeedback.error && priceInput.trim() !== ''
  );

  // ── Actions ───────────────────────────────────────────────────────────────

  const handleList = useCallback(async () => {
    if (!paymentId || !canSubmitPrice) return;

    setSubmitting(true);
    try {
      const { hash, listing: created } = await submitListForResale(
        paymentId,
        Number(priceInput)
      );
      setListing(created);

      // Mirror to the backend so buyers can discover it. The listing is real
      // on-chain either way, so a mirror failure is a discoverability problem,
      // not a failed sale — say so rather than implying the listing failed.
      try {
        await publishListing({
          paymentId,
          eventId: eventId ?? created.eventId,
          priceStroops: created.price,
          maxPriceStroops: created.maxPrice,
          royaltyBps: created.royaltyBps,
          listingTxHash: hash,
        });
        Alert.alert('Listed', 'Your ticket is now on the resale market.');
      } catch (error) {
        Alert.alert(
          'Listed on-chain',
          `Your ticket is listed, but it could not be published to the marketplace feed yet: ${
            error instanceof Error ? error.message : 'unknown error'
          }. Pull to refresh to retry.`
        );
      }

      loadOffers();
    } catch (error) {
      Alert.alert('Listing failed', describePurchaseFailure(error));
    } finally {
      setSubmitting(false);
    }
  }, [paymentId, eventId, priceInput, canSubmitPrice, loadOffers]);

  const handleCancel = useCallback(async () => {
    if (!paymentId) return;

    setSubmitting(true);
    try {
      await submitCancelResaleListing(paymentId);
      try {
        await withdrawListing(paymentId);
      } catch {
        // Same reasoning as listing: the chain is authoritative. A stale feed
        // entry resolves on the next mirror.
      }
      await loadChainState();
      setOffers([]);
      Alert.alert('Unlisted', 'Your ticket has been taken off the resale market.');
    } catch (error) {
      Alert.alert('Could not unlist', describePurchaseFailure(error));
    } finally {
      setSubmitting(false);
    }
  }, [paymentId, loadChainState]);

  /**
   * Seals this ticket's check-in secret to the chosen buyer and uploads it.
   *
   * Only meaningful once that buyer has settled on-chain — until then the
   * seller still owns the ticket. The confirmation spells that out, since
   * sending the key to someone who has not paid gives away the ticket.
   */
  const handleSendKey = useCallback(
    async (offer: ResaleOffer) => {
      if (!paymentId) return;

      const secret = await getTicketSecret(paymentId);
      if (!secret) {
        Alert.alert(
          'Ticket key unavailable',
          'This device does not hold the check-in key for this ticket, so the handover cannot be completed here. Use the device you originally bought the ticket on.'
        );
        return;
      }

      Alert.alert(
        'Send ticket key?',
        `This hands the check-in key to ${shortenWallet(offer.buyer_wallet)}. Only do this once their purchase has settled on-chain — afterwards they can enter the event with this ticket.`,
        [
          { text: 'Cancel', style: 'cancel' },
          {
            text: 'Send key',
            style: 'destructive',
            onPress: async () => {
              setSealingFor(offer.buyer_wallet);
              try {
                const envelope = sealTicketSecret(secret, offer.buyer_public_key);
                await uploadKeyEnvelope(paymentId, offer.buyer_wallet, envelope);

                // The ticket is theirs now; keeping our copy only widens the
                // window in which it could leak from this device.
                await clearTicketSecret(paymentId);

                await loadChainState();
                await loadOffers();
                Alert.alert('Key sent', 'The buyer can now check in with this ticket.');
              } catch (error) {
                Alert.alert(
                  'Could not send key',
                  error instanceof Error ? error.message : 'Unknown error.'
                );
              } finally {
                setSealingFor(null);
              }
            },
          },
        ]
      );
    },
    [paymentId, loadChainState, loadOffers]
  );

  // ── Render ────────────────────────────────────────────────────────────────

  if (screenState === 'loading') {
    return (
      <View style={styles.centered}>
        <ActivityIndicator size="large" color={Colors.primaryYellow} />
        <Text style={styles.mutedText}>Checking the resale price cap…</Text>
      </View>
    );
  }

  if (screenState === 'error') {
    return (
      <View style={styles.centered}>
        <Text style={styles.errorText}>{loadError}</Text>
        <Button title="Try again" onPress={loadChainState} style={styles.fullWidthButton} />
      </View>
    );
  }

  return (
    <ScrollView
      style={styles.container}
      contentContainerStyle={styles.content}
      refreshControl={
        <RefreshControl
          refreshing={isRefreshing}
          onRefresh={onRefresh}
          tintColor={Colors.primaryYellow}
        />
      }
    >
      <Text style={styles.header}>Resell Ticket</Text>
      <Text style={styles.subheader}>{paymentId}</Text>

      {/* Cap panel — the anti-scalping rule, stated before the seller types. */}
      <View style={styles.capCard}>
        <Text style={styles.capLabel}>Maximum resale price</Text>
        <Text style={styles.capValue}>
          {priceFeedback ? `${priceFeedback.maxUsdc.toFixed(2)} USDC` : '—'}
        </Text>
        <Text style={styles.capNote}>
          Enforced on-chain. The organizer caps how far above face value a ticket can be
          resold, and takes a {((listing?.royaltyBps ?? 500) / 100).toFixed(1)}% royalty on
          each sale.
        </Text>
      </View>

      {isListed ? (
        <View style={styles.statusCard}>
          <Text style={styles.statusBadge}>Listed</Text>
          <Text style={styles.statusPrice}>
            {stroopsToUsdc(listing!.price).toFixed(2)} USDC
          </Text>
          <Text style={styles.mutedText}>
            Waiting for a buyer. You will be notified when it sells
            {notifications.token ? '' : ' — enable notifications so you do not miss it'}.
          </Text>
          <Button
            title="Remove listing"
            variant="outline"
            onPress={handleCancel}
            loading={isSubmitting}
            style={styles.fullWidthButton}
          />
        </View>
      ) : (
        <View style={styles.formCard}>
          <Input
            label="Asking price (USDC)"
            placeholder="0.00"
            keyboardType="decimal-pad"
            value={priceInput}
            onChangeText={setPriceInput}
            error={priceFeedback?.error ?? undefined}
          />

          {priceFeedback && !priceFeedback.error && priceInput.trim() !== '' && (
            <View style={styles.breakdown}>
              <BreakdownRow
                label="Organizer royalty"
                value={`−${priceFeedback.royaltyUsdc.toFixed(2)} USDC`}
              />
              <BreakdownRow
                label="You receive"
                value={`${priceFeedback.proceedsUsdc.toFixed(2)} USDC`}
                emphasis
              />
            </View>
          )}

          <Button
            title="List for resale"
            onPress={handleList}
            disabled={!canSubmitPrice}
            loading={isSubmitting}
            style={styles.fullWidthButton}
          />
        </View>
      )}

      {/* Offers */}
      {isListed && (
        <View style={styles.offersSection}>
          <Text style={styles.sectionTitle}>Offers ({offers.length})</Text>

          {offers.length === 0 ? (
            <Text style={styles.mutedText}>
              No offers yet. Buyers who are interested will appear here.
            </Text>
          ) : (
            offers.map((offer) => (
              <View key={offer.id} style={styles.offerRow}>
                <View style={styles.offerInfo}>
                  <Text style={styles.offerWallet}>{shortenWallet(offer.buyer_wallet)}</Text>
                  <Text style={styles.offerPrice}>
                    {stroopsToUsdc(BigInt(offer.offer_price_stroops)).toFixed(2)} USDC
                  </Text>
                  <Text style={styles.offerStatus}>{offer.status}</Text>
                </View>
                {offer.status === 'accepted' ? (
                  <Text style={styles.offerDone}>Key sent</Text>
                ) : (
                  <Button
                    title="Send key"
                    onPress={() => handleSendKey(offer)}
                    loading={sealingFor === offer.buyer_wallet}
                    disabled={sealingFor != null}
                    style={styles.offerButton}
                  />
                )}
              </View>
            ))
          )}

          <Text style={styles.helpText}>
            The buyer pays and receives ownership in a single on-chain transaction. Sending
            the key is the last step — it is encrypted to that buyer alone and nobody in
            between, including Agora, can read it.
          </Text>
        </View>
      )}

      {user?.walletAddress ? (
        <Text style={styles.footerNote}>Selling as {shortenWallet(user.walletAddress)}</Text>
      ) : null}
    </ScrollView>
  );
}

function BreakdownRow({
  label,
  value,
  emphasis = false,
}: {
  label: string;
  value: string;
  emphasis?: boolean;
}) {
  return (
    <View style={styles.breakdownRow}>
      <Text style={[styles.breakdownLabel, emphasis && styles.breakdownEmphasis]}>
        {label}
      </Text>
      <Text style={[styles.breakdownValue, emphasis && styles.breakdownEmphasis]}>
        {value}
      </Text>
    </View>
  );
}

/** Renders a Stellar address as `GABC…WXYZ` for compact display. */
function shortenWallet(wallet: string): string {
  if (wallet.length <= 12) return wallet;
  return `${wallet.slice(0, 4)}…${wallet.slice(-4)}`;
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: Colors.darkBackground,
  },
  content: {
    padding: 16,
    paddingBottom: 48,
  },
  centered: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    padding: 24,
    backgroundColor: Colors.darkBackground,
  },
  header: {
    fontSize: 24,
    fontWeight: 'bold',
    color: Colors.primaryText,
  },
  subheader: {
    fontSize: 12,
    color: Colors.secondaryText,
    fontFamily: 'SpaceMono',
    marginTop: 4,
    marginBottom: 20,
  },
  capCard: {
    backgroundColor: '#1E1E20',
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
    borderLeftWidth: 4,
    borderLeftColor: Colors.primaryYellow,
    borderWidth: 1,
    borderColor: '#2C2C2E',
  },
  capLabel: {
    fontSize: 12,
    color: Colors.secondaryText,
    marginBottom: 4,
  },
  capValue: {
    fontSize: 26,
    fontWeight: 'bold',
    color: Colors.primaryYellow,
    marginBottom: 8,
  },
  capNote: {
    fontSize: 12,
    color: Colors.secondaryText,
    lineHeight: 18,
  },
  formCard: {
    backgroundColor: '#1E1E20',
    borderRadius: 12,
    padding: 16,
    marginBottom: 24,
    borderWidth: 1,
    borderColor: '#2C2C2E',
  },
  statusCard: {
    backgroundColor: '#1E1E20',
    borderRadius: 12,
    padding: 16,
    marginBottom: 24,
    borderWidth: 1,
    borderColor: '#2C2C2E',
    alignItems: 'flex-start',
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
    marginBottom: 8,
  },
  statusPrice: {
    fontSize: 22,
    fontWeight: 'bold',
    color: Colors.primaryText,
    marginBottom: 8,
  },
  breakdown: {
    borderTopWidth: 1,
    borderTopColor: '#2C2C2E',
    paddingTop: 12,
    marginBottom: 16,
  },
  breakdownRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 6,
  },
  breakdownLabel: {
    fontSize: 13,
    color: Colors.secondaryText,
  },
  breakdownValue: {
    fontSize: 13,
    color: Colors.primaryText,
  },
  breakdownEmphasis: {
    color: Colors.primaryYellow,
    fontWeight: 'bold',
  },
  offersSection: {
    marginTop: 8,
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: 'bold',
    color: Colors.primaryText,
    marginBottom: 12,
  },
  offerRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    backgroundColor: '#1E1E20',
    borderRadius: 12,
    padding: 12,
    marginBottom: 8,
    borderWidth: 1,
    borderColor: '#2C2C2E',
  },
  offerInfo: {
    flex: 1,
    marginRight: 12,
  },
  offerWallet: {
    fontSize: 13,
    color: Colors.primaryText,
    fontFamily: 'SpaceMono',
  },
  offerPrice: {
    fontSize: 16,
    fontWeight: 'bold',
    color: Colors.primaryYellow,
    marginTop: 2,
  },
  offerStatus: {
    fontSize: 11,
    color: Colors.secondaryText,
    marginTop: 2,
  },
  offerButton: {
    paddingHorizontal: 16,
  },
  offerDone: {
    fontSize: 12,
    color: '#34C759',
    fontWeight: 'bold',
  },
  helpText: {
    fontSize: 12,
    color: Colors.secondaryText,
    lineHeight: 18,
    marginTop: 12,
  },
  mutedText: {
    fontSize: 13,
    color: Colors.secondaryText,
    lineHeight: 20,
    marginBottom: 12,
  },
  errorText: {
    fontSize: 14,
    color: Colors.accentRed,
    textAlign: 'center',
    marginBottom: 16,
  },
  fullWidthButton: {
    width: '100%',
  },
  footerNote: {
    fontSize: 11,
    color: Colors.secondaryText,
    textAlign: 'center',
    marginTop: 24,
  },
});
