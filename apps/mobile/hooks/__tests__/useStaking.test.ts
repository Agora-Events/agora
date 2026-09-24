import { act, renderHook, waitFor } from '@testing-library/react-native';
import { useStaking } from '../useStaking';
import {
  claimRewards,
  fetchOrganizerStakingState,
  fetchStakingConfig,
  fetchStakingHistory,
  stakeCollateral,
  unstakeCollateral,
} from '@/services/staking';
import { StellarWalletManager } from '@/services/stellar';

/**
 * Issue #1464 — useStaking.
 *
 * `services/staking.ts` (Soroban calls) and the Stellar wallet balance lookup
 * are mocked; these tests cover the hook's own orchestration — loading flags,
 * state/balance updates, and the refresh that follows a successful action.
 */

jest.mock('@/services/staking', () => ({
  fetchStakingConfig: jest.fn(),
  fetchOrganizerStakingState: jest.fn(),
  fetchStakingHistory: jest.fn(),
  stakeCollateral: jest.fn(),
  unstakeCollateral: jest.fn(),
  claimRewards: jest.fn(),
}));

jest.mock('@/services/stellar', () => ({
  StellarWalletManager: { getBalances: jest.fn() },
}));

const mockFetchConfig = fetchStakingConfig as jest.MockedFunction<typeof fetchStakingConfig>;
const mockFetchState = fetchOrganizerStakingState as jest.MockedFunction<
  typeof fetchOrganizerStakingState
>;
const mockFetchHistory = fetchStakingHistory as jest.MockedFunction<typeof fetchStakingHistory>;
const mockStake = stakeCollateral as jest.MockedFunction<typeof stakeCollateral>;
const mockUnstake = unstakeCollateral as jest.MockedFunction<typeof unstakeCollateral>;
const mockClaim = claimRewards as jest.MockedFunction<typeof claimRewards>;
const mockGetBalances = StellarWalletManager.getBalances as jest.MockedFunction<
  typeof StellarWalletManager.getBalances
>;

const PUBLIC_KEY = 'GA7QYNF7SOWQ3GLR2BGMZEHXAVIRZA4KVWLTJJFC7MGXUA74P7UJVSGZ';

const unstakedState = {
  status: 'Unverified' as const,
  stakedAmount: 0,
  pendingRewards: 0,
  lockupEnds: null,
};

const stakedState = {
  status: 'Verified' as const,
  stakedAmount: 100,
  pendingRewards: 0,
  lockupEnds: '2026-12-31T00:00:00Z',
};

const stakeTx = {
  id: 'tx-1',
  type: 'stake' as const,
  amount: 100,
  date: '2026-09-24T00:00:00Z',
  txHash: 'hash-1',
};

beforeEach(() => {
  jest.clearAllMocks();
  mockFetchConfig.mockResolvedValue({ tokenAddress: 'CUSDC', minimumStake: 100 });
  mockFetchState.mockResolvedValue(unstakedState);
  mockFetchHistory.mockResolvedValue([]);
  mockGetBalances.mockResolvedValue({ usdcBalance: '250.50' } as any);
});

describe('useStaking', () => {
  it('loads config, organizer state, history and USDC balance on mount', async () => {
    const { result } = renderHook(() => useStaking(PUBLIC_KEY));

    expect(result.current.configLoading).toBe(true);

    await waitFor(() => expect(result.current.configLoading).toBe(false));
    await waitFor(() => expect(result.current.stateLoading).toBe(false));

    expect(result.current.config).toEqual({ tokenAddress: 'CUSDC', minimumStake: 100 });
    expect(result.current.stakingState).toEqual(unstakedState);
    expect(result.current.history).toEqual([]);
    expect(result.current.usdcBalance).toBe(250.5);
    expect(mockFetchState).toHaveBeenCalledWith(PUBLIC_KEY);
    expect(mockFetchHistory).toHaveBeenCalledWith(PUBLIC_KEY);
  });

  it('skips organizer and balance fetches when there is no public key', async () => {
    const { result } = renderHook(() => useStaking(null));

    await waitFor(() => expect(result.current.configLoading).toBe(false));

    expect(mockFetchState).not.toHaveBeenCalled();
    expect(mockGetBalances).not.toHaveBeenCalled();
    expect(result.current.stakingState).toBeNull();
    expect(result.current.usdcBalance).toBe(0);
  });

  it('falls back to a zero balance when the wallet lookup fails', async () => {
    mockGetBalances.mockRejectedValue(new Error('horizon down'));

    const { result } = renderHook(() => useStaking(PUBLIC_KEY));

    await waitFor(() => expect(mockGetBalances).toHaveBeenCalled());
    expect(result.current.usdcBalance).toBe(0);
  });

  it('stakes, toggles loading during the refresh, and updates state and balance', async () => {
    const { result } = renderHook(() => useStaking(PUBLIC_KEY));
    await waitFor(() => expect(result.current.stateLoading).toBe(false));

    mockStake.mockResolvedValue({ success: true, txHash: 'hash-1', message: 'Staked' });

    // Hold the post-stake refresh open so the loading flag can be observed.
    let resolveState: (value: typeof stakedState) => void = () => {};
    mockFetchState.mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          resolveState = resolve;
        })
    );
    mockFetchHistory.mockResolvedValue([stakeTx]);
    mockGetBalances.mockResolvedValue({ usdcBalance: '150.50' } as any);

    let stakeResult: Awaited<ReturnType<typeof result.current.handleStake>> | undefined;
    await act(async () => {
      stakeResult = await result.current.handleStake(100);
    });

    expect(mockStake).toHaveBeenCalledWith(100);
    expect(stakeResult).toEqual({ success: true, txHash: 'hash-1', message: 'Staked' });
    expect(result.current.stateLoading).toBe(true);
    expect(result.current.historyLoading).toBe(true);

    await act(async () => {
      resolveState(stakedState);
    });

    await waitFor(() => expect(result.current.stateLoading).toBe(false));
    expect(result.current.historyLoading).toBe(false);
    expect(result.current.stakingState).toEqual(stakedState);
    expect(result.current.history).toEqual([stakeTx]);
    expect(result.current.usdcBalance).toBe(150.5);
  });

  it('does not refresh when the staking service reports a failed transaction', async () => {
    const { result } = renderHook(() => useStaking(PUBLIC_KEY));
    await waitFor(() => expect(result.current.stateLoading).toBe(false));
    mockFetchState.mockClear();
    mockGetBalances.mockClear();

    mockStake.mockResolvedValue({ success: false, message: 'Insufficient USDC balance' });

    let stakeResult: Awaited<ReturnType<typeof result.current.handleStake>> | undefined;
    await act(async () => {
      stakeResult = await result.current.handleStake(10_000);
    });

    expect(stakeResult).toEqual({ success: false, message: 'Insufficient USDC balance' });
    expect(mockFetchState).not.toHaveBeenCalled();
    expect(mockGetBalances).not.toHaveBeenCalled();
    expect(result.current.stateLoading).toBe(false);
    expect(result.current.stakingState).toEqual(unstakedState);
  });

  it('propagates a rejected stake call to the caller and leaves loading reset', async () => {
    const { result } = renderHook(() => useStaking(PUBLIC_KEY));
    await waitFor(() => expect(result.current.stateLoading).toBe(false));
    mockFetchState.mockClear();

    mockStake.mockRejectedValue(new Error('wallet signature rejected'));

    await act(async () => {
      await expect(result.current.handleStake(100)).rejects.toThrow('wallet signature rejected');
    });

    expect(mockFetchState).not.toHaveBeenCalled();
    expect(result.current.stateLoading).toBe(false);
    expect(result.current.historyLoading).toBe(false);
    expect(result.current.stakingState).toEqual(unstakedState);
  });

  it('refreshes after a successful unstake and claim', async () => {
    const { result } = renderHook(() => useStaking(PUBLIC_KEY));
    await waitFor(() => expect(result.current.stateLoading).toBe(false));
    mockFetchState.mockClear();

    mockUnstake.mockResolvedValue({ success: true, txHash: 'hash-2', message: 'Unstaked' });
    mockClaim.mockResolvedValue({ success: true, txHash: 'hash-3', message: 'Claimed' });

    await act(async () => {
      await result.current.handleUnstake();
    });
    await waitFor(() => expect(mockFetchState).toHaveBeenCalledTimes(1));

    await act(async () => {
      await result.current.handleClaim();
    });
    await waitFor(() => expect(mockFetchState).toHaveBeenCalledTimes(2));

    expect(mockUnstake).toHaveBeenCalledTimes(1);
    expect(mockClaim).toHaveBeenCalledTimes(1);
  });
});
