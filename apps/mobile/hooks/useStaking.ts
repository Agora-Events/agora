/**
 * useStaking — React hook encapsulating all organizer staking state and actions.
 *
 * Automatically refreshes staking metrics after every successful transaction
 * and exposes helpers for stake, unstake, and claim operations.
 */

import { useState, useEffect, useCallback } from 'react';
import {
  StakingConfig,
  OrganizerStakingState,
  StakingTransaction,
  StakingResult,
  fetchStakingConfig,
  fetchOrganizerStakingState,
  fetchStakingHistory,
  stakeCollateral,
  unstakeCollateral,
  claimRewards,
} from '@/services/staking';
import { StellarWalletManager } from '@/services/stellar';

export interface UseStakingReturn {
  // Config
  config: StakingConfig | null;
  configLoading: boolean;
  configError: string | null;

  // Organizer state
  stakingState: OrganizerStakingState | null;
  stateLoading: boolean;
  stateError: string | null;

  // History
  history: StakingTransaction[];
  historyLoading: boolean;

  // USDC wallet balance (for stake button guard)
  usdcBalance: number;

  // Actions
  handleStake: (amount: number) => Promise<StakingResult>;
  handleUnstake: () => Promise<StakingResult>;
  handleClaim: () => Promise<StakingResult>;

  // Manual refresh
  refresh: () => void;
}

export function useStaking(publicKey: string | null): UseStakingReturn {
  const [config, setConfig] = useState<StakingConfig | null>(null);
  const [configLoading, setConfigLoading] = useState(true);
  const [configError, setConfigError] = useState<string | null>(null);

  const [stakingState, setStakingState] = useState<OrganizerStakingState | null>(null);
  const [stateLoading, setStateLoading] = useState(false);
  const [stateError, setStateError] = useState<string | null>(null);

  const [history, setHistory] = useState<StakingTransaction[]>([]);
  const [historyLoading, setHistoryLoading] = useState(false);

  const [usdcBalance, setUsdcBalance] = useState(0);

  // ── Fetch staking config once ────────────────────────────────────────────
  useEffect(() => {
    let cancelled = false;
    setConfigError(null);
    setConfigLoading(true);
    (async () => {
      try {
        const cfg = await fetchStakingConfig(publicKey ?? undefined);
        if (!cancelled) setConfig(cfg);
      } catch (error) {
        if (!cancelled) {
          setConfigError(
            error instanceof Error ? error.message : 'Failed to load staking configuration.'
          );
        }
      } finally {
        if (!cancelled) setConfigLoading(false);
      }
    })();
    return () => { cancelled = true; };
  }, [publicKey]);

  // ── Fetch organizer state + history whenever publicKey changes ───────────
  const loadOrganizerData = useCallback(async () => {
    if (!publicKey) return;

    setStateLoading(true);
    setHistoryLoading(true);
    setStateError(null);

    try {
      const [state, txHistory] = await Promise.all([
        fetchOrganizerStakingState(publicKey),
        fetchStakingHistory(publicKey),
      ]);
      setStakingState(state);
      setHistory(txHistory);
    } catch (error) {
      setStateError(
        error instanceof Error ? error.message : 'Failed to load organizer staking data.'
      );
    } finally {
      setStateLoading(false);
      setHistoryLoading(false);
    }
  }, [publicKey]);

  // ── Fetch USDC balance ────────────────────────────────────────────────────
  const loadBalance = useCallback(async () => {
    if (!publicKey) return;
    try {
      const balances = await StellarWalletManager.getBalances(publicKey);
      setUsdcBalance(parseFloat(balances.usdcBalance) || 0);
    } catch {
      setUsdcBalance(0);
    }
  }, [publicKey]);

  useEffect(() => {
    loadOrganizerData();
    loadBalance();
  }, [loadOrganizerData, loadBalance]);

  const refresh = useCallback(() => {
    loadOrganizerData();
    loadBalance();
  }, [loadOrganizerData, loadBalance]);

  // ── Action wrappers — refresh on success ─────────────────────────────────
  const handleStake = useCallback(async (amount: number): Promise<StakingResult> => {
    const result = await stakeCollateral(amount);
    if (result.success) refresh();
    return result;
  }, [refresh]);

  const handleUnstake = useCallback(async (): Promise<StakingResult> => {
    const result = await unstakeCollateral();
    if (result.success) refresh();
    return result;
  }, [refresh]);

  const handleClaim = useCallback(async (): Promise<StakingResult> => {
    const result = await claimRewards();
    if (result.success) refresh();
    return result;
  }, [refresh]);

  return {
    config,
    configLoading,
    configError,
    stakingState,
    stateLoading,
    stateError,
    history,
    historyLoading,
    usdcBalance,
    handleStake,
    handleUnstake,
    handleClaim,
    refresh,
  };
}
