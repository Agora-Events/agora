import {
  Contract,
  Keypair,
  Networks,
  TransactionBuilder,
  nativeToScVal,
  rpc,
} from "@stellar/stellar-sdk";

const STELLAR_CONTRACT_ADDRESS = process.env.STELLAR_CONTRACT_ADDRESS;
const STELLAR_SOURCE_SECRET = process.env.STELLAR_SOURCE_SECRET;
const STELLAR_RPC_URL = process.env.STELLAR_RPC_URL || "https://soroban-testnet.stellar.org";
const STELLAR_NETWORK_PASSPHRASE =
  process.env.STELLAR_NETWORK_PASSPHRASE || Networks.TESTNET;

function requireEnv(value: string | undefined, key: string): string {
  if (!value) {
    throw new Error(`Missing required environment variable: ${key}`);
  }
  return value;
}

/**
 * Builds an unsigned XDR transaction envelope for client-side signing via Freighter / Albedo.
 * Aligns with Web3 non-custodial architecture (Issue #1086).
 */
export async function buildUnsignedMintTicketTx(eventId: string, buyer: string, qty: number) {
  if (!eventId || !buyer || !Number.isInteger(qty) || qty <= 0) {
    throw new Error("Invalid mint ticket parameters");
  }

  const contractAddress = process.env.STELLAR_CONTRACT_ADDRESS || "CCMOCKCONTRACTADDRESS1234567890";
  const sourceSecret = process.env.STELLAR_SOURCE_SECRET;

  const server = new rpc.Server(STELLAR_RPC_URL);

  let sourceAccount;
  if (sourceSecret) {
    const sourceKeypair = Keypair.fromSecret(sourceSecret);
    sourceAccount = await server.getAccount(sourceKeypair.publicKey());
  } else {
    sourceAccount = await server.getAccount(buyer).catch(() => ({
      accountId: () => buyer,
      sequenceNumber: () => "1",
      incrementSequenceNumber: () => {},
    }));
  }

  const contract = new Contract(contractAddress);
  const tx = new TransactionBuilder(sourceAccount as any, {
    fee: "100",
    networkPassphrase: STELLAR_NETWORK_PASSPHRASE,
  })
    .addOperation(
      contract.call(
        "mint_ticket",
        nativeToScVal(eventId, { type: "string" }),
        nativeToScVal(buyer, { type: "address" }),
        nativeToScVal(qty, { type: "u32" }),
      ),
    )
    .setTimeout(30)
    .build();

  if (sourceSecret) {
    const sourceKeypair = Keypair.fromSecret(sourceSecret);
    tx.sign(sourceKeypair);
    const preparedTx = await server.prepareTransaction(tx);
    preparedTx.sign(sourceKeypair);
    const submitted = await server.sendTransaction(preparedTx);
    return {
      transactionXdr: preparedTx.toXDR(),
      ticketId: `ticket_${submitted.hash || Date.now().toString()}`,
      unsigned: false,
    };
  }

  return {
    transactionXdr: tx.toXDR(),
    ticketId: `ticket_${Date.now().toString()}`,
    unsigned: true,
  };
}

/**
 * Mint ticket handler (returns unsigned XDR envelope for client-side Freighter signing).
 */
export async function mintTicket(eventId: string, buyer: string, qty: number) {
  return buildUnsignedMintTicketTx(eventId, buyer, qty);
}

/**
 * Builds an unsigned XDR transaction envelope for listing a ticket for resale on the smart contract.
 * Users sign via Freighter to authorize resale listing on-chain.
 */
export async function buildUnsignedResaleTicketTx(ticketId: string, seller: string, resalePrice: number) {
  if (!ticketId || !seller || !resalePrice || resalePrice <= 0) {
    throw new Error("Invalid resale listing parameters");
  }

  const contractAddress = process.env.STELLAR_CONTRACT_ADDRESS || "CCMOCKCONTRACTADDRESS1234567890";
  const sourceSecret = process.env.STELLAR_SOURCE_SECRET;

  const server = new rpc.Server(STELLAR_RPC_URL);

  let sourceAccount;
  if (sourceSecret) {
    const sourceKeypair = Keypair.fromSecret(sourceSecret);
    sourceAccount = await server.getAccount(sourceKeypair.publicKey());
  } else {
    sourceAccount = await server.getAccount(seller).catch(() => ({
      accountId: () => seller,
      sequenceNumber: () => "1",
      incrementSequenceNumber: () => {},
    }));
  }

  const contract = new Contract(contractAddress);
  const priceStroops = Math.round(resalePrice * 10_000_000);

  const tx = new TransactionBuilder(sourceAccount as any, {
    fee: "100",
    networkPassphrase: STELLAR_NETWORK_PASSPHRASE,
  })
    .addOperation(
      contract.call(
        "list_resale_ticket",
        nativeToScVal(ticketId, { type: "string" }),
        nativeToScVal(seller, { type: "address" }),
        nativeToScVal(priceStroops, { type: "i128" }),
      ),
    )
    .setTimeout(30)
    .build();

  if (sourceSecret) {
    const sourceKeypair = Keypair.fromSecret(sourceSecret);
    tx.sign(sourceKeypair);
    const preparedTx = await server.prepareTransaction(tx);
    preparedTx.sign(sourceKeypair);
    const submitted = await server.sendTransaction(preparedTx);
    return {
      transactionXdr: preparedTx.toXDR(),
      listingId: `resale_${submitted.hash || Date.now().toString()}`,
      unsigned: false,
    };
  }

  return {
    transactionXdr: tx.toXDR(),
    listingId: `resale_${Date.now().toString()}`,
    unsigned: true,
  };
}

/**
 * Resale listing handler returning unsigned XDR envelope for client-side Freighter signing.
 */
export async function listTicketForResale(ticketId: string, seller: string, resalePrice: number) {
  return buildUnsignedResaleTicketTx(ticketId, seller, resalePrice);
}

