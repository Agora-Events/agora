import { Account, Keypair, rpc } from '@stellar/stellar-sdk';
import {
  __resetSorobanServerForTests,
  delay,
  extractContractErrorCode,
  getLatestLedgerSequence,
  loadSorobanAccount,
  pollTransactionUntilFinal,
  signTransaction,
  SorobanTransactionError,
  submitTransaction,
} from '../sorobanClient';

describe('extractContractErrorCode', () => {
  it('extracts the code from the standard soroban-sdk panic format', () => {
    expect(extractContractErrorCode('HostError: Error(Contract, #12)')).toBe(12);
  });

  it('extracts the code from an Error instance message', () => {
    expect(extractContractErrorCode(new Error('simulation failed: Error(Contract, #6)'))).toBe(6);
  });

  it('extracts the code from a "ContractError(N)" style message', () => {
    expect(extractContractErrorCode('status Ok(ContractError(27))')).toBe(27);
  });

  it('extracts the code out of a nested response object', () => {
    expect(extractContractErrorCode({ error: 'Error(Contract, #42)', latestLedger: 100 })).toBe(42);
  });

  it('returns null when no error code can be found', () => {
    expect(extractContractErrorCode('a plain network timeout')).toBeNull();
    expect(extractContractErrorCode(null)).toBeNull();
    expect(extractContractErrorCode(undefined)).toBeNull();
    expect(extractContractErrorCode({})).toBeNull();
  });
});

describe('delay', () => {
  it('resolves after roughly the requested duration', async () => {
    const start = Date.now();
    await delay(20);
    expect(Date.now() - start).toBeGreaterThanOrEqual(15);
  });
});

describe('SorobanTransactionError', () => {
  it('carries stage, contract error code, tx hash, and cause through to the instance', () => {
    const cause = { some: 'raw-response' };
    const error = new SorobanTransactionError({
      stage: 'confirm',
      message: 'boom',
      contractErrorCode: 12,
      cause,
      txHash: 'abcd',
    });

    expect(error).toBeInstanceOf(Error);
    expect(error.name).toBe('SorobanTransactionError');
    expect(error.stage).toBe('confirm');
    expect(error.contractErrorCode).toBe(12);
    expect(error.txHash).toBe('abcd');
    expect(error.cause).toBe(cause);
    expect(error.message).toBe('boom');
  });

  it('defaults optional fields to null when omitted', () => {
    const error = new SorobanTransactionError({ stage: 'sign', message: 'nope' });
    expect(error.contractErrorCode).toBeNull();
    expect(error.txHash).toBeNull();
    expect(error.cause).toBeNull();
  });
});

describe('pollTransactionUntilFinal', () => {
  afterEach(() => {
    jest.restoreAllMocks();
    __resetSorobanServerForTests();
  });

  it('resolves once getTransaction reports SUCCESS, retrying through NOT_FOUND first', async () => {
    const calls: string[] = [];
    jest.spyOn(rpc.Server.prototype, 'getTransaction').mockImplementation(async () => {
      calls.push('call');
      if (calls.length < 3) {
        return { status: rpc.Api.GetTransactionStatus.NOT_FOUND } as any;
      }
      return { status: rpc.Api.GetTransactionStatus.SUCCESS } as any;
    });

    const onAttempt = jest.fn();
    const result = await pollTransactionUntilFinal('hash-success', {
      intervalMs: 1,
      maxAttempts: 10,
      onAttempt,
    });

    expect(result.status).toBe(rpc.Api.GetTransactionStatus.SUCCESS);
    expect(calls.length).toBe(3);
    expect(onAttempt).toHaveBeenCalledWith(1, 10);
  });

  it('throws a SorobanTransactionError with stage "confirm" and a decoded error code on FAILED', async () => {
    jest.spyOn(rpc.Server.prototype, 'getTransaction').mockResolvedValue({
      status: rpc.Api.GetTransactionStatus.FAILED,
      resultXdr: 'Error(Contract, #7)',
    } as any);

    await expect(
      pollTransactionUntilFinal('hash-failed', { intervalMs: 1, maxAttempts: 3 })
    ).rejects.toMatchObject({
      stage: 'confirm',
      contractErrorCode: 7,
      txHash: 'hash-failed',
    });
  });

  it('throws a SorobanTransactionError with stage "timeout" once maxAttempts is exhausted', async () => {
    jest
      .spyOn(rpc.Server.prototype, 'getTransaction')
      .mockResolvedValue({ status: rpc.Api.GetTransactionStatus.NOT_FOUND } as any);

    await expect(
      pollTransactionUntilFinal('hash-timeout', { intervalMs: 1, maxAttempts: 2 })
    ).rejects.toMatchObject({ stage: 'timeout', txHash: 'hash-timeout' });
  });

  it('tolerates a transient network error while polling instead of failing immediately', async () => {
    let call = 0;
    jest.spyOn(rpc.Server.prototype, 'getTransaction').mockImplementation(async () => {
      call += 1;
      if (call === 1) throw new Error('ECONNRESET');
      return { status: rpc.Api.GetTransactionStatus.SUCCESS } as any;
    });

    const result = await pollTransactionUntilFinal('hash-flaky', { intervalMs: 1, maxAttempts: 5 });
    expect(result.status).toBe(rpc.Api.GetTransactionStatus.SUCCESS);
    expect(call).toBe(2);
  });
});

const BUYER_PUBLIC_KEY = Keypair.random().publicKey();

describe('loadSorobanAccount', () => {
  afterEach(() => {
    jest.restoreAllMocks();
    __resetSorobanServerForTests();
  });

  it('returns the account loaded from the RPC server', async () => {
    const account = new Account(BUYER_PUBLIC_KEY, '100');
    jest.spyOn(rpc.Server.prototype, 'getAccount').mockResolvedValue(account);

    const result = await loadSorobanAccount(BUYER_PUBLIC_KEY);
    expect(result).toBe(account);
  });

  it('wraps a failure to load the account in a SorobanTransactionError with stage "load-account"', async () => {
    jest.spyOn(rpc.Server.prototype, 'getAccount').mockRejectedValue(new Error('account not found'));

    await expect(loadSorobanAccount(BUYER_PUBLIC_KEY)).rejects.toMatchObject({
      name: 'SorobanTransactionError',
      stage: 'load-account',
    });
  });
});

describe('getLatestLedgerSequence', () => {
  afterEach(() => {
    jest.restoreAllMocks();
    __resetSorobanServerForTests();
  });

  it('returns the ledger sequence number reported by the RPC server', async () => {
    jest
      .spyOn(rpc.Server.prototype, 'getLatestLedger')
      .mockResolvedValue({ id: 'abc', sequence: 123456, protocolVersion: 21 } as any);

    await expect(getLatestLedgerSequence()).resolves.toBe(123456);
  });
});

describe('signTransaction', () => {
  it('signs the transaction with the given keypair and returns it', () => {
    const keypair = Keypair.random();
    const fakeTx = { sign: jest.fn() } as any;

    const result = signTransaction(fakeTx, keypair);

    expect(fakeTx.sign).toHaveBeenCalledWith(keypair);
    expect(result).toBe(fakeTx);
  });

  it('wraps a signing failure in a SorobanTransactionError with stage "sign"', () => {
    const keypair = Keypair.random();
    const fakeTx = {
      sign: () => {
        throw new Error('cannot sign');
      },
    } as any;

    expect(() => signTransaction(fakeTx, keypair)).toThrow(SorobanTransactionError);
    try {
      signTransaction(fakeTx, keypair);
    } catch (error) {
      expect((error as SorobanTransactionError).stage).toBe('sign');
    }
  });
});

describe('submitTransaction', () => {
  afterEach(() => {
    jest.restoreAllMocks();
    __resetSorobanServerForTests();
  });

  it('returns the send response on PENDING', async () => {
    const response = { status: 'PENDING', hash: 'tx-hash-1' };
    jest.spyOn(rpc.Server.prototype, 'sendTransaction').mockResolvedValue(response as any);

    const fakeTx = {} as any;
    await expect(submitTransaction(fakeTx)).resolves.toEqual(response);
  });

  it('passes through a DUPLICATE response instead of treating it as a failure', async () => {
    const response = { status: 'DUPLICATE', hash: 'tx-hash-2' };
    jest.spyOn(rpc.Server.prototype, 'sendTransaction').mockResolvedValue(response as any);

    const fakeTx = {} as any;
    await expect(submitTransaction(fakeTx)).resolves.toEqual(response);
  });

  it('throws a SorobanTransactionError with stage "submit" and the tx hash on ERROR', async () => {
    const response = {
      status: 'ERROR',
      hash: 'tx-hash-3',
      errorResult: 'Error(Contract, #18)',
    };
    jest.spyOn(rpc.Server.prototype, 'sendTransaction').mockResolvedValue(response as any);

    const fakeTx = {} as any;
    await expect(submitTransaction(fakeTx)).rejects.toMatchObject({
      stage: 'submit',
      contractErrorCode: 18,
      txHash: 'tx-hash-3',
    });
  });

  it('wraps a network-level submission failure in a SorobanTransactionError with stage "submit"', async () => {
    jest.spyOn(rpc.Server.prototype, 'sendTransaction').mockRejectedValue(new Error('network down'));

    const fakeTx = {} as any;
    await expect(submitTransaction(fakeTx)).rejects.toMatchObject({ stage: 'submit', txHash: null });
  });
});
