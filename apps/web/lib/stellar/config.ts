import { z } from "zod";
import { StrKey } from "@stellar/stellar-sdk";

// ─── Zod Schema for Stellar Network Config ──────────────────────────────────

/**
 * Validates a Stellar public key (G...) using the official SDK.
 */
function validatePublicKey(value: string): boolean {
  try {
    return StrKey.isValidEd25519PublicKey(value);
  } catch {
    return false;
  }
}

/**
 * Validates a Stellar contract ID (C...) using the official SDK.
 */
function validateContractId(value: string): boolean {
  try {
    return StrKey.isValidContractStrKey(value);
  } catch {
    return false;
  }
}

export const stellarNetworkConfigSchema = z.object({
  rpcUrl: z.string().url("Invalid RPC URL format"),
  horizonUrl: z.string().url("Invalid Horizon URL format"),
  networkPassphrase: z.string().min(1, "Network passphrase cannot be empty"),
  networkId: z.enum(["TESTNET", "MAINNET"], {
    errorMap: () => ({
      message: 'Network ID must be either "TESTNET" or "MAINNET"',
    }),
  }),
  eventContractId: z.string().refine(validateContractId, {
    message: "Invalid event contract ID format (must start with C...)",
  }),
  ticketContractId: z.string().refine(validateContractId, {
    message: "Invalid ticket contract ID format (must start with C...)",
  }),
  usdcAssetCode: z.string().default("USDC"),
  testnetUsdcIssuer: z.string().refine(validatePublicKey, {
    message: "Invalid Testnet USDC issuer (must be a valid G... public key)",
  }),
  mainnetUsdcIssuer: z.string().refine(validatePublicKey, {
    message: "Invalid Mainnet USDC issuer (must be a valid G... public key)",
  }),
});

export type StellarNetworkConfig = z.infer<typeof stellarNetworkConfigSchema>;

// ─── Config Loading & Validation ────────────────────────────────────────────

/**
 * Loads and validates Stellar network configuration from environment variables.
 * Fails fast at startup with descriptive errors if validation fails.
 *
 * @throws Error if required env vars are missing or invalid
 * @returns Frozen config object with all validated parameters
 */
function loadStellarConfig(): StellarNetworkConfig {
  const isTestnet =
    (process.env.NEXT_PUBLIC_STELLAR_NETWORK || "TESTNET").toUpperCase() ===
    "TESTNET";

  const config = {
    rpcUrl:
      process.env.NEXT_PUBLIC_SOROBAN_RPC_URL ||
      "https://soroban-testnet.stellar.org",
    horizonUrl:
      process.env.NEXT_PUBLIC_HORIZON_URL ||
      (isTestnet
        ? "https://horizon-testnet.stellar.org"
        : "https://horizon.stellar.org"),
    networkPassphrase:
      process.env.NEXT_PUBLIC_STELLAR_NETWORK_PASSPHRASE ||
      (isTestnet
        ? "Test SDF Network ; September 2015"
        : "Public Global Stellar Network ; September 2015"),
    networkId: isTestnet ? ("TESTNET" as const) : ("MAINNET" as const),
    eventContractId:
      process.env.NEXT_PUBLIC_EVENT_CONTRACT_ID ||
      process.env.STELLAR_CONTRACT_ADDRESS ||
      "",
    ticketContractId:
      process.env.NEXT_PUBLIC_TICKET_PAYMENT_CONTRACT_ID ||
      process.env.STELLAR_CONTRACT_ADDRESS ||
      "",
    usdcAssetCode: "USDC",
    testnetUsdcIssuer:
      process.env.NEXT_PUBLIC_TESTNET_USDC_ISSUER ||
      "GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5",
    mainnetUsdcIssuer:
      process.env.NEXT_PUBLIC_MAINNET_USDC_ISSUER ||
      "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN",
  };

  // Validate schema — will throw with a descriptive error if validation fails
  const validatedConfig = stellarNetworkConfigSchema.parse(config);

  // Ensure contract IDs are not empty (defensive check)
  if (!validatedConfig.eventContractId || !validatedConfig.ticketContractId) {
    throw new Error(
      "CRITICAL: Event and ticket contract IDs must be configured in .env. " +
        "Set NEXT_PUBLIC_EVENT_CONTRACT_ID and NEXT_PUBLIC_TICKET_PAYMENT_CONTRACT_ID.",
    );
  }

  return validatedConfig;
}

// ─── Frozen Singleton Config ────────────────────────────────────────────────

/**
 * Global, frozen Stellar network configuration.
 * Validated and loaded once at startup; immutable after initialization.
 *
 * Usage:
 * ```ts
 * import { stellarConfig } from "@/lib/stellar/config";
 * console.log(stellarConfig.rpcUrl);
 * console.log(stellarConfig.eventContractId);
 * ```
 */
let stellarConfig: StellarNetworkConfig | null = null;

export function getStellarConfig(): StellarNetworkConfig {
  if (stellarConfig === null) {
    stellarConfig = loadStellarConfig();
    // Freeze to prevent accidental mutations
    Object.freeze(stellarConfig);
  }
  return stellarConfig;
}

// Eagerly load config at module initialization (for early error detection)
if (typeof window === "undefined") {
  // Server-side only
  try {
    getStellarConfig();
  } catch (error) {
    console.error(
      "[Stellar Config] Failed to initialize configuration at startup:",
      error,
    );
    throw error;
  }
}

// ─── Helper Functions ───────────────────────────────────────────────────────

/**
 * Returns the appropriate USDC issuer address for the current network.
 */
export function getUsdcIssuer(): string {
  const config = getStellarConfig();
  return config.networkId === "TESTNET"
    ? config.testnetUsdcIssuer
    : config.mainnetUsdcIssuer;
}

/**
 * Returns whether we are operating on Testnet or Mainnet.
 */
export function isTestnet(): boolean {
  return getStellarConfig().networkId === "TESTNET";
}
