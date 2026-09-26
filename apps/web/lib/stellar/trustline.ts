"use client";

import {
  Asset,
  Networks,
  Operation,
  TransactionBuilder,
  rpc as StellarRpc,
} from "@stellar/stellar-sdk";

// ─── Constants ────────────────────────────────────────────────────────────────

/**
 * The canonical Testnet USDC issuer used by Circle / Stellar testnet faucets.
 * Swap for the real Circle issuer on Mainnet.
 */
export const TESTNET_USDC_ISSUER =
  "GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5";

export const MAINNET_USDC_ISSUER =
  "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN";

const STELLAR_RPC_URL =
  process.env.NEXT_PUBLIC_SOROBAN_RPC_URL ||
  "https://soroban-testnet.stellar.org";

const STELLAR_NETWORK =
  process.env.NEXT_PUBLIC_STELLAR_NETWORK || "TESTNET";

/**
 * Returns whether we are operating on Testnet or Mainnet, based on the
 * NEXT_PUBLIC_STELLAR_NETWORK env variable.
 */
export function isTestnet(): boolean {
  return STELLAR_NETWORK.toUpperCase() !== "MAINNET";
}

/**
 * Returns the correct USDC asset for the current network.
 */
export function getUsdcAsset(): Asset {
  const issuer = isTestnet() ? TESTNET_USDC_ISSUER : MAINNET_USDC_ISSUER;
  return new Asset("USDC", issuer);
}

// ─── Types ────────────────────────────────────────────────────────────────────

export interface TrustlineCheckResult {
  /** Whether the account has a USDC trustline set up. */
  hasTrustline: boolean;
  /** Whether the account exists at all on the network. */
  accountExists: boolean;
}

// ─── Core function ────────────────────────────────────────────────────────────

/**
 * Checks whether a Stellar account has a USDC trustline on the current network.
 *
 * A trustline is a voluntary link between a Stellar account and a specific
 * asset issuer. Without a trustline, an account cannot receive or hold USDC.
 * This is a common stumbling block for new contributors testing ticket purchases
 * on Testnet.
 *
 * @param publicKey - The G... Stellar public key to inspect.
 * @returns A result indicating trustline presence and account existence.
 *
 * @example
 * ```ts
 * const result = await checkUsdcTrustline("GABC...");
 * if (!result.hasTrustline) {
 *   // Show "Add USDC Trustline" UI
 * }
 * ```
 */
export async function checkUsdcTrustline(
  publicKey: string,
): Promise<TrustlineCheckResult> {
  if (!publicKey || !publicKey.startsWith("G") || publicKey.length !== 56) {
    throw new Error(`Invalid Stellar public key: "${publicKey}"`);
  }

  const server = new StellarRpc.Server(STELLAR_RPC_URL);
  const usdcAsset = getUsdcAsset();

  try {
    const account = await server.getAccount(publicKey);

    // The SDK's AccountResponse exposes balances via .balances on the
    // underlying Horizon-compatible structure when using rpc.Server.
    // We reach them through the raw account object.
    const balances: Array<{ asset_type: string; asset_code?: string; asset_issuer?: string }> =
      (account as unknown as { balances?: Array<{ asset_type: string; asset_code?: string; asset_issuer?: string }> })
        .balances ?? [];

    const hasTrustline = balances.some(
      (b) =>
        b.asset_type !== "native" &&
        b.asset_code === usdcAsset.getCode() &&
        b.asset_issuer === usdcAsset.getIssuer(),
    );

    return { hasTrustline, accountExists: true };
  } catch (err: unknown) {
    // RPC throws when the account does not exist on the network
    const message = err instanceof Error ? err.message : String(err);
    if (
      message.includes("not found") ||
      message.includes("404") ||
      message.includes("does not exist")
    ) {
      return { hasTrustline: false, accountExists: false };
    }
    throw err;
  }
}

// ─── Trustline transaction builder ───────────────────────────────────────────

/**
 * Builds an unsigned ChangeTrust XDR transaction envelope that the user must
 * sign with Freighter to establish a USDC trustline.
 *
 * This is a client-side helper: the returned XDR is passed to
 * `freighter.signTransaction()` and then submitted to the network.
 *
 * @param publicKey - The G... public key of the signer / trustline creator.
 * @returns The unsigned XDR string ready for Freighter signing.
 */
export async function buildAddTrustlineXdr(publicKey: string): Promise<string> {
  const server = new StellarRpc.Server(STELLAR_RPC_URL);
  const networkPassphrase = isTestnet()
    ? Networks.TESTNET
    : Networks.PUBLIC;

  let accountData;
  try {
    accountData = await server.getAccount(publicKey);
  } catch {
    throw new Error(
      "Account not found on the Stellar network. Fund it via Friendbot before adding a trustline.",
    );
  }

  const usdcAsset = getUsdcAsset();

  const tx = new TransactionBuilder(accountData as unknown as Parameters<typeof TransactionBuilder>[0], {
    fee: "100",
    networkPassphrase,
  })
    .addOperation(
      Operation.changeTrust({
        asset: usdcAsset,
      }),
    )
    .setTimeout(30)
    .build();

  return tx.toXDR();
}

// ─── Freighter integration ────────────────────────────────────────────────────

/**
 * High-level helper that:
 * 1. Builds the ChangeTrust XDR for the given account.
 * 2. Prompts the user to sign via Freighter.
 * 3. Submits the signed transaction to the network.
 *
 * Returns the transaction hash on success.
 *
 * @param publicKey - The G... public key initiating the trustline.
 * @throws If Freighter is not installed, the user rejects, or submission fails.
 */
export async function addUsdcTrustlineViaFreighter(
  publicKey: string,
): Promise<string> {
  const freighter = await import("@stellar/freighter-api");

  const isConnected = await freighter.isConnected();
  if (!isConnected) {
    throw new Error(
      "Freighter wallet is not installed. Please install it from https://freighter.app and try again.",
    );
  }

  const xdr = await buildAddTrustlineXdr(publicKey);
  const networkPassphrase = isTestnet()
    ? Networks.TESTNET
    : Networks.PUBLIC;

  // Request user signature
  let signedXdr: string;
  try {
    const result = await freighter.signTransaction(xdr, { networkPassphrase });
    // freighter-api v2 returns { signedTxXdr } or just the string depending on version
    signedXdr =
      typeof result === "string"
        ? result
        : (result as { signedTxXdr: string }).signedTxXdr;
  } catch (err) {
    throw new Error(
      `Freighter signing was cancelled or failed: ${err instanceof Error ? err.message : String(err)}`,
    );
  }

  // Submit the signed transaction
  const server = new StellarRpc.Server(STELLAR_RPC_URL);
  const { Transaction } = await import("@stellar/stellar-sdk");
  const signedTx = new Transaction(
    signedXdr,
    isTestnet() ? Networks.TESTNET : Networks.PUBLIC,
  );

  const submitResult = await server.sendTransaction(signedTx);

  if (submitResult.status === "ERROR") {
    throw new Error(
      `Trustline transaction failed on-chain: ${JSON.stringify(submitResult.errorResult)}`,
    );
  }

  return submitResult.hash;
}
