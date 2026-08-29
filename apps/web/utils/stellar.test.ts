import { beforeEach, describe, expect, it, vi } from "vitest";

const callMock = vi.fn(() => "contract_operation");
const signMock = vi.fn();
const buildMock = vi.fn(() => ({ sign: signMock, toXDR: () => "xdr_value" }));
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

  it("builds and submits mint ticket transaction with expected parameters when secret provided", async () => {
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
      unsigned: false,
    });
  });

  it("constructs unsigned XDR envelope when source secret is absent for client-side signing", async () => {
    delete process.env.STELLAR_SOURCE_SECRET;
    const { buildUnsignedMintTicketTx } = await import("./stellar");
    const result = await buildUnsignedMintTicketTx("evt_1", "GBUYER", 2);

    expect(callMock).toHaveBeenCalledWith(
      "mint_ticket",
      "evt_1",
      "GBUYER",
      2,
    );
    expect(result.unsigned).toBe(true);
    expect(result.transactionXdr).toBe("xdr_value");
  });

  it("builds and submits resale listing transaction with expected parameters", async () => {
    const { listTicketForResale } = await import("./stellar");
    const result = await listTicketForResale("tkt_123", "GSELLER", 50);

    expect(callMock).toHaveBeenCalledWith(
      "list_resale_ticket",
      "tkt_123",
      "GSELLER",
      500_000_000,
    );
    expect(result.listingId).toContain("resale_");
    expect(result.transactionXdr).toBe("xdr_value");
  });

  it("throws error for invalid resale parameters", async () => {
    const { buildUnsignedResaleTicketTx } = await import("./stellar");
    await expect(buildUnsignedResaleTicketTx("", "GSELLER", 50)).rejects.toThrow("Invalid resale listing parameters");
    await expect(buildUnsignedResaleTicketTx("tkt_123", "", 50)).rejects.toThrow("Invalid resale listing parameters");
    await expect(buildUnsignedResaleTicketTx("tkt_123", "GSELLER", 0)).rejects.toThrow("Invalid resale listing parameters");
  });
});

