import { Address, Keypair, scValToNative } from '@stellar/stellar-sdk';
import { buildAndSimulateContractCall, loadSorobanAccount } from '../sorobanClient';
import { StellarWalletManager } from '../stellar';
import {
  EVENT_REGISTRY_CONTRACT,
  fetchOrganizerStakingState,
  fetchStakingConfig,
} from '../staking';

jest.mock('@stellar/stellar-sdk', () => {
  const actual = jest.requireActual('@stellar/stellar-sdk');
  return {
    ...actual,
    Contract: jest.fn().mockImplementation((contractId: string) => ({
      call: (method: string, ...args: unknown[]) => ({ contractId, method, args }),
    })),
    scValToNative: jest.fn(),
  };
});

jest.mock('../sorobanClient', () => ({
  buildAndSimulateContractCall: jest.fn(async ({ operation }) => ({
    simulation: { result: { retval: operation } },
  })),
  loadSorobanAccount: jest.fn().mockResolvedValue({}),
}));

jest.mock('../stellar', () => ({
  StellarWalletManager: { getSecretKey: jest.fn() },
}));

const mockScValToNative = scValToNative as jest.MockedFunction<typeof scValToNative>;
const mockBuildAndSimulate = buildAndSimulateContractCall as jest.MockedFunction<
  typeof buildAndSimulateContractCall
>;
const mockLoadAccount = loadSorobanAccount as jest.MockedFunction<typeof loadSorobanAccount>;

const PUBLIC_KEY = 'GA7QYNF7SOWQ3GLR2BGMZEHXAVIRZA4KVWLTJJFC7MGXUA74P7UJVSGZ';
const SECRET_KEY = Keypair.random().secret();

beforeEach(() => {
  jest.clearAllMocks();
});

describe('staking Soroban reads', () => {
  it('reads staking configuration from the event registry', async () => {
    mockScValToNative.mockReturnValue({
      token: 'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM',
      min_amount: 5_000_000_000n,
    } as never);

    await expect(fetchStakingConfig(PUBLIC_KEY)).resolves.toEqual({
      tokenAddress: 'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM',
      minimumStake: 500,
    });

    expect(mockLoadAccount).toHaveBeenCalledWith(PUBLIC_KEY);
    expect(mockBuildAndSimulate).toHaveBeenCalledWith(
      expect.objectContaining({
        operation: expect.objectContaining({
          method: 'get_staking_config',
          args: [],
        }),
      })
    );
  });

  it('reads the organizer stake and converts token base units', async () => {
    mockScValToNative.mockReturnValue({
      amount: 1_250_000_000n,
      is_verified: true,
      reward_balance: 25_000_000n,
    } as never);

    await expect(fetchOrganizerStakingState(PUBLIC_KEY)).resolves.toEqual({
      status: 'Verified',
      stakedAmount: 125,
      pendingRewards: 2.5,
      lockupEnds: null,
    });

    const operation = mockBuildAndSimulate.mock.calls[0][0].operation;
    expect(operation).toEqual(
      expect.objectContaining({
        method: 'get_organizer_stake',
        args: [expect.anything()],
      })
    );
    expect(Address.fromString(PUBLIC_KEY).toString()).toBe(PUBLIC_KEY);
  });

  it('returns an unverified state when the contract has no stake record', async () => {
    mockScValToNative.mockReturnValue(null as never);

    await expect(fetchOrganizerStakingState(PUBLIC_KEY)).resolves.toEqual({
      status: 'Unverified',
      stakedAmount: 0,
      pendingRewards: 0,
      lockupEnds: null,
    });
  });

  it('uses the configured event registry contract', async () => {
    mockScValToNative.mockReturnValue({
      token: 'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM',
      min_amount: 1n,
    } as never);

    await fetchStakingConfig(PUBLIC_KEY);

    expect(mockBuildAndSimulate.mock.calls[0][0].operation).toEqual(
      expect.objectContaining({ contractId: EVENT_REGISTRY_CONTRACT })
    );
  });

  it('does not invent configuration when the contract is unconfigured', async () => {
    mockScValToNative.mockReturnValue({ token: null, min_amount: 0n } as never);

    await expect(fetchStakingConfig(PUBLIC_KEY)).rejects.toThrow(
      'Staking has not been configured in the event registry.'
    );
  });

  it('does not query organizer state without an address', async () => {
    await expect(fetchOrganizerStakingState()).resolves.toEqual({
      status: 'Unverified',
      stakedAmount: 0,
      pendingRewards: 0,
      lockupEnds: null,
    });
    expect(mockBuildAndSimulate).not.toHaveBeenCalled();
  });

  it('connects the config read to the service wallet when no key is supplied', async () => {
    (StellarWalletManager.getSecretKey as jest.Mock).mockResolvedValueOnce(
      SECRET_KEY
    );
    mockScValToNative.mockReturnValue({
      token: 'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM',
      min_amount: 1n,
    } as never);

    await expect(fetchStakingConfig()).resolves.toMatchObject({
      tokenAddress: 'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM',
    });
  });
});
