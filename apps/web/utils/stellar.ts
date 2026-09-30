/**
 * Stellar / Soroban utility functions — Issues #1493
 *
 * All rpc.Server instantiation is now delegated to lib/stellar/client.ts so
 * every call targets https://soroban-testnet.stellar.org with consistent
 * timeout/retry configuration.
 *
 * No deprecated SorobanRpc.Server or simulateTransaction calls remain here;
 * server.prepareTransaction() is used throughout (SDK v13+ recommended path).
 */

import {
  Contract,
  Keypair,
  TransactionBuilder,
  nativeToScVal,
} from "@stellar/stellar-sdk";

import {
  getSorobanRpcServer,
  STELLAR_NETWORK_PASSPHRASE,
} from "@/lib/stellar/client";

// ---------------------------------------------------------------------------
// Environment helpers
// ---------------------------------------------------------------------------

const STELLAR_CONTRACT_ADDRESS = process.env.STELLAR_CONTRACT_ADDRESS;
const STELLAR_SOURCE_SECRET = process.env.STELLAR_SOURCE_SECRET;

// ---------------------------------------------------------------------------
// buildUnsignedMintTicketTx
// ---------------------------------------------------------------------------

/**
 * Builds an unsigned XDR transaction envelope for client-side signing via
 * Freighter / Albedo.  Aligns with Web3 non-custodial architecture (#1086).
 *
 * When STELLAR_SOURCE_SECRET is present the transaction is built, simulated
 * (via server.prepareTransaction), signed, and submitted server-side —
 * returning `unsigned: false`.
 *
 * Without a server secret the raw unsigned XDR is returned for the client to
 * sign — returning `unsigned: true`.
 */
export async function buildUnsignedMintTicketTx(
  eventId: string,
  buyer: string,
  qty: number,
) {
  if (!eventId || !buyer || !Number.isInteger(qty) || qty <= 0) {
    throw new Error("Invalid mint ticket parameters");
  }

  const contractAddress =
    STELLAR_CONTRACT_ADDRESS ?? "CCMOCKCONTRACTADDRESS1234567890";

  const server = getSorobanRpcServer();

  let sourceAccount;
  if (STELLAR_SOURCE_SECRET) {
    const sourceKeypair = Keypair.fromSecret(STELLAR_SOURCE_SECRET);
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

  if (STELLAR_SOURCE_SECRET) {
    const sourceKeypair = Keypair.fromSecret(STELLAR_SOURCE_SECRET);
    tx.sign(sourceKeypair);
    // prepareTransaction handles simulation + resource-fee attachment (v13+).
    const preparedTx = await server.prepareTransaction(tx);
    preparedTx.sign(sourceKeypair);
    const submitted = await server.sendTransaction(preparedTx);
    return {
      transactionXdr: preparedTx.toXDR(),
      ticketId: `ticket_${submitted.hash ?? Date.now().toString()}`,
      unsigned: false,
    };
  }

  return {
    transactionXdr: tx.toXDR(),
    ticketId: `ticket_${Date.now().toString()}`,
    unsigned: true,
  };
}

// ---------------------------------------------------------------------------
// mintTicket
// ---------------------------------------------------------------------------

/**
 * Mint ticket handler — returns unsigned XDR envelope for client-side
 * Freighter signing.
 */
export async function mintTicket(
  eventId: string,
  buyer: string,
  qty: number,
) {
  return buildUnsignedMintTicketTx(eventId, buyer, qty);
}

// ---------------------------------------------------------------------------
// buildUnsignedResaleTicketTx
// ---------------------------------------------------------------------------

/**
 * Builds an unsigned XDR transaction envelope for listing a ticket for resale
 * on the smart contract.  Users sign via Freighter to authorise the listing.
 */
export async function buildUnsignedResaleTicketTx(
  ticketId: string,
  seller: string,
  resalePrice: number,
) {
  if (!ticketId || !seller || !resalePrice || resalePrice <= 0) {
    throw new Error("Invalid resale listing parameters");
  }

  const contractAddress =
    STELLAR_CONTRACT_ADDRESS ?? "CCMOCKCONTRACTADDRESS1234567890";

  const server = getSorobanRpcServer();

  let sourceAccount;
  if (STELLAR_SOURCE_SECRET) {
    const sourceKeypair = Keypair.fromSecret(STELLAR_SOURCE_SECRET);
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

  if (STELLAR_SOURCE_SECRET) {
    const sourceKeypair = Keypair.fromSecret(STELLAR_SOURCE_SECRET);
    tx.sign(sourceKeypair);
    // prepareTransaction handles simulation + sorobanData attachment (v13+).
    const preparedTx = await server.prepareTransaction(tx);
    preparedTx.sign(sourceKeypair);
    const submitted = await server.sendTransaction(preparedTx);
    return {
      transactionXdr: preparedTx.toXDR(),
      listingId: `resale_${submitted.hash ?? Date.now().toString()}`,
      unsigned: false,
    };
  }

  return {
    transactionXdr: tx.toXDR(),
    listingId: `resale_${Date.now().toString()}`,
    unsigned: true,
  };
}

// ---------------------------------------------------------------------------
// listTicketForResale
// ---------------------------------------------------------------------------

/**
 * Resale listing handler — returns unsigned XDR envelope for client-side
 * Freighter signing.
 */
export async function listTicketForResale(
  ticketId: string,
  seller: string,
  resalePrice: number,
) {
  return buildUnsignedResaleTicketTx(ticketId, seller, resalePrice);
}
