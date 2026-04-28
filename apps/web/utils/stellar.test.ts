import { beforeEach, describe, expect, it, vi } from "vitest";

const callMock = vi.fn(() => "contract_operation");
const signMock = vi.fn();
const buildMock = vi.fn(() => ({ sign: signMock }));
const setTimeoutMock = vi.fn(() => ({ build: buildMock }));
const addOperationMock = vi.fn(() => ({ setTimeout: setTimeoutMock }));
const txBuilderMock = vi.fn(() => ({ addOperation: addOperationMock }));

const sendTransactionMock = vi.fn(async () => ({ hash: "abc123" }));
const preparedSignMock = vi.fn();
const preparedToXdrMock = vi.fn(() => "xdr_value");
const prepareTransactionMock = vi.fn(async () => ({
  sign: preparedSignMock,
  toXDR: preparedToXdrMock,
}));
const getAccountMock = vi.fn(async () => ({ id: "source_account" }));
const serverMock = vi.fn(() => ({
  getAccount: getAccountMock,
  prepareTransaction: prepareTransactionMock,
  sendTransaction: sendTransactionMock,
}));

const fromSecretMock = vi.fn(() => ({ publicKey: () => "source_pk" }));

vi.mock("@stellar/stellar-sdk", () => ({
  Contract: vi.fn(() => ({ call: callMock })),
  Keypair: { fromSecret: fromSecretMock },
  Networks: { TESTNET: "Test SDF Network ; September 2015" },
  TransactionBuilder: txBuilderMock,
  nativeToScVal: vi.fn((value: unknown) => value),
  rpc: { Server: serverMock },
}));

describe("mintTicket", () => {
  beforeEach(() => {
    vi.resetModules();
    vi.clearAllMocks();
    process.env.STELLAR_CONTRACT_ADDRESS = "CABC";
    process.env.STELLAR_SOURCE_SECRET = "SABC";
    process.env.STELLAR_RPC_URL = "https://rpc.example.org";
    process.env.STELLAR_NETWORK_PASSPHRASE = "TESTNET";
  });

  it("builds and submits mint ticket transaction with expected parameters", async () => {
    const { mintTicket } = await import("./stellar");
    const result = await mintTicket("evt_1", "GBUYER", 2);

    expect(fromSecretMock).toHaveBeenCalledWith("SABC");
    expect(serverMock).toHaveBeenCalledWith("https://rpc.example.org");
    expect(getAccountMock).toHaveBeenCalledWith("source_pk");
    expect(callMock).toHaveBeenCalledWith(
      "mint_ticket",
      "evt_1",
      "GBUYER",
      2,
    );
    expect(addOperationMock).toHaveBeenCalledWith("contract_operation");
    expect(signMock).toHaveBeenCalledTimes(1);
    expect(prepareTransactionMock).toHaveBeenCalledTimes(1);
    expect(preparedSignMock).toHaveBeenCalledTimes(1);
    expect(sendTransactionMock).toHaveBeenCalledTimes(1);
    expect(result).toEqual({
      ticketId: "ticket_abc123",
      transactionXdr: "xdr_value",
    });
  });
});
