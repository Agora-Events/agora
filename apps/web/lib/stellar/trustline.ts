"use client";

import {
  Asset,
  Networks,
  Operation,
  TransactionBuilder,
  rpc as StellarRpc,
} from "@stellar/stellar-sdk";
import { getStellarConfig, getUsdcIssuer, isTestnet } from "./config";

// ─── Backward compatibility exports (deprecated — use config.ts instead) ─────

/**
 * @deprecated Use getStellarConfig().testnetUsdcIssuer or getUsdcIssuer() instead.
 * The canonical Testnet USDC issuer used by Circle / Stellar testnet faucets.
 */
export const TESTNET_USDC_ISSUER = "GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5";

/**
 * @deprecated Use getStellarConfig().mainnetUsdcIssuer instead.
 * Real Circle issuer on Mainnet.
 */
export const MAINNET_USDC_ISSUER = "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN";

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

  const config = getStellarConfig();
  const server = new StellarRpc.Server(config.rpcUrl);
  const issuer = getUsdcIssuer();
  const usdcAsset = new Asset("USDC", issuer);

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
  const config = getStellarConfig();
  const server = new StellarRpc.Server(config.rpcUrl);
  const networkPassphrase = config.networkPassphrase;

  let accountData;
  try {
    accountData = await server.getAccount(publicKey);
  } catch {
    throw new Error(
      "Account not found on the Stellar network. Fund it via Friendbot before adding a trustline.",
    );
  }

  const issuer = getUsdcIssuer();
  const usdcAsset = new Asset("USDC", issuer);

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
  const config = getStellarConfig();

  const isConnected = await freighter.isConnected();
  if (!isConnected) {
    throw new Error(
      "Freighter wallet is not installed. Please install it from https://freighter.app and try again.",
    );
  }

  const xdr = await buildAddTrustlineXdr(publicKey);
  const networkPassphrase = config.networkPassphrase;

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
  const server = new StellarRpc.Server(config.rpcUrl);
  const { Transaction } = await import("@stellar/stellar-sdk");
  const signedTx = new Transaction(
    signedXdr,
    config.networkPassphrase,
  );

  const submitResult = await server.sendTransaction(signedTx);

  if (submitResult.status === "ERROR") {
    throw new Error(
      `Trustline transaction failed on-chain: ${JSON.stringify(submitResult.errorResult)}`,
    );
  }

  return submitResult.hash;
}
